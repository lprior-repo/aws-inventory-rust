//! The reason stored beside a scope outcome, on PostgreSQL.

use super::{DatabaseUrl, PgPool, PgStore, TestResult, claim, run, support, work};
use inventory_core::inventory::ScopeOutcome;
use inventory_engine::inventory::{MAX_OUTCOME_DETAIL_BYTES, StoreError};

type Stored = Vec<(String, Option<String>)>;

async fn stored(pool: &PgPool, sweep: &str) -> TestResult<Stored> {
    Ok(
        sqlx::query_as("SELECT outcome,detail FROM inventory.scope_outcomes WHERE sweep_id=$1")
            .bind(sweep)
            .fetch_all(pool)
            .await?,
    )
}

/// Records an earlier failed outcome, with a reason, for the one scope queued in `sweep`.
async fn earlier_reason(pool: &PgPool, sweep: &str) -> TestResult {
    sqlx::query("INSERT INTO inventory.scope_outcomes(sweep_id,scope_key,outcome,observed_at,detail) SELECT sweep_id,scope_key,'failed',90,'transport: earlier' FROM inventory.work_queue WHERE sweep_id=$1")
        .bind(sweep)
        .execute(pool)
        .await?;
    Ok(())
}

async fn check_reason_stored(store: &PgStore, pool: &PgPool, _url: &DatabaseUrl) -> TestResult {
    store.seed_works(&[work("reason", None)?]).await?;
    let lease = claim(store, "reason", 100).await?;

    store
        .finish_scope(lease, ScopeOutcome::Failed, 101, Some("denied: HTTP 403"))
        .await?;

    assert_eq!(
        stored(pool, "reason").await?,
        [("failed".to_owned(), Some("denied: HTTP 403".to_owned()))]
    );
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_a_failed_scope_when_finished_with_a_reason_then_the_reason_is_stored() -> TestResult
{
    run(check_reason_stored).await
}

async fn check_bound(store: &PgStore, pool: &PgPool, _url: &DatabaseUrl) -> TestResult {
    store.seed_works(&[work("bound", None)?]).await?;
    let lease = claim(store, "bound", 100).await?;
    let longest = "r".repeat(MAX_OUTCOME_DETAIL_BYTES);
    let over = format!("{longest}r");

    let refused = store
        .finish_scope(lease.clone(), ScopeOutcome::Failed, 101, Some(&over))
        .await;

    assert!(matches!(refused, Err(StoreError::Invalid(_))));
    assert!(stored(pool, "bound").await?.is_empty());
    store
        .finish_scope(lease, ScopeOutcome::Failed, 102, Some(&longest))
        .await?;
    assert_eq!(
        stored(pool, "bound").await?,
        [("failed".to_owned(), Some(longest))]
    );
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_a_reason_over_the_bound_when_finished_then_it_is_refused_and_the_bound_is_accepted()
-> TestResult {
    run(check_bound).await
}

async fn check_finish_clears(store: &PgStore, pool: &PgPool, _url: &DatabaseUrl) -> TestResult {
    store.seed_works(&[work("cleared", None)?]).await?;
    let lease = claim(store, "cleared", 100).await?;
    earlier_reason(pool, "cleared").await?;

    store
        .finish_scope(lease, ScopeOutcome::Partial, 101, None)
        .await?;

    assert_eq!(
        stored(pool, "cleared").await?,
        [("partial".to_owned(), None)]
    );
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_a_stored_reason_when_the_scope_finishes_without_one_then_the_reason_is_cleared()
-> TestResult {
    run(check_finish_clears).await
}

async fn check_commit_clears(store: &PgStore, pool: &PgPool, _url: &DatabaseUrl) -> TestResult {
    store.seed_works(&[work("completed", None)?]).await?;
    let lease = claim(store, "completed", 100).await?;
    earlier_reason(pool, "completed").await?;

    store
        .commit_page(support::page(lease, Vec::new(), true, None, 101))
        .await?;

    assert_eq!(
        stored(pool, "completed").await?,
        [("complete".to_owned(), None)]
    );
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_a_stored_reason_when_the_last_page_commits_then_the_reason_is_cleared() -> TestResult
{
    run(check_commit_clears).await
}
