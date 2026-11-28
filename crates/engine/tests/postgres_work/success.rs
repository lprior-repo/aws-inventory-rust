use super::{claim, run, work};
use crate::postgres_support::TestResult;
use inventory_core::inventory::ScopeOutcome;
use inventory_engine::inventory::{DatabaseUrl, PgStore, SweepId};
use sqlx::PgPool;

const ACCOUNT: &str = "123456789012";

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn postgres_work_given_all_scopes_complete_when_drained_then_freshness_is_recorded()
-> TestResult {
    run(complete_sweep).await
}

async fn complete_sweep(store: &PgStore, pool: &PgPool, _: &DatabaseUrl) -> TestResult {
    manifest(store, "s", &["us-east-1"]).await?;
    let lease = claim(store, "s", 100).await?;
    store
        .finish_scope(lease, ScopeOutcome::Complete, 101, None)
        .await?;
    assert_eq!(freshness(pool, "s").await?, (1, 101));
    store.seed_works(&[work("s", "us-west-2")?]).await?;
    let lease = claim(store, "s", 500).await?;
    store
        .finish_scope(lease, ScopeOutcome::Complete, 501, None)
        .await?;
    assert_eq!(freshness(pool, "s").await?, (1, 101));
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn postgres_work_given_reverse_commit_order_when_drained_then_latest_observation_wins()
-> TestResult {
    run(reverse_order).await
}

async fn reverse_order(store: &PgStore, pool: &PgPool, _: &DatabaseUrl) -> TestResult {
    manifest(store, "s", &["us-east-1", "us-west-2"]).await?;
    let first = claim(store, "s", 100).await?;
    let second = claim(store, "s", 100).await?;
    store
        .finish_scope(first, ScopeOutcome::Complete, 110, None)
        .await?;
    store
        .finish_scope(second, ScopeOutcome::Complete, 105, None)
        .await?;
    assert_eq!(freshness(pool, "s").await?, (1, 110));
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn postgres_work_given_complete_and_not_applicable_scopes_when_drained_then_freshness_is_recorded()
-> TestResult {
    run(not_applicable_sweep).await
}

async fn not_applicable_sweep(store: &PgStore, pool: &PgPool, _: &DatabaseUrl) -> TestResult {
    manifest(store, "s", &["us-east-1", "us-west-2"]).await?;
    let complete = claim(store, "s", 100).await?;
    let not_applicable = claim(store, "s", 100).await?;
    store
        .finish_scope(complete, ScopeOutcome::Complete, 101, None)
        .await?;
    store
        .finish_scope(not_applicable, ScopeOutcome::NotApplicable, 102, None)
        .await?;
    assert_eq!(freshness(pool, "s").await?, (1, 102));
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn postgres_work_given_partial_scope_when_drained_then_freshness_is_not_recorded()
-> TestResult {
    run(partial_sweep).await
}

async fn partial_sweep(store: &PgStore, pool: &PgPool, _: &DatabaseUrl) -> TestResult {
    manifest(store, "s", &["us-east-1"]).await?;
    let lease = claim(store, "s", 100).await?;
    store
        .finish_scope(lease, ScopeOutcome::Partial, 101, None)
        .await?;
    assert_eq!(freshness(pool, "s").await?, (0, 0));
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn postgres_work_given_inflight_and_failed_scopes_when_drained_then_not_fresh() -> TestResult {
    run(mixed_sweep).await
}

async fn mixed_sweep(store: &PgStore, pool: &PgPool, _: &DatabaseUrl) -> TestResult {
    manifest(store, "s", &["us-east-1", "us-west-2"]).await?;
    let first = claim(store, "s", 100).await?;
    let second = claim(store, "s", 100).await?;
    store
        .finish_scope(first, ScopeOutcome::Complete, 101, None)
        .await?;
    assert_eq!(freshness(pool, "s").await?, (0, 0));
    store
        .finish_scope(second, ScopeOutcome::Failed, 102, None)
        .await?;
    assert_eq!(freshness(pool, "s").await?, (0, 0));
    Ok(())
}

async fn manifest(store: &PgStore, sweep: &str, regions: &[&str]) -> TestResult {
    let id = SweepId::new(sweep.to_owned())?;
    store.create_sweep_manifest(&id, ACCOUNT).await?;
    let works = regions
        .iter()
        .map(|region| work(sweep, region))
        .collect::<TestResult<Vec<_>>>()?;
    store.seed_works(&works).await?;
    Ok(())
}

async fn freshness(pool: &PgPool, sweep: &str) -> TestResult<(i64, i64)> {
    Ok(
        sqlx::query_as::<_, (i64, i64)>(
            "SELECT completed,ended_at FROM inventory.sweeps WHERE sweep_id=$1",
        )
        .bind(sweep)
        .fetch_one(pool)
        .await?,
    )
}
