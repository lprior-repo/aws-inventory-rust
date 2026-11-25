use super::*;
use inventory_engine::inventory::StoreError;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

#[tokio::test]
async fn given_one_rejected_commit_when_round_fails_then_other_inflight_commit_is_drained()
-> BenchResult {
    let directory = tempfile::tempdir()?;
    let bytes = crate::pipeline_test_catalog::bytes()?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let path = directory.path().join("inventory.sqlite");
    let store = BinaryStore::create(
        &path,
        SchemaRegistry::new(&bytes, catalog.schemas().to_vec())?,
    )
    .await?;
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(SqliteConnectOptions::new().filename(&path))
        .await?;
    sqlx::query("CREATE TRIGGER reject_one_scope BEFORE INSERT ON resources WHEN NEW.account='000000000001' BEGIN SELECT RAISE(FAIL,'fixture-rejected-scope'); END").execute(&pool).await?;
    let workload = Workload {
        path,
        catalog: directory.path().join("unused.catalog"),
        pages: 1,
        rows: 3,
        concurrency: 2,
        graph: None,
    };
    let error = run_phase(&workload, &catalog, &store, Phase::Initial)
        .await
        .err()
        .ok_or("rejected SQLite insert unexpectedly succeeded")?;
    assert!(
        error.to_string().contains("fixture-rejected-scope"),
        "{error}"
    );
    let accounts: Vec<(String, i64)> =
        sqlx::query_as("SELECT account,COUNT(*) FROM resources GROUP BY account ORDER BY account")
            .fetch_all(&pool)
            .await?;
    assert_eq!(accounts, vec![("000000000002".into(), 3)]);
    let completed: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM work_queue WHERE state=2")
        .fetch_one(&pool)
        .await?;
    assert_eq!(completed, 1);
    let tagged: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM tags WHERE account='000000000002'")
        .fetch_one(&pool)
        .await?;
    assert_eq!(tagged, 6);
    pool.close().await;
    Ok(())
}

async fn stored_catalog(pool: &sqlx::SqlitePool) -> BenchResult<Vec<(String, Vec<u8>)>> {
    Ok(sqlx::query_as("SELECT key,value FROM metadata WHERE key='reviewed_catalog' OR key GLOB 'reviewed_catalog.[0-9][0-9]' ORDER BY key")
        .fetch_all(pool).await?)
}

#[tokio::test]
async fn given_catalog_above_sqlite_value_limit_when_persisted_then_chunks_roundtrip_and_stale_rows_are_replaced()
-> BenchResult {
    const CHUNK: usize = 4 * 1024 * 1024;
    let directory = tempfile::tempdir()?;
    let bytes = crate::pipeline_test_catalog::bytes()?;
    assert!(
        bytes.len() > 2 * CHUNK,
        "fixture catalog must exceed the SQLite value limit"
    );
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let path = directory.path().join("inventory.sqlite");
    drop(
        BinaryStore::create(
            &path,
            SchemaRegistry::new(&bytes, catalog.schemas().to_vec())?,
        )
        .await?,
    );
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(SqliteConnectOptions::new().filename(&path))
        .await?;
    let rows = stored_catalog(&pool).await?;
    assert_eq!(rows.len(), bytes.len().div_ceil(CHUNK));
    assert_eq!(
        rows.first().map(|(key, _)| key.as_str()),
        Some("reviewed_catalog")
    );
    assert!(rows.iter().all(|(_, chunk)| chunk.len() <= CHUNK));
    assert_eq!(
        rows.iter()
            .flat_map(|(_, chunk)| chunk.iter().copied())
            .collect::<Vec<u8>>(),
        bytes
    );
    drop(BinaryStore::open(&path).await?);
    sqlx::query("UPDATE metadata SET value=x'00' WHERE key='reviewed_catalog.01'")
        .execute(&pool)
        .await?;
    let corrupt = BinaryStore::open(&path)
        .await
        .err()
        .ok_or("corrupt continuation row unexpectedly opened")?;
    assert!(
        matches!(
            corrupt,
            StoreError::Catalog(_) | StoreError::RuntimeCatalog(_) | StoreError::CatalogMismatch
        ),
        "{corrupt}"
    );
    sqlx::query("INSERT INTO metadata(key,value) VALUES('reviewed_catalog.99',x'00')")
        .execute(&pool)
        .await?;
    sqlx::query("DELETE FROM metadata WHERE key='reviewed_catalog'")
        .execute(&pool)
        .await?;
    drop(BinaryStore::open(&path).await?);
    drop(BinaryStore::open_with_catalog(&path, &bytes).await?);
    assert_eq!(stored_catalog(&pool).await?, rows);
    drop(BinaryStore::open_with_catalog(&path, &bytes).await?);
    assert_eq!(stored_catalog(&pool).await?, rows);
    drop(BinaryStore::open(&path).await?);
    pool.close().await;
    Ok(())
}
