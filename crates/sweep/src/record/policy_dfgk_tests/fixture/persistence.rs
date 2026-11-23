use super::*;
pub(in crate::record::policy_dfgk_tests) async fn persist(
    bytes: &[u8],
    plan: &CallablePlan<'_>,
    observations: Vec<Observation>,
) -> TestResult<Vec<(String, String, Vec<u8>)>> {
    use inventory_core::inventory::{OperationId, ScopeKey, ServiceId};
    use inventory_engine::inventory::{BinaryStore, PageCommit, SchemaRegistry, SweepId, WorkSpec};
    let root = tempfile::tempdir()?;
    let path = root.path().join("inventory.db");
    let runtime = RuntimeCatalog::from_bytes(bytes)?;
    let mut unresolved = Vec::new();
    for observation in &observations {
        unresolved.extend(crate::resource_projection::trust_references(
            &runtime,
            plan,
            observation,
        )?);
    }
    let expected_references = i64::try_from(unresolved.len())?;
    let registry = SchemaRegistry::new(bytes, runtime.schemas().to_vec())?;
    let store = BinaryStore::create(&path, registry).await?;
    let sweep = SweepId::new("policy-consumer-scenario".to_owned())?;
    let region = if plan.scope() == PlanScope::Account {
        Region::Global
    } else {
        Region::Regional(RegionId::parse("us-east-1")?)
    };
    let scope = ScopeKey::new(
        AccountId::parse("123456789012")?,
        region,
        ServiceId::parse(plan.service_name())?,
        OperationId::parse(plan.operation_name())?,
        None,
    )?;
    let bindings = if input::requires_parent(plan) {
        Vec::new()
    } else {
        let account = AccountId::parse("123456789012")?;
        input::root_bindings(plan, Some(&account), Some(&scope.region().clone()))?.into_bytes()
    };
    store
        .seed_work(WorkSpec::new(sweep.clone(), scope, None, bindings)?)
        .await?;
    let lease = store
        .claim(&sweep, 1000, 10000)
        .await?
        .ok_or("seeded work not claimable")?;
    store
        .commit_page(PageCommit {
            lease,
            observations,
            edges: Vec::new(),
            unresolved,
            tags: Vec::new(),
            next_token: None,
            continuation_digest: None,
            children: Vec::new(),
            done: true,
            observed_at_ms: 1001,
        })
        .await?;
    let reader = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename(&path)
                .read_only(true),
        )
        .await?;
    let saved = sqlx::query_as("SELECT kind,id,record FROM resource_facets ORDER BY kind,id")
        .fetch_all(&reader)
        .await?;
    let reference_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM unresolved_refs")
        .fetch_one(&reader)
        .await?;
    assert_eq!(reference_count, expected_references);
    reader.close().await;
    Ok(saved)
}
