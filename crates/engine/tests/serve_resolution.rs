mod postgres_support;

use std::time::Duration;

use inventory_core::inventory::{
    AccountId, OperationId, Region, ScopeKey, ScopeOutcome, ServiceId,
};
use inventory_engine::cadence::{Action, Millis, RunOutcome, Scheduler};
use inventory_engine::inventory::{
    DatabaseUrl, PgStore, PgWriteConfig, RunStart, SweepId, WorkSpec, last_completed_sweep,
    plan_run,
};
use inventory_harness::postgres::{TestDatabase, admin_url};
use postgres_support::{TestResult, registry};

const OWNER: &str = "111111111111";
const OTHER: &str = "222222222222";
const THIRD: &str = "333333333333";

async fn run(check: impl AsyncFn(&PgStore, &DatabaseUrl) -> TestResult) -> TestResult {
    let database = TestDatabase::create(&admin_url()?).await?;
    let url = DatabaseUrl::parse(database.url())?;
    let (registry, _) = registry(1, None)?;
    let store = PgStore::create(&url, registry, PgWriteConfig::default()).await?;
    let result = check(&store, &url).await;
    store.close().await;
    database.finish().await?;
    result
}

async fn restart<T>(
    seed: impl AsyncFn(&PgStore) -> TestResult<T>,
    check: impl AsyncFn(&PgStore) -> TestResult,
) -> TestResult {
    let database = TestDatabase::create(&admin_url()?).await?;
    let result = restart_in(&database, seed, check).await;
    database.finish().await?;
    result
}

async fn restart_in<T>(
    database: &TestDatabase,
    seed: impl AsyncFn(&PgStore) -> TestResult<T>,
    check: impl AsyncFn(&PgStore) -> TestResult,
) -> TestResult {
    let url = DatabaseUrl::parse(database.url())?;
    let (registry, _) = registry(1, None)?;
    let store = PgStore::create(&url, registry, PgWriteConfig::default()).await?;
    let seeded = seed(&store).await;
    store.close().await;
    seeded?;
    let reopened = PgStore::open(&url, PgWriteConfig::default()).await?;
    let result = check(&reopened).await;
    reopened.close().await;
    result
}

fn account(id: &str) -> TestResult<AccountId> {
    Ok(AccountId::parse(id)?)
}

fn work(sweep: &str) -> TestResult<WorkSpec> {
    let scope = ScopeKey::new(
        AccountId::parse("123456789012")?,
        Region::parse("us-east-1")?,
        ServiceId::parse("example")?,
        OperationId::parse("ListThings0")?,
        None,
    )?;
    Ok(WorkSpec::new(
        SweepId::new(sweep.to_owned())?,
        scope,
        None,
        Vec::new(),
    )?)
}

async fn seed_pending(store: &PgStore, sweep: &str, owner: &str) -> TestResult<SweepId> {
    let id = SweepId::new(sweep.to_owned())?;
    store.create_sweep_manifest(&id, owner).await?;
    store.seed_works(&[work(sweep)?]).await?;
    Ok(id)
}

async fn finish(store: &PgStore, sweep: &SweepId, outcome: ScopeOutcome, at: i64) -> TestResult {
    let lease = store
        .claim(sweep, at, 60_000)
        .await?
        .ok_or("expected claimable work")?;
    store.finish_scope(lease, outcome, at, None).await?;
    Ok(())
}

async fn complete(store: &PgStore, sweep: &SweepId, at: i64) -> TestResult {
    finish(store, sweep, ScopeOutcome::Complete, at).await
}

fn stalest_seed(last: Vec<Option<inventory_engine::inventory::LastSweep>>) -> Vec<Option<Millis>> {
    last.into_iter()
        .map(|sweep| sweep.map(|sweep| Millis::new(u64::try_from(sweep.ended_at_ms).unwrap_or(0))))
        .collect()
}

async fn seed_for(store: &PgStore, owners: [&str; 3]) -> TestResult<Vec<Option<Millis>>> {
    let mut last = Vec::new();
    for owner in owners {
        last.push(last_completed_sweep(store, owner).await?);
    }
    Ok(stalest_seed(last))
}

#[tokio::test]
async fn postgres_serve_given_a_fresh_database_when_planning_then_a_new_sweep_starts() -> TestResult {
    run(fresh).await
}

async fn fresh(store: &PgStore, _: &DatabaseUrl) -> TestResult {
    assert_eq!(plan_run(store, &account(OWNER)?).await?, RunStart::Fresh);
    Ok(())
}

#[tokio::test]
async fn postgres_serve_given_pending_work_when_planning_then_the_owner_resumes_and_others_are_blocked()
-> TestResult {
    run(pending_ownership).await
}

async fn pending_ownership(store: &PgStore, _: &DatabaseUrl) -> TestResult {
    let sweep = seed_pending(store, "pending", OWNER).await?;
    assert_eq!(
        plan_run(store, &account(OWNER)?).await?,
        RunStart::Resume(sweep.clone())
    );
    assert_eq!(
        plan_run(store, &account(OTHER)?).await?,
        RunStart::Blocked(sweep)
    );
    Ok(())
}

#[tokio::test]
async fn postgres_serve_given_a_completed_sweep_when_planning_then_the_next_cycle_starts_fresh()
-> TestResult {
    run(completed_cycle).await
}

