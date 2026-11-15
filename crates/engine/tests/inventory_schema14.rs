use inventory_engine::inventory::{BinaryStore, StoreError};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

#[tokio::test]
async fn given_schema13_when_opened_then_rejects_without_migration()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let database = directory.path().join("inventory.sqlite");
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new()
                .filename(&database)
                .create_if_missing(true),
        )
        .await?;
    sqlx::query("PRAGMA user_version=13").execute(&pool).await?;
    pool.close().await;

    let result = BinaryStore::open(&database).await;

    assert!(matches!(
        result,
        Err(StoreError::SchemaVersion { found: 13 })
    ));
    Ok(())
}
