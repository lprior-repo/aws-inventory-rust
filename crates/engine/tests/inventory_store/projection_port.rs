use super::support::{TestResult, fixture};
use inventory_engine::inventory::{
    InventoryStore, ProjectionDelivery, ProjectionIdentity, StoreError,
};

#[tokio::test]
async fn given_identity_when_port_persists_it_then_target_and_rebuild_state_survive() -> TestResult
{
    let (_directory, store, _) = fixture().await?;
    identity_contract(&store).await
}

pub(super) async fn identity_contract<S: InventoryStore>(store: &S) -> TestResult {
    assert_eq!(store.read_projection_identity().await?, None);
    let identity = ProjectionIdentity {
        endpoint: "redis://localhost:6379".into(),
        graph: "inventory".into(),
    };
    store.write_projection_identity(&identity).await?;
    assert_eq!(store.read_projection_identity().await?, Some(identity));
    store.mark_projection_rebuild_required().await?;
    assert!(store.projection_state().await?.rebuild_required);
    Ok(())
}

#[tokio::test]
async fn given_owned_delivery_when_port_reacquires_then_busy_and_generation_are_preserved()
-> TestResult {
    let (_directory, store, _) = fixture().await?;
    delivery_contract(&store).await
}

pub(super) async fn delivery_contract<S: InventoryStore>(store: &S) -> TestResult {
    let first = store.acquire_projection_delivery().await?;
    let generation = first.generation();
    assert!(generation > 0);
    assert!(matches!(
        store.acquire_projection_delivery().await,
        Err(StoreError::DeliveryBusy)
    ));
    first.release().await?;
    let second = store.acquire_projection_delivery().await?;
    assert!(second.generation() > generation);
    second.release().await?;
    Ok(())
}
