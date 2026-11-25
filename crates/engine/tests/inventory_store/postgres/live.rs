use super::{DatabaseUrl, PgPool, PgStore, TestResult, collect, run, support};
use inventory_engine::inventory::{InventoryStore, run_projection};
use redis::{Value, aio::MultiplexedConnection};
use std::time::{Duration, Instant};

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL and AWS_INVENTORY_PROJECTION_URL"]
async fn given_native_inventory_when_projected_and_rebuilt_then_live_graph_matches() -> TestResult {
    run(live).await
}

async fn live(store: &PgStore, pool: &PgPool, _: &DatabaseUrl) -> TestResult {
    let endpoint = std::env::var("AWS_INVENTORY_PROJECTION_URL")?;
    let graph = format!("pg_projection_{}", std::process::id());
    let mut connection = redis::Client::open(endpoint.as_str())?
        .get_multiplexed_async_connection()
        .await?;
    let result = tokio::time::timeout(
        Duration::from_secs(60),
        measure(store, pool, &endpoint, &graph, &mut connection),
    )
    .await;
    let cleanup = redis::cmd("GRAPH.DELETE")
        .arg(&graph)
        .query_async::<Value>(&mut connection)
        .await;
    result??;
    cleanup?;
    Ok(())
}

async fn measure(
    store: &PgStore,
    pool: &PgPool,
    endpoint: &str,
    graph: &str,
    connection: &mut MultiplexedConnection,
) -> TestResult {
    let observations = (0..64)
        .map(|id| support::observation(&format!("live-{id:03}"), "live-value"))
        .collect::<TestResult<Vec<_>>>()?;
    collect(store, "live-first", observations, Vec::new()).await?;
    let began = Instant::now();
    let backend = inventory_graph::falkor::FalkorDb::connect(endpoint, graph)?;
    let report = run_projection(store, &backend, 1000).await?;
    let elapsed = began.elapsed();
    assert_eq!(report.acknowledged, 64);
    assert_eq!(report.pending, 0);
    assert!(!report.offline);
    assert_eq!(graph_count(connection, graph).await?, 64);
    record_keys(elapsed);
    sqlx::query("INSERT INTO inventory.projection_outbox(resource_key,revision,deleted,full_rebuild) VALUES(decode('00','hex'),nextval('inventory.projection_revision_seq'),false,true)").execute(pool).await?;
    rebuild(store, endpoint, graph, connection).await
}

async fn rebuild(
    store: &PgStore,
    endpoint: &str,
    graph: &str,
    connection: &mut MultiplexedConnection,
) -> TestResult {
    let began = Instant::now();
    let backend = inventory_graph::falkor::FalkorDb::connect(endpoint, graph)?;
    let report = run_projection(store, &backend, 1000).await?;
    let elapsed = began.elapsed();
    assert_eq!(report.acknowledged, 1);
    assert_eq!(report.pending, 0);
    assert!(!report.offline);
    assert!(!report.rebuild_incomplete);
    assert_eq!(graph_count(connection, graph).await?, 64);
    assert!(!store.projection_state().await?.rebuild_required);
    eprintln!(
        "P15 native rebuild_resources=64 elapsed_us={}",
        elapsed.as_micros()
    );
    Ok(())
}

async fn graph_count(connection: &mut MultiplexedConnection, graph: &str) -> TestResult<i64> {
    let value = redis::cmd(inventory_graph::cypher::COMMAND)
        .arg(graph)
        .arg(inventory_graph::cypher::diagnostics::RESOURCE_NODE_COUNT)
        .query_async::<Value>(connection)
        .await?;
    let Value::Array(parts) = value else {
        return Err("missing graph response".into());
    };
    let Some(Value::Array(rows)) = parts.get(1) else {
        return Err("missing graph rows".into());
    };
    let Some(Value::Array(row)) = rows.first() else {
        return Err("missing graph row".into());
    };
    let Some(Value::Int(count)) = row.first() else {
        return Err("missing graph count".into());
    };
    Ok(*count)
}

fn record_keys(elapsed: Duration) {
    eprintln!(
        "P15 native keys=64 elapsed_us={} keys_per_second={:.2}",
        elapsed.as_micros(),
        64.0 / elapsed.as_secs_f64()
    );
}
