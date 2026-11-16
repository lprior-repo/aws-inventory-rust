use super::*;
use inventory_core::{catalog::runtime::ContextProjectionPlan, inventory::{DiscoveryFrame, ScopeOutcome}, record::{Exclusions, Key}};
use inventory_engine::inventory::WorkLease;

fn portal_observation(catalog: &RuntimeCatalog<'_>, operation: &str, id: &str, name: &str) -> TestResult<Observation> {
    let plan = catalog.lookup("workspaces-web", operation).ok_or("authority plan absent")?;
    let mut resource = key()?;
    resource.id = ResourceId::parse(id)?;
    let record = Record::encode(&Value::Map(BTreeMap::from([
        (Key::Field(FieldId::new(5)?), Value::Text(id.to_owned())),
        (Key::Field(FieldId::new(3)?), Value::Text(name.to_owned())),
    ])))?;
    Ok(Observation::new(resource, record, &Exclusions::none(), Some(name.into()))?.with_facet_precedence(plan.facet_precedence()))
}

fn contextual_work(sweep: &SweepId, ordinal: u32, parent: Option<ResourceKey>) -> TestResult<WorkSpec> {
    let selected = Record::encode(&Value::Map(BTreeMap::from([
        (Key::Field(FieldId::new(5)?), Value::Text(PORTAL.to_owned())),
    ])))?;
    let empty = Record::encode(&Value::Map(BTreeMap::new()))?;
    let projection = ContextProjectionPlan { selected: vec![vec![FieldId::new(5)?].into_boxed_slice()].into_boxed_slice(), ..ContextProjectionPlan::default() };
    let context = DiscoveryContext::empty().push(DiscoveryFrame::project(
        ServiceId::parse("workspaces-web")?, OperationId::parse("ListPortals")?, ordinal,
        &projection, &selected, &empty, &empty,
    )?)?;
    let scope = ScopeKey::new(AccountId::parse("123456789012")?, Region::parse("us-east-1")?, ServiceId::parse("workspaces-web")?, OperationId::parse("GetPortal")?, parent)?
        .with_context(context.discriminator()?);
    Ok(WorkSpec::new(sweep.clone(), scope, None, selected.into_bytes())?.with_context(context.encode()?)?)
}

async fn page(store: &BinaryStore, lease: WorkLease, observations: Vec<Observation>, children: Vec<WorkSpec>, at: i64) -> TestResult {
    store.commit_page(PageCommit { lease, observations, children, edges: Vec::new(), unresolved: Vec::new(), tags: Vec::new(), next_token: None, continuation_digest: None, done: true, observed_at_ms: at }).await?;
    Ok(())
}

async fn single(store: &BinaryStore, catalog: &RuntimeCatalog<'_>, sweep: &str, ordinal: u32, positive: bool, at: i64) -> TestResult {
    let sweep = SweepId::new(sweep.to_owned())?;
    store.seed_work(contextual_work(&sweep, ordinal, None)?).await?;
    let lease = store.claim(&sweep, at, 10_000).await?.ok_or("authority lease absent")?;
    let observations = if positive { vec![portal_observation(catalog, "GetPortal", PORTAL, "stable")?] } else { Vec::new() };
    page(store, lease, observations, Vec::new(), at + 1).await
}

async fn pair(store: &BinaryStore, sweep: &str, ordinal: u32, at: i64) -> TestResult<(WorkLease, WorkLease)> {
    let sweep = SweepId::new(sweep.to_owned())?;
    store.seed_work(contextual_work(&sweep, ordinal, None)?).await?;
    store.seed_work(contextual_work(&sweep, ordinal.checked_add(1).ok_or("ordinal overflow")?, None)?).await?;
    let first = store.claim(&sweep, at, 10_000).await?.ok_or("first duplicate context absent")?;
    let second = store.claim(&sweep, at, 10_000).await?.ok_or("second duplicate context absent")?;
    assert_ne!(first.scope().context(), second.scope().context());
    assert_eq!(first.bindings(), second.bindings());
    Ok((first, second))
}

async fn ownership(directory: &tempfile::TempDir) -> TestResult<(i64, i64)> {
    let pool = sqlx::SqlitePool::connect(&format!("sqlite://{}", directory.path().join("inventory.sqlite").display())).await?;
    let row = sqlx::query("SELECT count(*) AS owners,max(missing_complete) AS misses FROM resource_scopes WHERE id=?1").bind(PORTAL).fetch_one(&pool).await?;
    let result = (row.try_get("owners")?, row.try_get("misses")?);
    pool.close().await;
    Ok(result)
}

