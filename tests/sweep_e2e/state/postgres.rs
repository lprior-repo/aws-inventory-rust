use super::*;
use inventory_engine::inventory::{
    DatabaseUrl, InventoryStore, PgStore, PgWriteConfig, ProjectionResource,
};
use sqlx::{PgPool, postgres::PgPoolOptions};

pub(super) async fn snapshot(url: &str) -> Result<InventorySnapshot> {
    let store = PgStore::open(&DatabaseUrl::parse(url)?, PgWriteConfig::default()).await?;
    let result = complete_snapshot(url, &store).await;
    store.close().await;
    result
}

async fn complete_snapshot(url: &str, store: &PgStore) -> Result<InventorySnapshot> {
    let rows = resource_rows(store).await?;
    let miss_counters = snapshot_data(url, rows.len()).await?;
    let page = store.projection_edges(None, 1000).await?;
    ensure!(
        page.next.is_none(),
        "E2E relationships exceed the bounded one-page snapshot"
    );
    Ok(InventorySnapshot {
        resources: rows
            .into_iter()
            .map(|row| (row.key, row.content_hash))
            .collect(),
        edges: page.edges,
        miss_counters,
    })
}

async fn resource_rows(store: &PgStore) -> Result<Vec<ProjectionResource>> {
    let rows = store.projection_resources(None, 1000).await?;
    let next = match rows.last() {
        Some(last) => store
            .projection_resources(Some(last.key.clone()), 1)
            .await?
            .into_iter()
            .next(),
        None => None,
    };
    ensure!(
        next.is_none(),
        "E2E resources exceed the bounded one-page snapshot"
    );
    Ok(rows)
}

async fn snapshot_data(url: &str, rows: usize) -> Result<MissCounters> {
    let pool = readonly_pool(url).await?;
    let result = verify_and_read(&pool, rows).await;
    pool.close().await;
    result
}

async fn verify_and_read(pool: &PgPool, rows: usize) -> Result<MissCounters> {
    verify_counts(pool, rows).await?;
    misses(pool).await
}

async fn verify_counts(pool: &PgPool, rows: usize) -> Result<()> {
    let (resources, accounts, regions, services): (i64, i64, i64, i64) = sqlx::query_as("SELECT count(*),count(DISTINCT account),count(DISTINCT region),count(DISTINCT split_part(kind,'::',2)) FROM inventory.resources").fetch_one(pool).await?;
    ensure!(
        resources == i64::try_from(rows)? && resources > 0,
        "SQL resource counts must equal persisted page rows"
    );
    ensure!(
        accounts == 1 && regions > 0 && services > 0,
        "SQL account, region, and service counts must reflect persisted rows"
    );
    Ok(())
}

pub(super) async fn edges(url: &str) -> Result<Vec<ProjectionEdgeHeader>> {
    let store = PgStore::open(&DatabaseUrl::parse(url)?, PgWriteConfig::default()).await?;
    let result = store.projection_edges(None, 1000).await;
    store.close().await;
    let page = result?;
    ensure!(
        page.next.is_none(),
        "E2E relationships exceed the bounded one-page snapshot"
    );
    Ok(page.edges)
}

async fn readonly_pool(url: &str) -> Result<PgPool> {
    Ok(PgPoolOptions::new()
        .max_connections(1)
        .after_connect(|connection, _| {
            Box::pin(async move {
                sqlx::query("SET default_transaction_read_only=on")
                    .execute(connection)
                    .await?;
                Ok(())
            })
        })
        .connect(url)
        .await?)
}

async fn misses(pool: &PgPool) -> Result<MissCounters> {
    let rows = sqlx::query_as::<_, (String, String, String, String, Vec<u8>, i64)>("SELECT account,region,kind,id,scope_key,missing_complete FROM inventory.resource_scopes ORDER BY account,region,kind,id,scope_key LIMIT 10001").fetch_all(pool).await?;
    ensure!(
        rows.len() <= 10000,
        "E2E scope snapshot exceeds its row bound"
    );
    Ok(rows)
}

pub(super) async fn active(url: &str) -> Result<ActiveCheckpoint> {
    let pool = readonly_pool(url).await?;
    let mut ids = sqlx::query_scalar::<_, String>(
        "SELECT DISTINCT sweep_id FROM inventory.work_queue WHERE state IN(0,1) LIMIT 2",
    )
    .fetch_all(&pool)
    .await?;
    ensure!(
        ids.len() == 1,
        "interrupted CLI must leave exactly one active sweep"
    );
    let sweep_id = ids.pop().context("active sweep identifier is missing")?;
    let result = read_active(&pool, sweep_id).await;
    pool.close().await;
    result
}

async fn read_active(pool: &PgPool, sweep_id: String) -> Result<ActiveCheckpoint> {
    let observed: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM (SELECT account,region,kind,id FROM inventory.resource_scopes WHERE observed_sweep=$1 GROUP BY account,region,kind,id) r").bind(&sweep_id).fetch_one(pool).await?;
    let pending: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM inventory.work_queue WHERE sweep_id=$1 AND state IN(0,1)",
    )
    .bind(&sweep_id)
    .fetch_one(pool)
    .await?;
    Ok(ActiveCheckpoint {
        sweep_id,
        resources_observed: u64::try_from(observed)?,
        work_pending: u64::try_from(pending)?,
    })
}

pub(super) async fn finished(url: &str, sweep: &str) -> Result<()> {
    let pool = readonly_pool(url).await?;
    let unfinished: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM inventory.work_queue WHERE sweep_id=$1 AND state<>2",
    )
    .bind(sweep)
    .fetch_one(&pool)
    .await?;
    let outcomes: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM inventory.scope_outcomes WHERE sweep_id=$1")
            .bind(sweep)
            .fetch_one(&pool)
            .await?;
    pool.close().await;
    ensure!(
        unfinished == 0 && outcomes > 0,
        "resume must complete the exact interrupted sweep ID"
    );
    Ok(())
}
