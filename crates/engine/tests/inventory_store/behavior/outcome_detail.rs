use super::*;
use inventory_engine::inventory::WorkLease;

type Stored = Vec<(String, Option<String>)>;

async fn reader(directory: &tempfile::TempDir) -> TestResult<sqlx::SqlitePool> {
    Ok(SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new().filename(directory.path().join("inventory.sqlite")),
        )
        .await?)
}

async fn stored(pool: &sqlx::SqlitePool, sweep: &str) -> TestResult<Stored> {
    Ok(
        sqlx::query_as("SELECT outcome,detail FROM scope_outcomes WHERE sweep_id=?1")
            .bind(sweep)
            .fetch_all(pool)
            .await?,
    )
}

async fn lease(store: &BinaryStore, sweep: &str) -> TestResult<WorkLease> {
    store.seed_work(work(sweep, "us-east-1")?).await?;
    Ok(store
        .claim(&SweepId::new(sweep.to_owned())?, 100, 10_000)
        .await?
        .ok_or("missing lease")?)
}

#[tokio::test]
async fn given_sqlite_schema_without_detail_when_opened_then_reason_can_be_stored() -> TestResult {
    let (directory, store, _) = fixture().await?;
    drop(store);
    let pool = reader(&directory).await?;
    sqlx::query("ALTER TABLE scope_outcomes DROP COLUMN detail")
        .execute(&pool)
        .await?;

    let store = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
    let lease = lease(&store, "upgraded").await?;
    store
        .finish_scope(lease, ScopeOutcome::Failed, 101, Some("denied: HTTP 403"))
        .await?;

    assert_eq!(
        stored(&pool, "upgraded").await?,
        [("failed".to_owned(), Some("denied: HTTP 403".to_owned()))]
    );
    pool.close().await;
    Ok(())
}
