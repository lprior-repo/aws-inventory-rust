use super::*;
mod projection;
pub(super) use projection::{Resource, Edge, key, assert_state, unbranched_source};
use crate::execution::{
    draining,
    page::{PageRequest, assembly, cursor, preparation},
};
use inventory_engine::inventory::{BinaryStore, PageCommit, SchemaRegistry, WorkLease};
use std::{collections::BTreeSet, path::Path};

async fn seed(
    store: &BinaryStore,
    catalog: &RuntimeCatalog<'_>,
    sweep: &SweepId,
    reverse: bool,
) -> TestResult {
    let account = AccountId::parse("123456789012")?;
    let region = RegionId::parse("us-west-2")?;
    let mut work = Vec::new();
    for plan in catalog.plans() {
        draining::append_plan_work(
            &account,
            &region,
            sweep,
            plan,
            &[Region::Regional(region.clone())],
            &mut work,
        )?;
    }
    if reverse {
        work.reverse();
    }
    store.seed_works(&work).await?;
    Ok(())
}

pub(super) fn wire(
    plan: &CallablePlan<'_>,
    lease: &WorkLease,
) -> TestResult<(PageRequest, http::Request<bytes::Bytes>)> {
    let home = RegionId::parse("us-east-1")?;
    let region =
        routing::request_region(plan, lease.scope().region(), &home, lease.scope().branch())?;
    let endpoint = routing::resolve(plan, &region)?;
    let base = Record::from_bytes(lease.bindings().to_vec())?;
    let mut fields = cursor::Cursor::decode(lease.token())?.request_fields(plan.pagination())?;
    if let Some(size) = plan.pagination().page_size.as_ref() {
        fields.push(input::InputField {
            path: size.path.clone(),
            value: size.value(),
        });
    }
    let wire = crate::request::encode(plan, &endpoint.url, &base, &fields)?;
    Ok((
        PageRequest {
            record: input::merge_input(&base, &fields)?,
            region,
        },
        wire,
    ))
}

fn response(
    plan: &CallablePlan<'_>,
    wire: &Json,
    requests: &mut BTreeSet<(String, String)>,
) -> TestResult<Json> {
    assert_eq!(wire.get("Regions"), Some(&json!(["us-west-2"])));
    let organization = wire
        .get("OrganizationId")
        .and_then(Json::as_str)
        .ok_or("organization absent")?;
    let index = organization
        .strip_prefix("org-")
        .ok_or("wrong organization")?;
    if plan.operation_name() != "ListChildren" {
        assert_eq!(plan.operation_name(), format!("ListParents{index}"));
        return Ok(if wire.get("NextToken").is_none() {
            json!({"Items":[],"NextToken":"parents-next"})
        } else {
            assert_eq!(wire.get("NextToken"), Some(&json!("parents-next")));
            json!({"Items":[{"Id":format!("parent-{index}")}]})
        });
    }
    let entity = wire
        .get("EntityId")
        .and_then(Json::as_str)
        .ok_or("entity absent")?;
    assert_eq!(entity, format!("parent-{index}"));
    requests.insert((organization.to_owned(), entity.to_owned()));
    Ok(if wire.get("NextToken").is_none() {
        json!({"Items":[{"Id":format!("child-{index}-first")}],"NextToken":"children-next"})
    } else {
        assert_eq!(wire.get("NextToken"), Some(&json!("children-next")));
        json!({"Items":[{"Id":format!("child-{index}-second")} ]})
    })
}

pub(super) async fn apply(
    store: &BinaryStore,
    catalog: &RuntimeCatalog<'_>,
    plan: &CallablePlan<'_>,
    lease: WorkLease,
    request: &PageRequest,
    body: Json,
) -> TestResult {
    let response = crate::call::DecodedResponse {
        record: decode(plan, &serde_json::to_vec(&body)?)?,
        headers: http::HeaderMap::new(),
    };
    let account = AccountId::parse("123456789012")?;
    let home = RegionId::parse("us-east-1")?;
    let consumers = ConsumerIndex::build(catalog)?;
    let executor = inventory_engine::limiter::ApiExecutor::new(Default::default())?;
    let context = assembly::Context {
        catalog,
        account: &account,
        home: &home,
        consumers: &consumers,
        executor: &executor,
        max_pages: 10,
    };
    let ancestry = preparation::ancestry(lease.context(), lease.scope().context())?;
    let cursor = cursor::Cursor::decode(lease.token())?;
    let parts =
        assembly::prepare(&context, &lease, plan, cursor, response, request, &ancestry).await?;
    let done = parts.next.is_none();
    let continuation_digest = parts
        .next
        .as_ref()
        .and_then(|advance| advance.continuation_digest);
    store
        .commit_page(PageCommit {
            lease,
            observations: parts.observations,
            children: parts.children,
            unresolved: parts.references,
            tags: parts.tags,
            edges: Vec::new(),
            next_token: parts.next.map(|advance| advance.cursor.into_bytes()),
            continuation_digest,
            done,
            observed_at_ms: 1001,
        })
        .await?;
    Ok(())
}

