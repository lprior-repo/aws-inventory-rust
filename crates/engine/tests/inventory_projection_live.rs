use inventory_engine::inventory::BinaryStore;
use inventory_engine::inventory::run_projection;
use std::env;
use std::path::Path;

#[tokio::test]
#[ignore = "requires AWS_INVENTORY_PROJECTION_DATABASE and an explicit connected FalkorDB"]
async fn live_falkor_delivery_acknowledges_only_sent_revisions()
-> Result<(), Box<dyn std::error::Error>> {
    let database = env::var("AWS_INVENTORY_PROJECTION_DATABASE")?;
    let endpoint = env::var("AWS_INVENTORY_PROJECTION_URL")?;
    let graph = env::var("AWS_INVENTORY_PROJECTION_GRAPH")?;
    let store = BinaryStore::open(Path::new(&database)).await?;
    let before = store.projection_state().await?;
    if before.pending == 0 {
        return Err("live projection test requires pending outbox work".into());
    }
    let result = run_projection(&store, &endpoint, &graph, 32).await?;
    if result.offline || result.acknowledged == 0 || result.pending >= before.pending {
        return Err("live Falkor delivery did not acknowledge the sent outbox revisions".into());
    }
    Ok(())
}

#[tokio::test]
#[ignore = "requires AWS_INVENTORY_PROJECTION_DATABASE and pending projection work"]
async fn offline_falkor_keeps_every_pending_revision() -> Result<(), Box<dyn std::error::Error>> {
    let database = env::var("AWS_INVENTORY_PROJECTION_DATABASE")?;
    let store = BinaryStore::open(Path::new(&database)).await?;
    let before = store.projection_state().await?;
    let revisions = store.drain_projection(32).await?;
    if before.pending == 0 || revisions.is_empty() {
        return Err("offline projection test requires pending outbox work".into());
    }
    let result = run_projection(&store, "redis://127.0.0.1:1", "offline_contract", 32).await?;
    let after = store.drain_projection(32).await?;
    if !result.offline
        || result.acknowledged != 0
        || result.pending != before.pending
        || after != revisions
    {
        return Err("offline projection changed durable outbox revisions".into());
    }
    Ok(())
}