#[tokio::test]
async fn given_equivalent_contexts_when_empty_and_positive_complete_in_either_order_then_no_transient_tombstone_or_first_seen_reset() -> TestResult {
    let bytes = compiled()?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    for positive_first in [false, true] {
        let (directory, store) = store(&catalog).await?;
        single(&store, &catalog, "initial", 0, true, 1).await?;
        let original = store.projection_resource(&key()?).await?.ok_or("initial resource absent")?;
        let original_seen = first_seen(&directory).await?;
        single(&store, &catalog, "missing-once", 1, false, 3).await?;
        assert_eq!(ownership(&directory).await?, (1, 1));
        let (first, second) = pair(&store, "duplicate-positive", 2, 5).await?;
        let observations = if positive_first { vec![portal_observation(&catalog, "GetPortal", PORTAL, "stable")?] } else { Vec::new() };
        page(&store, first, observations, Vec::new(), 6).await?;
        assert!(store.projection_resource(&key()?).await?.is_some(), "resource transiently tombstoned");
        assert_eq!(first_seen(&directory).await?, original_seen);
        let observations = if positive_first { Vec::new() } else { vec![portal_observation(&catalog, "GetPortal", PORTAL, "stable")?] };
        page(&store, second, observations, Vec::new(), 7).await?;
        assert_eq!(store.projection_resource(&key()?).await?.ok_or("positive resource absent")?, original);
        assert_eq!(first_seen(&directory).await?, original_seen);
        assert_eq!(ownership(&directory).await?, (1, 0));
        let (first, second) = pair(&store, "duplicate-missing-one", 4, 8).await?;
        page(&store, first, Vec::new(), Vec::new(), 9).await?;
        assert_eq!(ownership(&directory).await?, (1, 0));
        page(&store, second, Vec::new(), Vec::new(), 10).await?;
        assert_eq!(ownership(&directory).await?, (1, 1));
        let (first, second) = pair(&store, "duplicate-missing-two", 6, 11).await?;
        page(&store, first, Vec::new(), Vec::new(), 12).await?;
        assert!(store.projection_resource(&key()?).await?.is_some());
        page(&store, second, Vec::new(), Vec::new(), 13).await?;
        assert!(store.projection_resource(&key()?).await?.is_none());
    }
    Ok(())
}

#[tokio::test]
async fn given_equivalent_partial_and_complete_executions_when_sweep_drains_then_partial_blocks_negative_authority() -> TestResult {
    let bytes = compiled()?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let (directory, store) = store(&catalog).await?;
    single(&store, &catalog, "initial", 0, true, 1).await?;
    single(&store, &catalog, "missing-once", 1, false, 3).await?;
    let (complete, partial) = pair(&store, "partial-and-complete", 2, 5).await?;
    page(&store, complete, Vec::new(), Vec::new(), 6).await?;
    store.finish_scope(partial, ScopeOutcome::Partial, 7).await?;
    assert!(store.projection_resource(&key()?).await?.is_some());
    assert_eq!(ownership(&directory).await?, (1, 1));
    single(&store, &catalog, "actual-second-miss", 4, false, 8).await?;
    assert!(store.projection_resource(&key()?).await?.is_none());
    Ok(())
}

#[tokio::test]
async fn given_pending_parent_discovery_when_late_equivalent_positive_context_is_admitted_then_absence_waits_for_its_completion() -> TestResult {
    let bytes = compiled()?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let (directory, store) = store(&catalog).await?;
    single(&store, &catalog, "initial", 0, true, 1).await?;
    let original_seen = first_seen(&directory).await?;
    single(&store, &catalog, "missing-once", 1, false, 3).await?;
    let sweep = SweepId::new("late-discovery".to_owned())?;
    store.seed_work(contextual_work(&sweep, 2, None)?).await?;
    let root = ScopeKey::new(AccountId::parse("123456789012")?, Region::parse("us-east-1")?, ServiceId::parse("workspaces-web")?, OperationId::parse("ListPortals")?, None)?;
    store.seed_work(WorkSpec::new(sweep.clone(), root, None, Record::encode(&Value::Map(BTreeMap::new()))?.into_bytes())?).await?;
    let left = store.claim(&sweep, 5, 10_000).await?.ok_or("first pending lease absent")?;
    let right = store.claim(&sweep, 5, 10_000).await?.ok_or("second pending lease absent")?;
    let (empty, discovery) = if left.scope().operation().as_str() == "GetPortal" { (left, right) } else { (right, left) };
    page(&store, empty, Vec::new(), Vec::new(), 6).await?;
    assert!(store.projection_resource(&key()?).await?.is_some(), "resource deleted before discovery");
    assert_eq!(first_seen(&directory).await?, original_seen);
    page(&store, discovery, vec![portal_observation(&catalog, "ListPortals", PORTAL, "stable")?], vec![contextual_work(&sweep, 3, None)?], 7).await?;
    let late = store.claim(&sweep, 8, 10_000).await?.ok_or("late positive context absent")?;
    page(&store, late, vec![portal_observation(&catalog, "GetPortal", PORTAL, "stable")?], Vec::new(), 9).await?;
    assert!(store.projection_resource(&key()?).await?.is_some(), "late positive lost");
    assert_eq!(first_seen(&directory).await?, original_seen);
    Ok(())
}

