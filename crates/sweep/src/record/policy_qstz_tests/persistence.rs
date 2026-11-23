use super::*;
pub(super) async fn persist(
    plan: &CallablePlan<'_>,
    pages: Vec<(
        &CallablePlan<'_>,
        Option<ResourceKey>,
        Record,
        Vec<Observation>,
    )>,
) -> TestResult<(Vec<(String, String, Vec<u8>)>, Vec<(String, String)>)> {
    use inventory_core::inventory::{OperationId, Region, ScopeKey, ServiceId};
    use inventory_engine::inventory::{BinaryStore, PageCommit, SchemaRegistry, SweepId, WorkSpec};
    let root = tempfile::tempdir()?;
    let database = root.path().join("inventory.db");
    let runtime = RuntimeCatalog::from_bytes(bytes()?)?;
    let store = BinaryStore::create(
        &database,
        SchemaRegistry::new(bytes()?, runtime.schemas().to_vec())?,
    )
    .await?;
    let sweep = SweepId::new("qstz-policy-regression".to_owned())?;
    for (producer, parent, bindings, observations) in pages {
        let scope = ScopeKey::new(
            AccountId::parse("123456789012")?,
            Region::Regional(RegionId::parse("us-east-1")?),
            ServiceId::parse(producer.service_name())?,
            OperationId::parse(producer.operation_name())?,
            parent,
        )?;
        store
            .seed_work(WorkSpec::new(
                sweep.clone(),
                scope,
                None,
                bindings.into_bytes(),
            )?)
            .await?;
        let tags = if let Some(tags) = producer.tags() {
            observations
                .iter()
                .map(|observation| tag_observation(observation.record(), observation.key(), tags))
                .collect::<Result<Vec<_>, _>>()?
        } else {
            Vec::new()
        };
        let lease = store
            .claim(&sweep, 1000, 10000)
            .await?
            .ok_or("seeded work absent")?;
        store
            .commit_page(PageCommit {
                lease,
                observations,
                edges: Vec::new(),
                unresolved: Vec::new(),
                tags,
                next_token: None,
                continuation_digest: None,
                children: Vec::new(),
                done: true,
                observed_at_ms: 1001,
            })
            .await?;
    }
    let reader = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename(&database)
                .read_only(true),
        )
        .await?;
    let resources =
        sqlx::query_as("SELECT kind,id,record FROM resource_facets WHERE kind=?1 ORDER BY id")
            .bind(plan.kind().as_str())
            .fetch_all(&reader)
            .await?;
    let tags = sqlx::query_as("SELECT tag_key,tag_value FROM tags ORDER BY tag_key")
        .fetch_all(&reader)
        .await?;
    reader.close().await;
    Ok((resources, tags))
}
