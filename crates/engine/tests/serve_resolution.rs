#[path = "inventory_store/support.rs"]
#[allow(
    dead_code,
    reason = "shared via #[path]; each test target uses a subset of the helpers"
)]
mod support;

use std::time::Duration;

use inventory_core::inventory::{AccountId, ScopeOutcome};
use inventory_engine::cadence::{Action, Millis, Scheduler};
use inventory_engine::inventory::{BinaryStore, RunStart, SweepId, last_completed_sweep, plan_run};
use support::TestResult;

fn account(id: &str) -> TestResult<AccountId> {
    Ok(AccountId::parse(id)?)
}

async fn seed_pending(store: &BinaryStore, sweep: &str, owner: &str) -> TestResult<SweepId> {
    let id = SweepId::new(sweep.to_owned())?;
    store.create_sweep_manifest(&id, owner).await?;
    store
        .seed_works(&[support::work(sweep, "us-east-1")?])
        .await?;
    Ok(id)
}

async fn complete(store: &BinaryStore, sweep: &SweepId, at: i64) -> TestResult {
    let lease = store
        .claim(sweep, at, 60_000)
        .await?
        .ok_or("expected claimable work")?;
    store
        .finish_scope(lease, ScopeOutcome::Complete, at, None)
        .await?;
    Ok(())
}

#[tokio::test]
async fn given_a_fresh_database_when_planning_a_run_then_a_new_sweep_starts() -> TestResult {
    let (_directory, store, _catalog) = support::fixture().await?;
    assert_eq!(
        plan_run(&store, &account("111111111111")?).await?,
        RunStart::Fresh
    );
    Ok(())
}

#[tokio::test]
async fn given_pending_work_when_planning_then_the_owner_resumes_and_others_are_blocked()
-> TestResult {
    let (_directory, store, _catalog) = support::fixture().await?;
    let sweep = seed_pending(&store, "pending", "111111111111").await?;
    assert_eq!(
        plan_run(&store, &account("111111111111")?).await?,
        RunStart::Resume(sweep.clone())
    );
    assert_eq!(
        plan_run(&store, &account("222222222222")?).await?,
        RunStart::Blocked(sweep)
    );
    Ok(())
}

#[tokio::test]
async fn given_a_completed_sweep_when_planning_then_the_next_cycle_starts_fresh() -> TestResult {
    let (_directory, store, _catalog) = support::fixture().await?;
    let sweep = seed_pending(&store, "done", "111111111111").await?;
    complete(&store, &sweep, 5_000).await?;
    assert_eq!(
        plan_run(&store, &account("111111111111")?).await?,
        RunStart::Fresh
    );
    let last = last_completed_sweep(&store, "111111111111")
        .await?
        .ok_or("completed sweep not recorded")?;
    assert_eq!((last.sweep_id.as_str(), last.ended_at_ms), ("done", 5_000));
    assert!(
        last_completed_sweep(&store, "222222222222")
            .await?
            .is_none()
    );
    Ok(())
}

#[tokio::test]
async fn given_a_failed_sweep_when_querying_history_then_only_the_successful_one_counts()
-> TestResult {
    let (_directory, store, _catalog) = support::fixture().await?;
    let good = seed_pending(&store, "good", "111111111111").await?;
    complete(&store, &good, 1_000).await?;
    let bad = seed_pending(&store, "bad", "111111111111").await?;
    let lease = store
        .claim(&bad, 2_000, 60_000)
        .await?
        .ok_or("expected claimable work")?;
    store
        .finish_scope(lease, ScopeOutcome::Failed, 2_000, None)
        .await?;
    assert_eq!(
        plan_run(&store, &account("111111111111")?).await?,
        RunStart::Fresh
    );
    let last = last_completed_sweep(&store, "111111111111")
        .await?
        .ok_or("successful sweep not recorded")?;
    assert_eq!((last.sweep_id.as_str(), last.ended_at_ms), ("good", 1_000));
    Ok(())
}

#[tokio::test]
async fn given_unfinished_work_when_querying_history_then_it_is_not_reported_completed()
-> TestResult {
    let (_directory, store, _catalog) = support::fixture().await?;
    seed_pending(&store, "pending", "111111111111").await?;
    assert!(
        last_completed_sweep(&store, "111111111111")
            .await?
            .is_none()
    );
    Ok(())
}

#[tokio::test]
async fn given_persisted_history_when_restarting_then_scheduling_follows_staleness() -> TestResult {
    let (_directory, store, _catalog) = support::fixture().await?;
    let newer = seed_pending(&store, "newer", "111111111111").await?;
    complete(&store, &newer, 2_000).await?;
    let older = seed_pending(&store, "older", "222222222222").await?;
    complete(&store, &older, 1_000).await?;

    let mut seed = Vec::new();
    for owner in ["111111111111", "222222222222", "333333333333"] {
        let last = last_completed_sweep(&store, owner).await?;
        seed.push(last.map(|sweep| Millis::new(u64::try_from(sweep.ended_at_ms).unwrap_or(0))));
    }
    let mut scheduler = Scheduler::seeded(Duration::from_secs(60), seed, Millis::new(3_000))?;
    let mut order = Vec::new();
    while let Action::Run { profile } = scheduler.next_action(Millis::new(3_000)) {
        order.push(profile);
        scheduler.record(
            profile,
            inventory_engine::cadence::RunOutcome::Completed,
            Millis::new(3_000),
        )?;
    }
    assert_eq!(order, vec![2, 1, 0]);
    Ok(())
}