async fn run(
    path: &Path,
    catalog: &RuntimeCatalog<'_>,
    sweep: &SweepId,
) -> TestResult<BTreeSet<(String, String)>> {
    let mut requests = BTreeSet::new();
    for _ in 0..catalog.plans().len().saturating_sub(1).saturating_mul(4) {
        // Reopen the durable queue before every page, including empty parent pages.
        let store = BinaryStore::open_with_catalog(path, catalog.raw_bytes()).await?;
        let lease = store
            .claim(sweep, 1001, 10000)
            .await?
            .ok_or("expected page absent after restart")?;
        let plan = catalog
            .lookup("bindings", lease.scope().operation().as_str())
            .ok_or("leased plan absent")?;
        let (request, wire) = wire(plan, &lease)?;
        let body = response(plan, &serde_json::from_slice(wire.body())?, &mut requests)?;
        apply(&store, catalog, plan, lease, &request, body).await?;
    }
    let store = BinaryStore::open_with_catalog(path, catalog.raw_bytes()).await?;
    assert!(
        store.claim(sweep, 1001, 10000).await?.is_none(),
        "duplicate or Cartesian child work remained"
    );
    Ok(requests)
}

async fn saved(path: &Path) -> TestResult<Vec<(String, String, Vec<u8>)>> {
    let reader = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename(path)
                .read_only(true),
        )
        .await?;
    let rows = sqlx::query_as("SELECT kind,id,record FROM resource_facets ORDER BY kind,id")
        .fetch_all(&reader)
        .await?;
    reader.close().await;
    Ok(rows)
}

pub(super) async fn synthetic(count: usize) -> TestResult {
    let bytes = synthetic::compiled(count, |_| {})?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("alternatives.db");
    let registry = SchemaRegistry::new(catalog.raw_bytes(), catalog.schemas().to_vec())?;
    let store = BinaryStore::create(&path, registry).await?;
    let first = SweepId::new("alternatives-first".to_owned())?;
    seed(&store, &catalog, &first, false).await?;
    drop(store);
    let expected = (0..count)
        .map(|index| (format!("org-{index}"), format!("parent-{index}")))
        .collect::<BTreeSet<_>>();
    assert_eq!(run(&path, &catalog, &first).await?, expected);
    let resources = (0..count)
        .flat_map(|index| {
            [
                key(
                    "us-west-2",
                    "AWS::ApiOnly::BindingParent",
                    &format!("parent-{index}"),
                ),
                key(
                    "us-west-2",
                    "AWS::ApiOnly::BindingChild",
                    &format!("child-{index}-first"),
                ),
                key(
                    "us-west-2",
                    "AWS::ApiOnly::BindingChild",
                    &format!("child-{index}-second"),
                ),
            ]
        })
        .collect::<BTreeSet<_>>();
    let edges = (0..count)
        .flat_map(|index| {
            ["first", "second"].map(move |suffix| {
                (
                    key(
                        "us-west-2",
                        "AWS::ApiOnly::BindingChild",
                        &format!("child-{index}-{suffix}"),
                    ),
                    "IN_PARENT".to_owned(),
                    key(
                        "us-west-2",
                        "AWS::ApiOnly::BindingParent",
                        &format!("parent-{index}"),
                    ),
                )
            })
        })
        .collect::<BTreeSet<_>>();
    assert_state(&path, &catalog, &resources, &edges).await?;
    let original = saved(&path).await?;
    assert_eq!(
        original
            .iter()
            .filter(|(kind, _, _)| kind == "AWS::ApiOnly::BindingParent")
            .count(),
        count
    );
    assert_eq!(
        original
            .iter()
            .filter(|(kind, _, _)| kind == "AWS::ApiOnly::BindingChild")
            .count(),
        count.saturating_mul(2)
    );
    let second = SweepId::new("alternatives-reordered".to_owned())?;
    let store = BinaryStore::open_with_catalog(&path, catalog.raw_bytes()).await?;
    seed(&store, &catalog, &second, true).await?;
    drop(store);
    assert_eq!(run(&path, &catalog, &second).await?, expected);
    assert_eq!(
        saved(&path).await?,
        original,
        "resource keys or records changed with producer order"
    );
    assert_state(&path, &catalog, &resources, &edges).await?;
    Ok(())
}

#[tokio::test]
async fn given_alternative_roots_when_empty_and_child_pages_restart_then_sqlite_keeps_exact_parent_inputs_and_stable_rows()
-> TestResult {
    synthetic(2).await
}

