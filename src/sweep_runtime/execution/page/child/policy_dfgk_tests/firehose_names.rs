use super::*;
use catalog_tool::{endpoints, kind_map, overlay, runtime_catalog, smithy};
use std::sync::LazyLock;

const KIND: &str = "AWS::KinesisFirehose::DeliveryStream";
const LAST_NAME: &str = "stream.Middle-09_";
const MAX_NAME: &str = concat!("zzzzzzzz", "zzzzzzzz", "zzzzzzzz", "zzzzzzzz", "zzzzzzzz", "zzzzzzzz", "zzzzzzzz", "zzzzzzzz");
// Independent, pinned service-grammar oracles, never derived from observed ResourceKey IDs.
const FIRST_ARN: &str = "arn:aws:firehose:us-east-1:123456789012:deliverystream/a";
const LAST_ARN: &str = "arn:aws:firehose:us-east-1:123456789012:deliverystream/stream.Middle-09_";
const NEW_ARN: &str = "arn:aws:firehose:us-east-1:123456789012:deliverystream/b";
const MAX_ARN: &str = concat!("arn:aws:firehose:us-east-1:123456789012:deliverystream/", "zzzzzzzz", "zzzzzzzz", "zzzzzzzz", "zzzzzzzz", "zzzzzzzz", "zzzzzzzz", "zzzzzzzz", "zzzzzzzz");
static CATALOG: LazyLock<Result<Vec<u8>, String>> = LazyLock::new(|| compile().map_err(|error| error.to_string()));

type ScopedKey = (String, String, String, String);
type ResourceRow = (String, String, String, String, Option<String>, Vec<u8>, i64, i64);
#[derive(Debug, PartialEq, Eq)]
struct FacetRow {
    key: ScopedKey,
    source: (String, String, Vec<u8>, Option<Vec<u8>>, i64),
    content: (Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>, Option<String>, i64),
}
type OwnedKey = (String, String, String, String, Vec<u8>, i64, i64, String, String);
type QueueRow = (Vec<u8>, Option<Vec<u8>>, Vec<u8>, Vec<u8>, i64, i64, i64, i64, i64);

fn catalog() -> TestResult<RuntimeCatalog<'static>> {
    match &*CATALOG {
        Ok(bytes) => Ok(RuntimeCatalog::from_bytes(bytes)?),
        Err(error) => Err(format!("pinned Firehose production plan failed: {error}").into()),
    }
}

