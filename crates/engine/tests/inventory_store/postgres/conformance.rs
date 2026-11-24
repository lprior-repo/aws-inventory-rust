use super::{PgStore, PgWriteConfig, TestDatabase, TestResult, admin_url, registry, support};
use inventory_engine::inventory::{DatabaseUrl, SweepId};

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_native_backend_when_conformance_runs_then_every_store_contract_holds() -> TestResult
{
    super::super::conformance::run(fixture).await
}

async fn fixture() -> TestResult<(TestDatabase, PgStore, Vec<u8>)> {
    let database = TestDatabase::create(&admin_url()?).await?;
    let url = DatabaseUrl::parse(database.url())?;
    let store = PgStore::create(&url, registry()?, PgWriteConfig::default()).await?;
    for id in ["work-1", "live-lease", "takeover", "dummy"] {
        store
            .create_sweep_manifest(&SweepId::new(id.to_owned())?, "111111111111")
            .await?;
    }
    Ok((database, store, support::catalog_bytes()?))
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
