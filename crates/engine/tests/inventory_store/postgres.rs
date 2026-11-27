#[path = "postgres/batch_claim.rs"]
mod batch_claim;
#[path = "postgres/completion.rs"]
mod completion;
#[path = "postgres/conformance.rs"]
mod conformance;
#[path = "postgres/live.rs"]
mod live;
#[path = "postgres/overflow.rs"]
mod overflow;
#[path = "postgres/projection.rs"]
mod projection;
#[path = "postgres/references.rs"]
mod references;

use super::support;
use inventory_core::inventory::{
    Column, Kind, KindSchema, OperationId, ResourceKey, ScalarType, ScopeKey, ServiceId, SqlName,
};
use inventory_core::record::{FieldId, Key, Record, Value};
use inventory_engine::inventory::{
    CommitResult, DatabaseUrl, PgStore, PgWriteConfig, SchemaRegistry, SweepId, WorkLease, WorkSpec,
};
use inventory_harness::postgres::{TestDatabase, admin_url};
use sqlx::PgPool;
use std::collections::BTreeMap;
use std::sync::Arc;
use support::TestResult;

pub(super) async fn run(
    check: impl AsyncFn(&PgStore, &PgPool, &DatabaseUrl) -> TestResult,
) -> TestResult {
    run_with(registry()?, check).await
}

pub(super) async fn run_with(
    registry: SchemaRegistry,
    check: impl AsyncFn(&PgStore, &PgPool, &DatabaseUrl) -> TestResult,
) -> TestResult {
    let database = TestDatabase::create(&admin_url()?).await?;
    let url = DatabaseUrl::parse(database.url())?;
    let pool = PgPool::connect(database.url()).await?;
    let store = PgStore::create(&url, registry, PgWriteConfig::default()).await?;
    let result = check(&store, &pool, &url).await;
    store.close().await;
    pool.close().await;
    database.finish().await?;
    result
}

fn text_kind(kind: &str, table: &str) -> TestResult<KindSchema> {
    Ok(KindSchema::new(
        Kind::parse(kind)?,
        SqlName::parse(table)?,
        vec![Column::new(vec![FieldId::new(1)?], ScalarType::Text)?],
    )?)
}

pub(super) fn registry() -> TestResult<SchemaRegistry> {
    Ok(SchemaRegistry::new(
        &support::catalog_bytes()?,
        vec![Arc::new(text_kind("AWS::Example::Thing", "thing_table")?)],
    )?)
}

pub(super) fn multi_kind_registry() -> TestResult<SchemaRegistry> {
    Ok(SchemaRegistry::new(
        &support::multi_kind_catalog_bytes()?,
        vec![
            Arc::new(text_kind("AWS::Example::Thing", "thing_table")?),
            Arc::new(text_kind("AWS::Example::Other", "other_table")?),
        ],
    )?)
}

pub(super) fn work(sweep: &str, parent: Option<ResourceKey>) -> TestResult<WorkSpec> {
    let root = support::scope("us-east-1")?;
    let scope = ScopeKey::new(
        root.account().clone(),
        root.region().clone(),
        ServiceId::parse("example")?,
        OperationId::parse("ListThings")?,
        parent,
    )?;
    let fields = BTreeMap::from([(
        Key::Field(FieldId::new(2)?),
        Value::Text("target-name".to_owned()),
    )]);
    Ok(WorkSpec::new(
        SweepId::new(sweep.to_owned())?,
        scope,
        None,
        Record::encode(&Value::Map(fields))?.into_bytes(),
    )?)
}

pub(super) async fn claim(store: &PgStore, sweep: &str, at: i64) -> TestResult<WorkLease> {
    Ok(store
        .claim(&SweepId::new(sweep.to_owned())?, at, 10_000)
        .await?
        .ok_or("missing PostgreSQL lease")?)
}

pub(super) async fn collect(
    store: &PgStore,
    sweep: &str,
    observations: Vec<inventory_core::inventory::Observation>,
    references: Vec<inventory_core::inventory::UnresolvedReference>,
) -> TestResult<CommitResult> {
    store.seed_works(&[work(sweep, None)?]).await?;
    let mut page = support::page(
        claim(store, sweep, 100).await?,
        observations,
        true,
        None,
        101,
    );
    page.unresolved = references;
    Ok(store.commit_page(page).await?)
}

pub(super) async fn single(store: &PgStore, sweep: &str, id: &str, value: &str) -> TestResult {
    collect(
        store,
        sweep,
        vec![support::observation(id, value)?],
        Vec::new(),
    )
    .await?;
    Ok(())
}
