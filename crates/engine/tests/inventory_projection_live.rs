#[path = "inventory_projection_live/degree.rs"]
mod degree;
use inventory_engine::inventory::{
    BinaryStore, DatabaseUrl, InventoryStore, PgStore, PgWriteConfig, run_projection,
};
use std::env;
use std::path::Path;

static LIVE_DATABASE: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

#[tokio::test]
#[ignore = "requires AWS_INVENTORY_PROJECTION_DATABASE and an explicit connected FalkorDB"]
async fn live_falkor_delivery_acknowledges_only_sent_revisions()
-> Result<(), Box<dyn std::error::Error>> {
    let _database_owner = LIVE_DATABASE.lock().await;
    let database = env::var("AWS_INVENTORY_PROJECTION_DATABASE")?;
    let endpoint = env::var("AWS_INVENTORY_PROJECTION_URL")?;
    let graph = env::var("AWS_INVENTORY_PROJECTION_GRAPH")?;
    if database.starts_with("postgres://") || database.starts_with("postgresql://") {
        let store =
            PgStore::open(&DatabaseUrl::parse(&database)?, PgWriteConfig::default()).await?;
        let result = connected(&store, &endpoint, &graph).await;
        store.close().await;
        result
    } else {
        connected(
            &BinaryStore::open(Path::new(&database)).await?,
            &endpoint,
            &graph,
        )
        .await
    }
}

async fn connected<S: InventoryStore>(
    store: &S,
    endpoint: &str,
    graph: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let before = store.projection_state().await?;
    if before.pending == 0 {
        return Err("live projection test requires pending outbox work".into());
    }
    let backend = inventory_graph::falkor::FalkorDb::connect(endpoint, graph)?;
    let result = run_projection(store, &backend, 32).await?;
    if result.offline || result.acknowledged == 0 || result.pending >= before.pending {
        return Err("live Falkor delivery did not acknowledge the sent outbox revisions".into());
    }
    Ok(())
}

#[tokio::test]
#[ignore = "requires AWS_INVENTORY_PROJECTION_DATABASE and pending projection work"]
async fn offline_falkor_keeps_every_pending_revision() -> Result<(), Box<dyn std::error::Error>> {
    let _database_owner = LIVE_DATABASE.lock().await;
    let database = env::var("AWS_INVENTORY_PROJECTION_DATABASE")?;
    if database.starts_with("postgres://") || database.starts_with("postgresql://") {
        let store =
            PgStore::open(&DatabaseUrl::parse(&database)?, PgWriteConfig::default()).await?;
        let result = offline(&store).await;
        store.close().await;
        result
    } else {
        offline(&BinaryStore::open(Path::new(&database)).await?).await
    }
}

async fn offline<S: InventoryStore>(store: &S) -> Result<(), Box<dyn std::error::Error>> {
    let before = store.projection_state().await?;
    let revisions = store.drain_projection(32).await?;
    if before.pending == 0 || revisions.is_empty() {
        return Err("offline projection test requires pending outbox work".into());
    }
    let backend =
        inventory_graph::falkor::FalkorDb::connect("redis://127.0.0.1:1", "offline_contract")?;
    let result = run_projection(store, &backend, 32).await?;
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
