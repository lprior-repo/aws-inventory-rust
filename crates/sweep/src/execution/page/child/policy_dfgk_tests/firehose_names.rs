mod cases;
mod model;
use super::*;
use catalog_tool::{endpoints, kind_map, overlay, runtime_catalog, smithy};
use model::compile;
use std::sync::LazyLock;

const KIND: &str = "AWS::KinesisFirehose::DeliveryStream";
const LAST_NAME: &str = "stream.Middle-09_";
const MAX_NAME: &str = concat!(
    "zzzzzzzz", "zzzzzzzz", "zzzzzzzz", "zzzzzzzz", "zzzzzzzz", "zzzzzzzz", "zzzzzzzz", "zzzzzzzz"
);
const FIRST_ARN: &str = "arn:aws:firehose:us-east-1:123456789012:deliverystream/a";
const LAST_ARN: &str = "arn:aws:firehose:us-east-1:123456789012:deliverystream/stream.Middle-09_";
const NEW_ARN: &str = "arn:aws:firehose:us-east-1:123456789012:deliverystream/b";
const MAX_ARN: &str = concat!(
    "arn:aws:firehose:us-east-1:123456789012:deliverystream/",
    "zzzzzzzz",
    "zzzzzzzz",
    "zzzzzzzz",
    "zzzzzzzz",
    "zzzzzzzz",
    "zzzzzzzz",
    "zzzzzzzz",
    "zzzzzzzz"
);
static CATALOG: LazyLock<Result<Vec<u8>, String>> =
    LazyLock::new(|| compile().map_err(|error| error.to_string()));

type ScopedKey = (String, String, String, String);
type ResourceRow = (
    String,
    String,
    String,
    String,
    Option<String>,
    Vec<u8>,
    i64,
    i64,
);
#[derive(Debug, PartialEq, Eq)]
struct FacetRow {
    key: ScopedKey,
    source: (String, String, Vec<u8>, Option<Vec<u8>>, i64),
    content: (Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>, Option<String>, i64),
}
type OwnedKey = (
    String,
    String,
    String,
    String,
    Vec<u8>,
    i64,
    i64,
    String,
    String,
);
type QueueRow = (
    Vec<u8>,
    Option<Vec<u8>>,
    Vec<u8>,
    Vec<u8>,
    i64,
    i64,
    i64,
    i64,
    i64,
);

fn catalog() -> TestResult<RuntimeCatalog<'static>> {
    match &*CATALOG {
        Ok(bytes) => Ok(RuntimeCatalog::from_bytes(bytes)?),
        Err(error) => Err(format!("pinned Firehose production plan failed: {error}").into()),
    }
}

async fn claim(scenario: &Scenario<'_, '_>, now: i64) -> TestResult<WorkLease> {
    let lease = scenario
        .store
        .claim(&scenario.sweep, now, 10_000)
        .await?
        .ok_or("Firehose continuation was lost")?;
    assert_eq!(lease.scope().service().as_str(), "firehose");
    assert_eq!(lease.scope().operation().as_str(), "ListDeliveryStreams");
    assert_eq!(lease.scope().account().as_str(), "123456789012");
    assert_eq!(
        lease.scope().region(),
        &Region::Regional(RegionId::parse("us-east-1")?)
    );
    assert!(lease.scope().parent().is_none());
    Ok(lease)
}

fn native_request(
    scenario: &Scenario<'_, '_>,
    plan: &CallablePlan<'_>,
    lease: &WorkLease,
    expected: Json,
) -> TestResult<Record> {
    let (record, wire) = scenario.prepared_request(plan, lease)?;
    assert_eq!(wire.method(), http::Method::POST);
    assert_eq!(
        wire.uri().to_string(),
        "https://firehose.us-east-1.amazonaws.com/"
    );
    assert_eq!(
        wire.headers()
            .get("x-amz-target")
            .ok_or("native Firehose target absent")?
            .to_str()?,
        "Firehose_20150804.ListDeliveryStreams"
    );
    assert_eq!(
        serde_json::from_slice::<Json>(wire.body())?,
        expected,
        "native request must not filter DeliveryStreamType or invent ARN input"
    );
    Ok(record)
}

async fn first_page(
    scenario: &Scenario<'_, '_>,
    catalog: &RuntimeCatalog<'_>,
    plan: &CallablePlan<'_>,
) -> TestResult {
    let lease = claim(scenario, 1001).await?;
    assert!(lease.token().is_none());
    let request = native_request(scenario, plan, &lease, json!({}))?;
    scenario
        .apply(
            catalog,
            plan,
            lease,
            &request,
            json!({"DeliveryStreamNames":["a",LAST_NAME],"HasMoreDeliveryStreams":true}),
        )
        .await
}

#[derive(Debug, PartialEq, Eq)]
struct DurableState {
    resources: Vec<ResourceRow>,
    facets: Vec<FacetRow>,
    owners: Vec<OwnedKey>,
    queue: Vec<QueueRow>,
    tokens: Vec<(Vec<u8>, Vec<u8>)>,
    outcomes: Vec<(Vec<u8>, String, i64)>,
    edges: i64,
}

