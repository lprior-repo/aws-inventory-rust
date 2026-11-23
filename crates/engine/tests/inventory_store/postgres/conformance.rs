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
