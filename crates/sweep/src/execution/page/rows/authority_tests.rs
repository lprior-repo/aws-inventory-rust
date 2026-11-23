use super::*;
#[path = "authority_tests/completion.rs"]
mod completion;
use inventory_core::{
    catalog::runtime::ContextProjectionPlan,
    inventory::{DiscoveryFrame, ScopeOutcome},
    record::{Exclusions, Key},
};
use inventory_engine::inventory::WorkLease;

fn portal_observation(
    catalog: &RuntimeCatalog<'_>,
    operation: &str,
    id: &str,
    name: &str,
) -> TestResult<Observation> {
    let plan = catalog
        .lookup("workspaces-web", operation)
        .ok_or("authority plan absent")?;
    let mut resource = key()?;
    resource.id = ResourceId::parse(id)?;
    let record = Record::encode(&Value::Map(BTreeMap::from([
        (Key::Field(FieldId::new(5)?), Value::Text(id.to_owned())),
        (Key::Field(FieldId::new(3)?), Value::Text(name.to_owned())),
    ])))?;
    Ok(
        Observation::new(resource, record, &Exclusions::none(), Some(name.into()))?
            .with_facet_precedence(plan.facet_precedence()),
    )
}

fn contextual_work(
    sweep: &SweepId,
    ordinal: u32,
    parent: Option<ResourceKey>,
) -> TestResult<WorkSpec> {
    let selected = Record::encode(&Value::Map(BTreeMap::from([(
        Key::Field(FieldId::new(5)?),
        Value::Text(PORTAL.to_owned()),
    )])))?;
    let empty = Record::encode(&Value::Map(BTreeMap::new()))?;
    let projection = ContextProjectionPlan {
        selected: vec![vec![FieldId::new(5)?].into_boxed_slice()].into_boxed_slice(),
        ..ContextProjectionPlan::default()
    };
    let context = DiscoveryContext::empty().push(DiscoveryFrame::project(
        ServiceId::parse("workspaces-web")?,
        OperationId::parse("ListPortals")?,
        ordinal,
        &projection,
        &selected,
        &empty,
        &empty,
    )?)?;
    let scope = ScopeKey::new(
        AccountId::parse("123456789012")?,
        Region::parse("us-east-1")?,
        ServiceId::parse("workspaces-web")?,
        OperationId::parse("GetPortal")?,
        parent,
    )?
    .with_context(context.discriminator()?);
    Ok(
        WorkSpec::new(sweep.clone(), scope, None, selected.into_bytes())?
            .with_context(context.encode()?)?,
    )
}

async fn page(
    store: &BinaryStore,
    lease: WorkLease,
    observations: Vec<Observation>,
    children: Vec<WorkSpec>,
    at: i64,
) -> TestResult {
    store
        .commit_page(PageCommit {
            lease,
            observations,
            children,
            edges: Vec::new(),
            unresolved: Vec::new(),
            tags: Vec::new(),
            next_token: None,
            continuation_digest: None,
            done: true,
            observed_at_ms: at,
        })
        .await?;
    Ok(())
}

async fn single(
    store: &BinaryStore,
    catalog: &RuntimeCatalog<'_>,
    sweep: &str,
    ordinal: u32,
    positive: bool,
    at: i64,
) -> TestResult {
    let sweep = SweepId::new(sweep.to_owned())?;
    store
        .seed_work(contextual_work(&sweep, ordinal, None)?)
        .await?;
    let lease = store
        .claim(&sweep, at, 10_000)
        .await?
        .ok_or("authority lease absent")?;
    let observations = if positive {
        vec![portal_observation(catalog, "GetPortal", PORTAL, "stable")?]
    } else {
        Vec::new()
    };
    page(store, lease, observations, Vec::new(), at.saturating_add(1)).await
}

async fn pair(
    store: &BinaryStore,
    sweep: &str,
    ordinal: u32,
    at: i64,
) -> TestResult<(WorkLease, WorkLease)> {
    let sweep = SweepId::new(sweep.to_owned())?;
    store
        .seed_work(contextual_work(&sweep, ordinal, None)?)
        .await?;
    store
        .seed_work(contextual_work(
            &sweep,
            ordinal.checked_add(1).ok_or("ordinal overflow")?,
            None,
        )?)
        .await?;
    let first = store
        .claim(&sweep, at, 10_000)
        .await?
        .ok_or("first duplicate context absent")?;
    let second = store
        .claim(&sweep, at, 10_000)
        .await?
        .ok_or("second duplicate context absent")?;
    assert_ne!(first.scope().context(), second.scope().context());
    assert_eq!(first.bindings(), second.bindings());
    Ok((first, second))
}

async fn ownership(directory: &tempfile::TempDir) -> TestResult<(i64, i64)> {
    let pool = sqlx::SqlitePool::connect(&format!(
        "sqlite://{}",
        directory.path().join("inventory.sqlite").display()
    ))
    .await?;
    let row = sqlx::query("SELECT count(*) AS owners,max(missing_complete) AS misses FROM resource_scopes WHERE id=?1").bind(PORTAL).fetch_one(&pool).await?;
    let result = (row.try_get("owners")?, row.try_get("misses")?);
    pool.close().await;
    Ok(result)
}

async fn parent_detail(
    store: &BinaryStore,
    catalog: &RuntimeCatalog<'_>,
    parent: &ResourceKey,
    sweep: &str,
    ordinal: u32,
    name: Option<&str>,
    at: i64,
) -> TestResult {
    let sweep = SweepId::new(sweep.to_owned())?;
    store
        .seed_work(contextual_work(&sweep, ordinal, Some(parent.clone()))?)
        .await?;
    let lease = store
        .claim(&sweep, at, 10_000)
        .await?
        .ok_or("parent-owned detail absent")?;
    let observations = name
        .map(|name| portal_observation(catalog, "GetPortal", PORTAL, name))
        .transpose()?
        .into_iter()
        .collect();
    page(store, lease, observations, Vec::new(), at.saturating_add(1)).await
}

async fn first_seen(directory: &tempfile::TempDir) -> TestResult<i64> {
    let pool = sqlx::SqlitePool::connect(&format!(
        "sqlite://{}",
        directory.path().join("inventory.sqlite").display()
    ))
    .await?;
    let first_seen = sqlx::query_scalar("SELECT first_seen FROM resources WHERE id=?1")
        .bind(PORTAL)
        .fetch_one(&pool)
        .await?;
    pool.close().await;
    Ok(first_seen)
}

#[path = "authority_partition_tests.rs"]
mod partitions;