async fn facets(reader: &sqlx::SqlitePool) -> TestResult<Vec<FacetRow>> {
    use sqlx::Row;
    let rows = sqlx::query("SELECT account,region,kind,id,service,operation,source_key,source_authority,precedence,record,owned,exclusions,relations,name,name_owned FROM resource_facets ORDER BY account,region,kind,id,service,operation,source_key").fetch_all(reader).await?;
    Ok(rows
        .into_iter()
        .map(|row| {
            Ok(FacetRow {
                key: (
                    row.try_get("account")?,
                    row.try_get("region")?,
                    row.try_get("kind")?,
                    row.try_get("id")?,
                ),
                source: (
                    row.try_get("service")?,
                    row.try_get("operation")?,
                    row.try_get("source_key")?,
                    row.try_get("source_authority")?,
                    row.try_get("precedence")?,
                ),
                content: (
                    row.try_get("record")?,
                    row.try_get("owned")?,
                    row.try_get("exclusions")?,
                    row.try_get("relations")?,
                    row.try_get("name")?,
                    row.try_get("name_owned")?,
                ),
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()?)
}

async fn state(reader: &sqlx::SqlitePool) -> TestResult<DurableState> {
    Ok(DurableState {
        resources: sqlx::query_as("SELECT account,region,kind,id,name,content_hash,first_seen,last_changed FROM resources ORDER BY account,region,kind,id").fetch_all(reader).await?,
        facets: facets(reader).await?,
        owners: sqlx::query_as("SELECT account,region,kind,id,scope_key,last_seen,missing_complete,observed_sweep,completed_sweep FROM resource_scopes ORDER BY account,region,kind,id,scope_key").fetch_all(reader).await?,
        queue: sqlx::query_as("SELECT scope_key,token,bindings,context,state,generation,authority_generation,authority_complete,lease_until FROM work_queue ORDER BY sweep_id,scope_key").fetch_all(reader).await?,
        tokens: sqlx::query_as("SELECT scope_key,digest FROM work_tokens ORDER BY sweep_id,scope_key,digest").fetch_all(reader).await?,
        outcomes: sqlx::query_as("SELECT scope_key,outcome,observed_at FROM scope_outcomes ORDER BY sweep_id,scope_key").fetch_all(reader).await?,
        edges: sqlx::query_scalar("SELECT COUNT(*) FROM edges").fetch_one(reader).await?,
    })
}

async fn reopen<'a, 'catalog>(
    scenario: Scenario<'a, 'catalog>,
) -> TestResult<Scenario<'a, 'catalog>> {
    let Scenario {
        directory,
        store,
        sweep,
        account,
        home,
        consumers,
    } = scenario;
    let path = directory.path().join("inventory.db");
    drop(store);
    let store = BinaryStore::open(&path).await?;
    Ok(Scenario {
        directory,
        store,
        sweep,
        account,
        home,
        consumers,
    })
}

async fn assert_persisted(
    scenario: Scenario<'_, '_>,
    expected: &[&str],
    outcome: &str,
) -> TestResult {
    let reopened = reopen(scenario).await?;
    assert!(
        reopened
            .store
            .claim(&reopened.sweep, 12_000, 10_000)
            .await?
            .is_none(),
        "unexpected DescribeDeliveryStream or extra continuation"
    );
    let reader = reopened.reader().await?;
    let expected: Vec<ScopedKey> = expected
        .iter()
        .map(|id| {
            (
                "123456789012".to_owned(),
                "us-east-1".to_owned(),
                KIND.to_owned(),
                (*id).to_owned(),
            )
        })
        .collect();
    let actual: Vec<ScopedKey> = sqlx::query_as(
        "SELECT account,region,kind,id FROM resources ORDER BY account,region,kind,id",
    )
    .fetch_all(&reader)
    .await?;
    assert_eq!(actual, expected);
    let owners: Vec<ScopedKey> = sqlx::query_as("SELECT account,region,kind,id FROM resource_scopes ORDER BY account,region,kind,id,scope_key").fetch_all(&reader).await?;
    assert_eq!(owners, expected, "duplicate or wrong durable owner");
    let outcomes: Vec<String> =
        sqlx::query_scalar("SELECT outcome FROM scope_outcomes ORDER BY scope_key")
            .fetch_all(&reader)
            .await?;
    assert_eq!(outcomes, vec![outcome.to_owned()]);
    let queue: Vec<(i64, Option<Vec<u8>>)> =
        sqlx::query_as("SELECT state,token FROM work_queue ORDER BY scope_key")
            .fetch_all(&reader)
            .await?;
    assert_eq!(queue, vec![(2, None)]);
    let markers: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM work_tokens")
        .fetch_one(&reader)
        .await?;
    assert_eq!(markers, 1);
    let edges: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM edges")
        .fetch_one(&reader)
        .await?;
    assert_eq!(edges, 0);
    reader.close().await;
    Ok(())
}
