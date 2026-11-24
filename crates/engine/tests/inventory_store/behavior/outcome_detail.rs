//! The reason stored beside a scope outcome.

use super::*;
use inventory_engine::inventory::{MAX_OUTCOME_DETAIL_BYTES, WorkLease};

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

/// Records an earlier failed outcome, with a reason, for the one scope queued in `sweep`.
async fn earlier_reason(pool: &sqlx::SqlitePool, sweep: &str) -> TestResult {
    sqlx::query("INSERT INTO scope_outcomes(sweep_id,scope_key,outcome,observed_at,detail) SELECT sweep_id,scope_key,'failed',90,'transport: earlier' FROM work_queue WHERE sweep_id=?1")
        .bind(sweep)
        .execute(pool)
        .await?;
    Ok(())
}

#[tokio::test]
async fn given_a_failed_scope_when_finished_with_a_reason_then_the_reason_is_stored() -> TestResult
{
    let (directory, store, _) = fixture().await?;
    let lease = lease(&store, "reason").await?;

    store
        .finish_scope(
            lease,
            ScopeOutcome::Failed,
            101,
            Some("decode: ListThingsOutput: unexpected member"),
        )
        .await?;

    let pool = reader(&directory).await?;
    assert_eq!(
        stored(&pool, "reason").await?,
        [(
            "failed".to_owned(),
            Some("decode: ListThingsOutput: unexpected member".to_owned())
        )]
    );
    pool.close().await;
    Ok(())
}

#[tokio::test]
async fn given_a_reason_over_the_bound_when_finished_then_it_is_refused_and_the_bound_is_accepted()
-> TestResult {
    let (directory, store, _) = fixture().await?;
    let lease = lease(&store, "bound").await?;
    let longest = "r".repeat(MAX_OUTCOME_DETAIL_BYTES);
    let over = format!("{longest}r");

    let refused = store
        .finish_scope(lease.clone(), ScopeOutcome::Failed, 101, Some(&over))
        .await;

    assert!(matches!(refused, Err(StoreError::Invalid(_))));
    let pool = reader(&directory).await?;
    assert!(stored(&pool, "bound").await?.is_empty());
    store
        .finish_scope(lease, ScopeOutcome::Failed, 102, Some(&longest))
        .await?;
    assert_eq!(
        stored(&pool, "bound").await?,
        [("failed".to_owned(), Some(longest))]
    );
    pool.close().await;
    Ok(())
}

#[tokio::test]
async fn given_a_stored_reason_when_the_scope_finishes_without_one_then_the_reason_is_cleared()
-> TestResult {
    let (directory, store, _) = fixture().await?;
    let lease = lease(&store, "cleared").await?;
    let pool = reader(&directory).await?;
    earlier_reason(&pool, "cleared").await?;

    store
        .finish_scope(lease, ScopeOutcome::Partial, 101, None)
        .await?;

    assert_eq!(
        stored(&pool, "cleared").await?,
        [("partial".to_owned(), None)]
    );
    pool.close().await;
    Ok(())
}

#[tokio::test]
async fn given_a_stored_reason_when_the_last_page_commits_then_the_reason_is_cleared() -> TestResult
{
    let (directory, store, _) = fixture().await?;
    let lease = lease(&store, "completed").await?;
    let pool = reader(&directory).await?;
    earlier_reason(&pool, "completed").await?;

    store
        .commit_page(page(lease, Vec::new(), true, None, 101))
        .await?;

    assert_eq!(
        stored(&pool, "completed").await?,
        [("complete".to_owned(), None)]
    );
    pool.close().await;
    Ok(())
}

#[tokio::test]
async fn given_a_database_made_before_reasons_when_opened_then_a_reason_can_be_stored() -> TestResult
{
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
