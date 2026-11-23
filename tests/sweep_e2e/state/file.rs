use super::*;
use cmdb_view::{Database as FileDatabase, PageRequest, ResourceFilter};
use inventory_engine::inventory::BinaryStore;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

pub(super) async fn snapshot(path: &Path) -> Result<InventorySnapshot> {
    Ok(InventorySnapshot {
        resources: resources(path).await?,
        edges: edges(path).await?,
        miss_counters: misses(path).await?,
    })
}

async fn resources(path: &Path) -> Result<Vec<(inventory_core::inventory::ResourceKey, [u8; 32])>> {
    let database = FileDatabase::open(path).await?;
    let counts = database.counts().await?;
    let (total, rows) = resource_page(&database).await?;
    verify_counts(
        (
            counts.resources,
            counts.accounts,
            counts.regions,
            counts.services,
        ),
        total,
        rows.len(),
    )?;
    Ok(rows
        .into_iter()
        .map(|row| (row.key, *row.content_hash.as_bytes()))
        .collect())
}

async fn resource_page(database: &FileDatabase) -> Result<(u64, Vec<cmdb_view::StoredResource>)> {
    let page = database
        .page(
            &ResourceFilter::default(),
            PageRequest {
                limit: 1000,
                after: None,
            },
        )
        .await?;
    ensure!(
        page.next.is_none(),
        "E2E inventory exceeds the bounded one-page snapshot"
    );
    Ok((page.total_matching, page.resources))
}

fn verify_counts(counts: (u64, u64, u64, u64), total: u64, rows: usize) -> Result<()> {
    let (resources, accounts, regions, services) = counts;
    ensure!(
        total == resources && resources == u64::try_from(rows)? && resources > 0,
        "SQL resource counts must equal persisted page rows"
    );
    ensure!(
        accounts == 1 && regions > 0 && services > 0,
        "SQL account, region, and service counts must reflect persisted rows"
    );
    Ok(())
}

pub(super) async fn edges(path: &Path) -> Result<Vec<ProjectionEdgeHeader>> {
    let store = BinaryStore::open(path).await?;
    let page = store.projection_edges(None, 1000).await?;
    ensure!(
        page.next.is_none(),
        "E2E relationships exceed the bounded one-page snapshot"
    );
    Ok(page.edges)
}

async fn misses(path: &Path) -> Result<MissCounters> {
    let pool = readonly_pool(path).await?;
    let rows = sqlx::query_as::<_, (String, String, String, String, Vec<u8>, i64)>("SELECT account,region,kind,id,scope_key,missing_complete FROM resource_scopes ORDER BY account,region,kind,id,scope_key LIMIT 10001").fetch_all(&pool).await?;
    pool.close().await;
    ensure!(
        rows.len() <= 10000,
        "E2E scope snapshot exceeds its row bound"
    );
    Ok(rows)
}

async fn readonly_pool(path: &Path) -> Result<sqlx::SqlitePool> {
    let options = SqliteConnectOptions::new()
        .filename(path)
        .read_only(true)
        .pragma("query_only", "ON");
    Ok(SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await?)
}

pub(super) async fn active(path: &Path) -> Result<ActiveCheckpoint> {
    let pool = readonly_pool(path).await?;
    let mut ids = sqlx::query_scalar::<_, String>(
        "SELECT DISTINCT sweep_id FROM work_queue WHERE state IN(0,1) LIMIT 2",
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

async fn read_active(pool: &sqlx::SqlitePool, sweep_id: String) -> Result<ActiveCheckpoint> {
    let observed: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM (SELECT account,region,kind,id FROM resource_scopes WHERE observed_sweep=?1 GROUP BY account,region,kind,id)").bind(&sweep_id).fetch_one(pool).await?;
    let pending: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM work_queue WHERE sweep_id=?1 AND state IN(0,1)")
            .bind(&sweep_id)
            .fetch_one(pool)
            .await?;
    Ok(ActiveCheckpoint {
        sweep_id,
        resources_observed: u64::try_from(observed)?,
        work_pending: u64::try_from(pending)?,
    })
}

pub(super) async fn finished(path: &Path, sweep: &str) -> Result<()> {
    let pool = readonly_pool(path).await?;
    let unfinished: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM work_queue WHERE sweep_id=?1 AND state<>2")
            .bind(sweep)
            .fetch_one(&pool)
            .await?;
    let outcomes: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM scope_outcomes WHERE sweep_id=?1")
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
