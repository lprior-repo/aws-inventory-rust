use super::*;
use inventory_engine::inventory::{
    BinaryStore, DatabaseUrl, InventoryStore, PgStore, PgWriteConfig, ProjectionReport,
    run_projection,
};

pub(super) async fn run(
    database: &Database,
    environment: &Environment,
    graph: &str,
    pending: u64,
) -> Result<()> {
    match database {
        Database::File(path) => {
            let store = BinaryStore::open(path).await?;
            project(&store, environment, graph, pending).await
        }
        Database::Postgres(database) => native(database.url(), environment, graph, pending).await,
    }
}

async fn native(url: &str, environment: &Environment, graph: &str, pending: u64) -> Result<()> {
    let store = PgStore::open(&DatabaseUrl::parse(url)?, PgWriteConfig::default()).await?;
    let result = project(&store, environment, graph, pending).await;
    store.close().await;
    result
}

async fn project<S: InventoryStore>(
    store: &S,
    environment: &Environment,
    graph: &str,
    pending: u64,
) -> Result<()> {
    let state = store.projection_state().await?;
    ensure!(
        pending > 0 && state.pending == pending,
        "CLI offline projection must match durable outbox state"
    );
    drain(store, environment, graph).await
}

async fn drain<S: InventoryStore>(store: &S, environment: &Environment, graph: &str) -> Result<()> {
    let backend = inventory_graph::falkor::FalkorDb::connect(&environment.graph_url, graph)?;
    let mut acknowledged = 0_u64;
    for _ in 0..100 {
        let report = run_projection(store, &backend, 100).await?;
        acknowledged = advance(acknowledged, &report)?;
        if report.pending == 0 && !report.rebuild_incomplete {
            return verify_drained(store, acknowledged).await;
        }
    }
    anyhow::bail!("projection did not drain within its 100-batch budget")
}

fn advance(acknowledged: u64, report: &ProjectionReport) -> Result<u64> {
    ensure!(
        !report.offline,
        "connected Falkor projection unexpectedly went offline"
    );
    acknowledged
        .checked_add(report.acknowledged)
        .context("projection acknowledgment count overflow")
}

async fn verify_drained<S: InventoryStore>(store: &S, acknowledged: u64) -> Result<()> {
    ensure!(
        acknowledged > 0,
        "reconnected projection acknowledged no durable work"
    );
    ensure!(
        store.projection_state().await?.pending == 0,
        "projection outbox is not durably drained"
    );
    Ok(())
}
