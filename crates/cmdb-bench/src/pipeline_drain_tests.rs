use super::*;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

#[tokio::test]
async fn given_one_rejected_commit_when_round_fails_then_other_inflight_commit_is_drained() -> BenchResult {
    // Given two actual SQLite page commits and one transactionally rejected scope.
    let directory = tempfile::tempdir()?;
    let bytes = crate::pipeline_test_catalog::bytes()?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let path = directory.path().join("inventory.sqlite");
    let store = BinaryStore::create(&path, SchemaRegistry::new(&bytes, catalog.schemas().to_vec())?).await?;
    let pool = SqlitePoolOptions::new().max_connections(1).connect_with(SqliteConnectOptions::new().filename(&path)).await?;
    sqlx::query("CREATE TRIGGER reject_one_scope BEFORE INSERT ON resources WHEN NEW.account='000000000001' BEGIN SELECT RAISE(FAIL,'fixture-rejected-scope'); END").execute(&pool).await?;
    let workload = Workload { path, catalog: directory.path().join("unused.catalog"), pages: 1, rows: 3, concurrency: 2, graph: None };
    // When the round encounters its first error, it must not cancel sibling work.
    let error = run_phase(&workload, &catalog, &store, Phase::Initial).await.err().ok_or("rejected SQLite insert unexpectedly succeeded")?;
    assert!(error.to_string().contains("fixture-rejected-scope"), "{error}");
    // Then the rejected scope rolled back and the other scope fully committed.
    let accounts: Vec<(String, i64)> = sqlx::query_as("SELECT account,COUNT(*) FROM resources GROUP BY account ORDER BY account").fetch_all(&pool).await?;
    assert_eq!(accounts, vec![("000000000002".into(), 3)]);
    let completed: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM work_queue WHERE state=2").fetch_one(&pool).await?;
    assert_eq!(completed, 1);
    let tagged: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM tags WHERE account='000000000002'").fetch_one(&pool).await?;
    assert_eq!(tagged, 6);
    pool.close().await;
    Ok(())
}
