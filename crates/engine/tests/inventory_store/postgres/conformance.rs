use super::{PgStore, PgWriteConfig, TestDatabase, TestResult, admin_url, registry, support};
use inventory_engine::inventory::{DatabaseUrl, SweepId};
use std::num::NonZeroUsize;

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_native_backend_when_conformance_runs_then_every_store_contract_holds() -> TestResult
{
    super::super::conformance::run(async || fixture(PgWriteConfig::default()).await).await
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn forced_staging_copy_preserves_every_store_contract() -> TestResult {
    let config = PgWriteConfig::default().with_copy_min_rows(NonZeroUsize::MIN);
    let (database, store, _) = fixture(config).await?;
    let pool = sqlx::PgPool::connect(database.url()).await?;
    let sweep_id = SweepId::new("copy-test".to_owned())?;
    store
        .create_sweep_manifest(&sweep_id, "111111111111")
        .await?;
    store.seed_works(&[super::work("copy-test", None)?]).await?;
    let lease = super::claim(&store, "copy-test", 100).await?;
    let page = support::page(
        lease,
        vec![
            support::observation("one", "first")?,
            support::observation("two", "second")?,
        ],
        true,
        None,
        101,
    );
    let result = store.commit_page(page).await?;
    assert_eq!(result.stats.resources_inserted, 2);
    assert_eq!(
        staging_tables(&pool).await?,
        ["stg_resources", "stg_thing_table"]
    );
    store.close().await;
    pool.close().await;
    database.finish().await?;
    super::super::conformance::run(async || fixture(config).await).await
}

async fn fixture(config: PgWriteConfig) -> TestResult<(TestDatabase, PgStore, Vec<u8>)> {
    let database = TestDatabase::create(&admin_url()?).await?;
    let url = DatabaseUrl::parse(database.url())?;
    let store = PgStore::create(&url, registry()?, config).await?;
    for id in ["work-1", "live-lease", "takeover", "dummy"] {
        store
            .create_sweep_manifest(&SweepId::new(id.to_owned())?, "111111111111")
            .await?;
    }
    Ok((database, store, support::catalog_bytes()?))
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn a_page_at_the_threshold_creates_staging_tables_but_a_smaller_page_does_not() -> TestResult
{
    let threshold = NonZeroUsize::new(2).ok_or("threshold must be nonzero")?;
    let (database, store, _) =
        fixture(PgWriteConfig::default().with_copy_min_rows(threshold)).await?;
    let pool = sqlx::PgPool::connect(database.url()).await?;
    let result = check_stage_threshold(&store, &pool).await;
    store.close().await;
    pool.close().await;
    database.finish().await?;
    result
}

async fn check_stage_threshold(store: &PgStore, pool: &sqlx::PgPool) -> TestResult {
    store.seed_works(&[super::work("work-1", None)?]).await?;
    let first = support::page(
        super::claim(store, "work-1", 100).await?,
        vec![support::observation("one", "first")?],
        false,
        Some(b"next".to_vec()),
        101,
    );
    let (result, lease) = store.commit_page_keeping(first, 10_000).await?;
    assert_eq!(result.stats.resources_inserted, 1);
    assert!(staging_tables(pool).await?.is_empty());
    let second = support::page(
        lease.ok_or("first page did not keep its lease")?,
        vec![
            support::observation("two", "second")?,
            support::observation("three", "third")?,
        ],
        true,
        None,
        102,
    );
    let result = store.commit_page(second).await?;
    assert_eq!(result.stats.resources_inserted, 2);
    assert_eq!(
        staging_tables(pool).await?,
        ["stg_resources", "stg_thing_table"]
    );
    Ok(())
}

async fn staging_tables(pool: &sqlx::PgPool) -> TestResult<Vec<String>> {
    Ok(sqlx::query_scalar(
        "SELECT relname::text FROM pg_class WHERE relpersistence = 't' AND relname IN ('stg_resources','stg_thing_table') ORDER BY relname",
    )
    .fetch_all(pool)
    .await?)
}
impl support::OutcomeDetailStorageCapability for TestDatabase {
    async fn stored_outcomes(
        &self,
        sweep: &str,
    ) -> support::TestResult<Vec<(String, Option<String>)>> {
        let pool = sqlx::PgPool::connect(self.url()).await?;
        let rows = sqlx::query_as(
            "SELECT outcome,detail FROM inventory.scope_outcomes WHERE sweep_id=$1 ORDER BY scope_key",
        )
        .bind(sweep)
        .fetch_all(&pool)
        .await?;
        pool.close().await;
        Ok(rows)
    }

    async fn seed_prior_outcome_detail(&self, sweep: &str) -> support::TestResult {
        let pool = sqlx::PgPool::connect(self.url()).await?;
        sqlx::query("INSERT INTO inventory.scope_outcomes(sweep_id,scope_key,outcome,observed_at,detail) SELECT sweep_id,scope_key,'failed',90,'transport: earlier' FROM inventory.work_queue WHERE sweep_id=$1")
            .bind(sweep)
            .execute(&pool)
            .await?;
        pool.close().await;
        Ok(())
    }
}

impl support::UnobservedScopesCapability for PgStore {
    async fn unobserved_scopes(
        &self,
        sweep: &SweepId,
        now_ms: i64,
    ) -> Result<
        Vec<inventory_engine::inventory::UnobservedScope>,
        inventory_engine::inventory::StoreError,
    > {
        PgStore::unobserved_scopes(self, sweep, now_ms).await
    }
}
