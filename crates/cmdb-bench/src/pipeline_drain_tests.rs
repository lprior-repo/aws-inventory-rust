use super::*;
use inventory_harness::postgres::{TestDatabase, admin_url};
use sqlx::PgPool;

#[tokio::test]
async fn given_one_rejected_commit_when_round_fails_then_other_inflight_commit_is_drained()
-> BenchResult {
    let directory = tempfile::tempdir()?;
    let database = TestDatabase::create(&admin_url()?).await?;
    let outcome = rejected_commit_state(directory.path(), database.url()).await;
    let cleanup = database.finish().await;
    outcome?;
    cleanup?;
    Ok(())
}

async fn rejected_commit_state(directory: &std::path::Path, url: &str) -> BenchResult {
    let bytes = crate::pipeline_test_catalog::bytes()?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let database = BenchDatabase::parse(url)?;
    let store = database
        .create_store(SchemaRegistry::new(&bytes, catalog.schemas().to_vec())?)
        .await?;
    let pool = PgPool::connect(url).await?;
    let outcome = install_rejection_and_run(directory, &database, &catalog, &store, &pool).await;
    pool.close().await;
    store.close().await;
    outcome
}

async fn install_rejection_and_run(
    directory: &std::path::Path,
    database: &BenchDatabase,
    catalog: &RuntimeCatalog<'_>,
    store: &PgStore,
    pool: &PgPool,
) -> BenchResult {
    sqlx::query(
        "CREATE FUNCTION inventory.reject_one_scope() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN IF NEW.account='000000000001' THEN RAISE EXCEPTION 'fixture-rejected-scope'; END IF; RETURN NEW; END $$",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE TRIGGER reject_one_scope BEFORE INSERT ON inventory.resources FOR EACH ROW EXECUTE FUNCTION inventory.reject_one_scope()",
    )
    .execute(pool)
    .await?;
    let workload = Workload {
        database: database.clone(),
        catalog: directory.join("unused.catalog"),
        pages: 1,
        rows: 3,
        concurrency: 2,
        graph: None,
    };
    let error = run_phase(&workload, catalog, store, Phase::Initial)
        .await
        .err()
        .ok_or("rejected PostgreSQL insert unexpectedly succeeded")?;
    assert!(
        error.to_string().contains("fixture-rejected-scope"),
        "{error}"
    );
    let accounts: Vec<(String, i64)> = sqlx::query_as(
        "SELECT account,COUNT(*) FROM inventory.resources GROUP BY account ORDER BY account",
    )
    .fetch_all(pool)
    .await?;
    assert_eq!(accounts, vec![("000000000002".into(), 3)]);
    let completed: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM inventory.work_queue WHERE state=2")
            .fetch_one(pool)
            .await?;
    assert_eq!(completed, 1);
    let tagged: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM inventory.tags WHERE account='000000000002'",
    )
    .fetch_one(pool)
    .await?;
    assert_eq!(tagged, 6);
    Ok(())
}