async fn completed_cycle(store: &PgStore, _: &DatabaseUrl) -> TestResult {
    let sweep = seed_pending(store, "done", OWNER).await?;
    complete(store, &sweep, 5_000).await?;
    assert_eq!(plan_run(store, &account(OWNER)?).await?, RunStart::Fresh);
    let last = last_completed_sweep(store, OWNER)
        .await?
        .ok_or("completed sweep not recorded")?;
    assert_eq!((last.sweep_id.as_str(), last.ended_at_ms), ("done", 5_000));
    assert!(last_completed_sweep(store, OTHER).await?.is_none());
    Ok(())
}

#[tokio::test]
async fn postgres_serve_given_a_failed_sweep_when_querying_history_then_only_the_successful_one_counts()
-> TestResult {
    run(failed_after_success).await
}

async fn failed_after_success(store: &PgStore, _: &DatabaseUrl) -> TestResult {
    let good = seed_pending(store, "good", OWNER).await?;
    complete(store, &good, 1_000).await?;
    let bad = seed_pending(store, "bad", OWNER).await?;
    finish(store, &bad, ScopeOutcome::Failed, 2_000).await?;
    assert_eq!(plan_run(store, &account(OWNER)?).await?, RunStart::Fresh);
    let last = last_completed_sweep(store, OWNER)
        .await?
        .ok_or("successful sweep not recorded")?;
    assert_eq!((last.sweep_id.as_str(), last.ended_at_ms), ("good", 1_000));
    Ok(())
}

#[tokio::test]
async fn postgres_serve_given_unfinished_work_when_querying_history_then_it_is_not_reported_completed()
-> TestResult {
    run(unfinished).await
}

async fn unfinished(store: &PgStore, _: &DatabaseUrl) -> TestResult {
    seed_pending(store, "pending", OWNER).await?;
    assert!(last_completed_sweep(store, OWNER).await?.is_none());
    Ok(())
}

#[tokio::test]
async fn postgres_serve_given_ties_when_querying_history_then_the_latest_end_and_greatest_id_win()
-> TestResult {
    run(ties).await
}

async fn ties(store: &PgStore, _: &DatabaseUrl) -> TestResult {
    for (sweep, at) in [("z", 1_000), ("b", 2_000)] {
        let id = seed_pending(store, sweep, OWNER).await?;
        complete(store, &id, at).await?;
    }
    let id = seed_pending(store, "a", OWNER).await?;
    complete(store, &id, 2_000).await?;
    let last = last_completed_sweep(store, OWNER)
        .await?
        .ok_or("completed sweep not recorded")?;
    assert_eq!((last.sweep_id.as_str(), last.ended_at_ms), ("b", 2_000));
    Ok(())
}

#[tokio::test]
async fn postgres_serve_given_a_restart_with_pending_work_when_planning_then_ownership_persists()
-> TestResult {
    restart(seed_pending_owner, pending_survives_restart).await
}

async fn seed_pending_owner(store: &PgStore) -> TestResult<SweepId> {
    seed_pending(store, "pending", OWNER).await
}

async fn pending_survives_restart(store: &PgStore) -> TestResult {
    let sweep = SweepId::new("pending".to_owned())?;
    assert_eq!(
        plan_run(store, &account(OWNER)?).await?,
        RunStart::Resume(sweep.clone())
    );
    assert_eq!(
        plan_run(store, &account(OTHER)?).await?,
        RunStart::Blocked(sweep)
    );
    Ok(())
}

#[tokio::test]
async fn postgres_serve_given_a_restart_with_history_when_scheduling_then_staleness_is_followed()
-> TestResult {
    restart(seed_history, schedule_after_restart).await
}

async fn seed_history(store: &PgStore) -> TestResult {
    let newer = seed_pending(store, "newer", OWNER).await?;
    complete(store, &newer, 2_000).await?;
    let older = seed_pending(store, "older", OTHER).await?;
    complete(store, &older, 1_000).await?;
    Ok(())
}

async fn schedule_after_restart(store: &PgStore) -> TestResult {
    let seed = seed_for(store, [OWNER, OTHER, THIRD]).await?;
    let mut scheduler = Scheduler::seeded(Duration::from_secs(60), seed, Millis::new(3_000))?;
    let mut order = Vec::new();
    while let Action::Run { profile } = scheduler.next_action(Millis::new(3_000)) {
        order.push(profile);
        scheduler.record(profile, RunOutcome::Completed, Millis::new(3_000))?;
    }
    assert_eq!(order, vec![2, 1, 0]);
    Ok(())
}

#[tokio::test]
async fn postgres_serve_given_a_profile_blocked_by_another_owner_when_scheduling_then_it_is_deferred()
-> TestResult {
    run(deferred_profile).await
}

async fn deferred_profile(store: &PgStore, _: &DatabaseUrl) -> TestResult {
    let sweep = seed_pending(store, "pending", OWNER).await?;
    let accounts = [account(OTHER)?, account(OWNER)?];
    let mut scheduler = Scheduler::seeded(
        Duration::from_secs(60),
        vec![None, None],
        Millis::new(3_000),
    )?;
    while let Action::Run { profile } = scheduler.next_action(Millis::new(3_000)) {
        let outcome = match plan_run(store, &accounts[profile]).await? {
            RunStart::Blocked(blocked) => {
                assert_eq!(blocked, sweep);
                RunOutcome::NotOwner
            }
            RunStart::Resume(_) | RunStart::Fresh => RunOutcome::Completed,
        };
        scheduler.record(profile, outcome, Millis::new(3_000))?;
    }
    assert_eq!(scheduler.last_completed(), &[None, Some(Millis::new(3_000))]);
    Ok(())
}
