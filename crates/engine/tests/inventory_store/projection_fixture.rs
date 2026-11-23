use super::{postgres, support};
use inventory_engine::inventory::{DatabaseUrl, InventoryStore, PgStore, PgWriteConfig};
use support::TestResult;

#[tokio::test]
#[ignore = "creates inventory in the explicit empty AWS_INVENTORY_PROJECTION_DATABASE"]
async fn given_empty_native_database_when_fixture_is_prepared_then_live_tests_have_pending_work()
-> TestResult {
    let url = DatabaseUrl::parse(&std::env::var("AWS_INVENTORY_PROJECTION_DATABASE")?)?;
    let store = PgStore::create(&url, postgres::registry()?, PgWriteConfig::default()).await?;
    let observations = (0..64)
        .map(|id| support::observation(&format!("live-{id:03}"), "live-value"))
        .collect::<TestResult<Vec<_>>>()?;
    postgres::collect(&store, "live-fixture", observations, Vec::new()).await?;
    assert_eq!(store.projection_state().await?.pending, 64);
    store.close().await;
    Ok(())
}
