use super::{DatabaseUrl, PgPool, PgStore, PgWriteConfig, TestResult, run, single, support};
use inventory_engine::inventory::{
    InventoryStore, ProjectionDelivery, ProjectionSnapshotRead, StoreError,
};

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_native_identity_when_persisted_then_projection_port_contract_holds() -> TestResult {
    run(identity).await
}

async fn identity(store: &PgStore, _: &PgPool, _: &DatabaseUrl) -> TestResult {
    super::super::projection_port::identity_contract(store).await?;
    super::super::projection_port::delivery_contract(store).await
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_snapshot_when_commit_precedes_first_read_then_original_view_is_preserved()
-> TestResult {
    run(snapshot).await
}

async fn snapshot(store: &PgStore, _: &PgPool, _: &DatabaseUrl) -> TestResult {
    single(store, "snapshot-first", "thing-1", "before").await?;
    let mut snapshot = store.begin_projection_snapshot().await?;
    single(store, "snapshot-second", "thing-2", "after").await?;
    assert!(
        snapshot
            .resource(&support::resource("thing-2")?)
            .await?
            .is_none()
    );
    assert_eq!(snapshot.resources(None, 1000).await?.len(), 1);
    assert_eq!(store.projection_resources(None, 1000).await?.len(), 2);
    snapshot.finish().await?;
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_owned_delivery_when_other_handle_writes_then_writers_progress_and_drop_releases()
-> TestResult {
    run(ownership).await
}

async fn ownership(store: &PgStore, _: &PgPool, url: &DatabaseUrl) -> TestResult {
    let other = PgStore::open(url, PgWriteConfig::default()).await?;
    let owned = other.acquire_projection_delivery().await?;
    let generation = owned.generation();
    assert!(matches!(
        store.acquire_projection_delivery().await,
        Err(StoreError::DeliveryBusy)
    ));
    tokio::time::timeout(
        std::time::Duration::from_secs(10),
        single(store, "owner-writer", "thing-1", "live"),
    )
    .await??;
    drop(owned);
    other.close().await;
    let replacement = store.acquire_projection_delivery().await?;
    assert!(replacement.generation() > generation);
    replacement.release().await?;
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_native_pages_when_limits_are_invalid_then_bounded_reads_fail() -> TestResult {
    run(bounds).await
}

async fn bounds(store: &PgStore, _: &PgPool, _: &DatabaseUrl) -> TestResult {
    assert!(store.drain_projection(0).await.is_err());
    assert!(store.projection_resources(None, 0).await.is_err());
    assert!(store.projection_resources(None, 1001).await.is_err());
    assert!(store.projection_edges(None, 1001).await.is_err());
    let mut snapshot = store.begin_projection_snapshot().await?;
    assert!(
        snapshot
            .tags_for(&support::resource("thing-1")?, None, 0)
            .await
            .is_err()
    );
    snapshot.finish().await?;
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_corrupt_identity_when_read_then_oversize_values_fail_without_loading_them()
-> TestResult {
    run(corrupt_identity).await
}

async fn corrupt_identity(store: &PgStore, pool: &PgPool, _: &DatabaseUrl) -> TestResult {
    sqlx::query("INSERT INTO inventory.metadata(key,value) VALUES('projection_identity',decode(repeat('61',4226),'hex'))")
        .execute(pool).await?;
    assert!(matches!(
        store.read_projection_identity().await,
        Err(StoreError::Invalid(_))
    ));
    Ok(())
}