fn compile() -> TestResult<Vec<u8>> {
    let directory = tempfile::tempdir()?;
    let source = directory.path().join("source");
    crate::sweep_runtime::pinned_model::install(&source.join("models"), "firehose", "2015-08-04", &["ListDeliveryStreams", "DescribeDeliveryStream"])?;
    let embedded = overlay::embedded()?;
    let policy = embedded.iter().find(|policy| policy.service() == "firehose").ok_or("embedded Firehose policy absent")?;
    let selected = ["ListDeliveryStreams", "DescribeDeliveryStream"].into_iter().map(|name| {
        let operation = overlay::OperationName::new(name)?;
        Ok((name, policy.operations().get(&operation).ok_or("production Firehose operation absent")?.clone()))
    }).collect::<TestResult<Vec<_>>>()?;
    let enumeration = selected.iter().find(|(name, _)| *name == "ListDeliveryStreams").ok_or("Firehose enumeration absent")?;
    assert_eq!(enumeration.1.paginator, Some(overlay::Paginator::LastListItem {
        input: overlay::MemberPath::new("ExclusiveStartDeliveryStreamName")?,
        output: overlay::MemberPath::new("DeliveryStreamNames")?,
        truncated: overlay::MemberPath::new("HasMoreDeliveryStreams")?,
    }));
    let policies = [overlay::ServiceOverlay::new("firehose", selected)?];
    let checked = overlay::check_policies(&source, &policies, &catalog_tool::classes::DenyList::builtin())?;
    let fields = smithy::Registry::parse(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/catalog/field-ids.txt")))?;
    let base = smithy::build_catalog(&source, &fields, "46135f8468cd5cb60d983b97290c9f6661e359c6")?;
    let kinds = kind_map::embedded(include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/catalog/cloudcontrol-universal.json")))?;
    Ok(runtime_catalog::augment(&base, &checked, &kinds, &endpoints::verified_endpoints(&source)?)?)
}

async fn claim(scenario: &Scenario<'_, '_>, now: i64) -> TestResult<WorkLease> {
    let lease = scenario.store.claim(&scenario.sweep, now, 10_000).await?.ok_or("Firehose continuation was lost")?;
    assert_eq!(lease.scope().service().as_str(), "firehose");
    assert_eq!(lease.scope().operation().as_str(), "ListDeliveryStreams");
    assert_eq!(lease.scope().account().as_str(), "123456789012");
    assert_eq!(lease.scope().region(), &Region::Regional(RegionId::parse("us-east-1")?));
    assert!(lease.scope().parent().is_none());
    Ok(lease)
}

fn native_request(scenario: &Scenario<'_, '_>, plan: &CallablePlan<'_>, lease: &WorkLease, expected: Json) -> TestResult<Record> {
    let (record, wire) = scenario.prepared_request(plan, lease)?;
    assert_eq!(wire.method(), http::Method::POST);
    assert_eq!(wire.uri().to_string(), "https://firehose.us-east-1.amazonaws.com/");
    assert_eq!(wire.headers().get("x-amz-target").ok_or("native Firehose target absent")?.to_str()?, "Firehose_20150804.ListDeliveryStreams");
    assert_eq!(serde_json::from_slice::<Json>(wire.body())?, expected, "native request must not filter DeliveryStreamType or invent ARN input");
    Ok(record)
}

async fn first_page(scenario: &Scenario<'_, '_>, catalog: &RuntimeCatalog<'_>, plan: &CallablePlan<'_>) -> TestResult {
    let lease = claim(scenario, 1001).await?;
    assert!(lease.token().is_none());
    let request = native_request(scenario, plan, &lease, json!({}))?;
    scenario.apply(catalog, plan, lease, &request, json!({"DeliveryStreamNames":["a",LAST_NAME],"HasMoreDeliveryStreams":true})).await
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
    Ok(rows.into_iter().map(|row| {
        Ok(FacetRow {
            key: (row.try_get("account")?, row.try_get("region")?, row.try_get("kind")?, row.try_get("id")?),
            source: (row.try_get("service")?, row.try_get("operation")?, row.try_get("source_key")?, row.try_get("source_authority")?, row.try_get("precedence")?),
            content: (row.try_get("record")?, row.try_get("owned")?, row.try_get("exclusions")?, row.try_get("relations")?, row.try_get("name")?, row.try_get("name_owned")?),
        })
    }).collect::<Result<Vec<_>, sqlx::Error>>()?)
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

async fn reopen<'a, 'catalog>(scenario: Scenario<'a, 'catalog>) -> TestResult<Scenario<'a, 'catalog>> {
    let Scenario { directory, store, sweep, account, home, consumers } = scenario;
    let path = directory.path().join("inventory.db");
    drop(store);
    let store = BinaryStore::open(&path).await?;
    Ok(Scenario { directory, store, sweep, account, home, consumers })
}

async fn assert_persisted(scenario: Scenario<'_, '_>, expected: &[&str], outcome: &str) -> TestResult {
    let reopened = reopen(scenario).await?;
    assert!(reopened.store.claim(&reopened.sweep, 12_000, 10_000).await?.is_none(), "unexpected DescribeDeliveryStream or extra continuation");
    let reader = reopened.reader().await?;
    let expected: Vec<ScopedKey> = expected.iter().map(|id| ("123456789012".to_owned(), "us-east-1".to_owned(), KIND.to_owned(), (*id).to_owned())).collect();
    let actual: Vec<ScopedKey> = sqlx::query_as("SELECT account,region,kind,id FROM resources ORDER BY account,region,kind,id").fetch_all(&reader).await?;
    assert_eq!(actual, expected);
    let owners: Vec<ScopedKey> = sqlx::query_as("SELECT account,region,kind,id FROM resource_scopes ORDER BY account,region,kind,id,scope_key").fetch_all(&reader).await?;
    assert_eq!(owners, expected, "duplicate or wrong durable owner");
    let outcomes: Vec<String> = sqlx::query_scalar("SELECT outcome FROM scope_outcomes ORDER BY scope_key").fetch_all(&reader).await?;
    assert_eq!(outcomes, vec![outcome.to_owned()]);
    let queue: Vec<(i64, Option<Vec<u8>>)> = sqlx::query_as("SELECT state,token FROM work_queue ORDER BY scope_key").fetch_all(&reader).await?;
    assert_eq!(queue, vec![(2,None)]);
    let markers: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM work_tokens").fetch_one(&reader).await?;
    assert_eq!(markers, 1);
    let edges: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM edges").fetch_one(&reader).await?;
    assert_eq!(edges, 0);
    reader.close().await;
    Ok(())
}

#[tokio::test]
async fn given_native_firehose_names_when_last_name_continues_and_nonempty_final_page_stops_then_literal_scoped_arns_survive_reopen() -> TestResult {
    // Given a real pinned producer, including the one- and 64-character name boundaries.
    let catalog = catalog()?;
    assert!(catalog.lookup("firehose", "DescribeDeliveryStream").is_none());
    assert_eq!(MAX_NAME.len(), 64);
    let plan = fixture::plan(&catalog, "firehose", "ListDeliveryStreams")?;
    let scenario = Scenario::new(&catalog, "native-firehose-last-name").await?;
    scenario.seed(plan).await?;
    first_page(&scenario, &catalog, plan).await?;
    // When page two starts after the LAST native name, not the first name or an ARN.
    let lease = claim(&scenario, 1001).await?;
    let request = native_request(&scenario, plan, &lease, json!({"ExclusiveStartDeliveryStreamName":"stream.Middle-09_"}))?;
    scenario.apply(&catalog, plan, lease, &request, json!({"DeliveryStreamNames":[MAX_NAME],"HasMoreDeliveryStreams":false})).await?;
    // Then a nonempty false page is terminal and all metadata is durably scoped.
    assert_persisted(scenario, &[FIRST_ARN,LAST_ARN,MAX_ARN], "complete").await
}

#[tokio::test]
async fn given_a_durable_firehose_marker_when_true_page_has_empty_or_missing_names_then_whole_page_rejects_and_resume_keeps_prior_data() -> TestResult {
    for invalid in [json!({"DeliveryStreamNames":[],"HasMoreDeliveryStreams":true}), json!({"HasMoreDeliveryStreams":true})] {
        // Given prior real native data and a durable last-name cursor.
        let catalog = catalog()?;
        let plan = fixture::plan(&catalog, "firehose", "ListDeliveryStreams")?;
        let scenario = Scenario::new(&catalog, "native-firehose-invalid-next").await?;
        scenario.seed(plan).await?;
        first_page(&scenario, &catalog, plan).await?;
        let lease = claim(&scenario, 1001).await?;
        let saved_cursor = lease.token().ok_or("prior native cursor absent")?.to_vec();
        let request = native_request(&scenario, plan, &lease, json!({"ExclusiveStartDeliveryStreamName":"stream.Middle-09_"}))?;
        let reader = scenario.reader().await?;
        let before = state(&reader).await?;
        // When an asserted-more response has no possible last native name.
        let error = match scenario.apply(&catalog, plan, lease, &request, invalid).await {
            Ok(()) => return Err("malformed Firehose continuation was accepted".into()),
            Err(error) => error,
        };
        let Some(SweepError::Operation(message)) = error.downcast_ref::<SweepError>() else { return Err(format!("wrong native continuation error: {error}").into()) };
        assert_eq!(*message, "pagination response omitted required continuation");
        // Then no rows, authority, ledger, outcome, or cursor changed, even after reopen.
        assert_eq!(state(&reader).await?, before);
        reader.close().await;
        let resumed = reopen(scenario).await?;
        let reader = resumed.reader().await?;
        assert_eq!(state(&reader).await?, before);
        let lease = claim(&resumed, 12_000).await?;
        assert_eq!(lease.token(), Some(saved_cursor.as_slice()));
        assert_eq!(native_request(&resumed, plan, &lease, json!({"ExclusiveStartDeliveryStreamName":"stream.Middle-09_"}))?.as_bytes(), request.as_bytes());
        let actual: Vec<ScopedKey> = sqlx::query_as("SELECT account,region,kind,id FROM resources ORDER BY account,region,kind,id").fetch_all(&reader).await?;
        assert_eq!(actual, vec![("123456789012".to_owned(),"us-east-1".to_owned(),KIND.to_owned(),FIRST_ARN.to_owned()),("123456789012".to_owned(),"us-east-1".to_owned(),KIND.to_owned(),LAST_ARN.to_owned())]);
        reader.close().await;
    }
    Ok(())
}

#[tokio::test]
async fn given_a_reopened_native_firehose_cursor_when_last_marker_repeats_then_new_page_data_persists_as_partial_without_more_work() -> TestResult {
    // Given the first marker already recorded durably, not just held in memory.
    let catalog = catalog()?;
    let plan = fixture::plan(&catalog, "firehose", "ListDeliveryStreams")?;
    let scenario = Scenario::new(&catalog, "native-firehose-repeated-marker").await?;
    scenario.seed(plan).await?;
    first_page(&scenario, &catalog, plan).await?;
    let scenario = reopen(scenario).await?;
    // When a native page contributes a new stream but repeats its last marker.
    let lease = claim(&scenario, 1001).await?;
    let request = native_request(&scenario, plan, &lease, json!({"ExclusiveStartDeliveryStreamName":"stream.Middle-09_"}))?;
    scenario.apply(&catalog, plan, lease, &request, json!({"DeliveryStreamNames":["b",LAST_NAME],"HasMoreDeliveryStreams":true})).await?;
    // Then existing durable repeat protection retains data and records partial, never complete.
    assert_persisted(scenario, &[FIRST_ARN,NEW_ARN,LAST_ARN], "partial").await
}
