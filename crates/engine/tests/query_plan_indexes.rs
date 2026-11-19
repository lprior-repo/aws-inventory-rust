#[path = "inventory_store/support.rs"]
mod support;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use support::*;

#[tokio::test]
async fn work_queue_has_ready_work_index() -> TestResult {
    let (directory, _store, _catalog) = fixture().await?;

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new().filename(directory.path().join("inventory.sqlite")),
        )
        .await?;

    let row: Option<(String,)> = sqlx::query_as(
        "SELECT name FROM sqlite_master WHERE type='index' AND name='work_queue_claim'",
    )
    .fetch_optional(&pool)
    .await?;
    pool.close().await;

    if row.is_none() {
        return Err("work_queue_claim index does not exist".into());
    }
    Ok(())
}

#[tokio::test]
async fn unresolved_refs_has_target_lookup_index() -> TestResult {
    let (directory, _store, _catalog) = fixture().await?;

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new().filename(directory.path().join("inventory.sqlite")),
        )
        .await?;

    let row: Option<(String,)> = sqlx::query_as(
        "SELECT name FROM sqlite_master WHERE type='index' AND name='unresolved_refs_target'",
    )
    .fetch_optional(&pool)
    .await?;
    pool.close().await;

    if row.is_none() {
        return Err("unresolved_refs_target index does not exist".into());
    }
    Ok(())
}
