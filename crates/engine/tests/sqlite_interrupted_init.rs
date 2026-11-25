//! R07: a SQLite file left behind by an interrupted initialization must be
//! recoverable through the same open/create selection the CLI uses.
mod postgres_support;

use inventory_engine::inventory::{BinaryStore, SchemaRegistry, StoreError, StoreState};
use postgres_support::{TestResult, registry};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use std::path::Path;

/// Mirrors the CLI: open initialized databases, create everything else.
async fn open_or_create(
    path: &Path,
    registry: SchemaRegistry,
    catalog: &[u8],
) -> Result<BinaryStore, StoreError> {
    match BinaryStore::probe(path).await? {
        StoreState::Initialized => BinaryStore::open_with_catalog(path, catalog).await,
        StoreState::Absent | StoreState::Uninitialized => BinaryStore::create(path, registry).await,
    }
}

/// Leaves exactly what `create` leaves when initialization never commits: a
/// non-empty WAL-mode file whose `user_version` is still 0.
async fn leave_interrupted(path: &Path) -> TestResult {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new()
                .filename(path)
                .create_if_missing(true)
                .journal_mode(SqliteJournalMode::Wal),
        )
        .await?;
    sqlx::query("PRAGMA user_version").fetch_one(&pool).await?;
    pool.close().await;
    Ok(())
}

#[tokio::test]
async fn given_interrupted_init_when_cli_selection_runs_then_store_is_created_and_reopens()
-> TestResult {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("inventory.sqlite");
    leave_interrupted(&path).await?;
    assert!(std::fs::metadata(&path)?.len() > 0);
    assert!(matches!(
        BinaryStore::open(&path).await,
        Err(StoreError::SchemaVersion { found: 0 })
    ));
    assert_eq!(BinaryStore::probe(&path).await?, StoreState::Uninitialized);

    let (registry, catalog) = registry(1, None)?;
    open_or_create(&path, registry, &catalog).await?;

    assert_eq!(BinaryStore::probe(&path).await?, StoreState::Initialized);
    let (registry, catalog) = self::registry(1, None)?;
    open_or_create(&path, registry, &catalog).await?;
    Ok(())
}

#[tokio::test]
async fn given_absent_or_empty_file_when_probed_then_absent() -> TestResult {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("inventory.sqlite");
    assert_eq!(BinaryStore::probe(&path).await?, StoreState::Absent);
    std::fs::write(&path, b"")?;
    assert_eq!(BinaryStore::probe(&path).await?, StoreState::Absent);
    Ok(())
}

#[tokio::test]
async fn given_foreign_version_zero_database_with_tables_when_probed_then_refused() -> TestResult {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("foreign.sqlite");
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new()
                .filename(&path)
                .create_if_missing(true),
        )
        .await?;
    sqlx::query("CREATE TABLE unrelated(x)")
        .execute(&pool)
        .await?;
    pool.close().await;
    assert!(matches!(
        BinaryStore::probe(&path).await,
        Err(StoreError::SchemaVersion { found: 0 })
    ));
    Ok(())
}