async fn parent_detail(
    store: &BinaryStore, catalog: &RuntimeCatalog<'_>, parent: &ResourceKey,
    sweep: &str, ordinal: u32, name: Option<&str>, at: i64,
) -> TestResult {
    let sweep = SweepId::new(sweep.to_owned())?;
    store.seed_work(contextual_work(&sweep, ordinal, Some(parent.clone()))?).await?;
    let lease = store.claim(&sweep, at, 10_000).await?.ok_or("parent-owned detail absent")?;
    let observations = name.map(|name| portal_observation(catalog, "GetPortal", PORTAL, name)).transpose()?.into_iter().collect();
    page(store, lease, observations, Vec::new(), at + 1).await
}

#[tokio::test]
async fn given_multiple_parent_and_root_owners_when_parent_retires_then_child_keeps_live_lower_facets_and_origin_links_detach() -> TestResult {
    const PARENT_ONE: &str = "arn:aws:workspaces-web:us-east-1:123456789012:portal/parent-1";
    const PARENT_TWO: &str = "arn:aws:workspaces-web:us-east-1:123456789012:portal/parent-2";
    let bytes = compiled()?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let (directory, store) = store(&catalog).await?;
    persist(&store, &catalog, "ListPortals", json!({"portals":[
        {"portalArn":PARENT_ONE}, {"portalArn":PARENT_TWO}, {"portalArn":PORTAL,"displayName":"root"}
    ]}), 1).await?;
    let original_seen = first_seen(&directory).await?;
    let mut one = key()?;
    one.id = ResourceId::parse(PARENT_ONE)?;
    let mut two = key()?;
    two.id = ResourceId::parse(PARENT_TWO)?;
    parent_detail(&store, &catalog, &one, "one-detail", 0, Some("parent-one"), 2).await?;
    parent_detail(&store, &catalog, &two, "two-detail", 1, Some("parent-two"), 4).await?;
    assert_eq!(field(&state(&directory, &catalog).await?.0, 3)?, Some(Value::Text("parent-two".to_owned())));
    let surviving = json!({"portals":[{"portalArn":PARENT_ONE},{"portalArn":PORTAL,"displayName":"root"}]});
    persist(&store, &catalog, "ListPortals", surviving.clone(), 6).await?;
    drop(store);
    let reopened = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
    persist(&reopened, &catalog, "ListPortals", surviving, 7).await?;
    assert!(reopened.projection_resource(&two).await?.is_none());
    let child = reopened.projection_resource(&key()?).await?.ok_or("independently owned child deleted")?;
    assert_eq!(first_seen(&directory).await?, original_seen);
    assert_eq!(child.name.as_deref(), Some("parent-one"));
    assert_eq!(field(&state(&directory, &catalog).await?.0, 3)?, Some(Value::Text("parent-one".to_owned())));
    parent_detail(&reopened, &catalog, &one, "one-miss-once", 2, None, 8).await?;
    parent_detail(&reopened, &catalog, &one, "one-miss-twice", 3, None, 10).await?;
    assert_eq!(reopened.projection_resource(&key()?).await?.ok_or("root-owned child deleted")?.name.as_deref(), Some("root"));
    let pool = sqlx::SqlitePool::connect(&format!("sqlite://{}", directory.path().join("inventory.sqlite").display())).await?;
    assert_eq!(sqlx::query_scalar::<_, i64>("SELECT count(*) FROM resource_parents WHERE child_id=?1").bind(PORTAL).fetch_one(&pool).await?, 0);
    assert_eq!(sqlx::query_scalar::<_, i64>("SELECT count(*) FROM resource_scopes WHERE id=?1 AND missing_complete<2").bind(PORTAL).fetch_one(&pool).await?, 1);
    pool.close().await;
    let child_only = json!({"portals":[{"portalArn":PORTAL,"displayName":"root"}]});
    persist(&reopened, &catalog, "ListPortals", child_only.clone(), 12).await?;
    persist(&reopened, &catalog, "ListPortals", child_only, 13).await?;
    assert!(reopened.projection_resource(&one).await?.is_none());
    let child = reopened.projection_resource(&key()?).await?.ok_or("detached root-owned child deleted")?;
    assert_eq!(first_seen(&directory).await?, original_seen);
    assert_eq!(child.name.as_deref(), Some("root"));
    assert_eq!(reopened.projection_resources(None, 100).await?.len(), 1);
    Ok(())
}

async fn first_seen(directory: &tempfile::TempDir) -> TestResult<i64> {
    let pool = sqlx::SqlitePool::connect(&format!("sqlite://{}", directory.path().join("inventory.sqlite").display())).await?;
    let first_seen = sqlx::query_scalar("SELECT first_seen FROM resources WHERE id=?1").bind(PORTAL).fetch_one(&pool).await?;
    pool.close().await;
    Ok(first_seen)
}

#[path = "authority_partition_tests.rs"]
mod partitions;
