//! Small generated command sequences exercise SQLx state, ownership and restart.
//! The oracle is an append-only list of caller-visible attempts and outcomes.

use std::collections::BTreeSet;
use std::future::Future;
use std::time::Duration;

use futures_util::{StreamExt, stream};
use inventory_engine::jobs::{Completion, JobError, JobSpec, JobState, JobStore, Lease};
use proptest::prelude::*;
use proptest::test_runner::{Config, RngSeed, TestCaseError, TestCaseResult};
use serde_json::json;
use tokio::runtime::{Builder, Runtime};

const JOB: &str = "property-job";
const START: i64 = 100;

fn failure(error: impl std::fmt::Display) -> TestCaseError {
    TestCaseError::fail(error.to_string())
}

fn runtime() -> Result<Runtime, TestCaseError> {
    // Queue time is an explicit integer capability. SQLx worker I/O uses real
    // Tokio scheduling; paused time would race its connection-acquire timers.
    Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(failure)
}

async fn bounded_case(case: impl Future<Output = TestCaseResult>) -> TestCaseResult {
    tokio::time::timeout(Duration::from_secs(10), case)
        .await
        .map_err(|_| failure("durable job property exceeded its 10-second case deadline"))?
}

fn spec(id: &str, budget: u32, value: u16) -> JobSpec {
    JobSpec {
        job_id: id.into(),
        kind: "synthetic_scan".into(),
        payload: json!({"value": value}),
        max_attempts: budget,
        available_at: START,
    }
}

fn completion(state: JobState, value: u16) -> Completion {
    Completion {
        state,
        result: Some(json!({"value": value, "reported": state})),
        error: (state == JobState::Failed).then(|| "synthetic failure".into()),
    }
}

fn terminal_state(value: u8) -> JobState {
    match value {
        0 => JobState::Succeeded,
        1 => JobState::Partial,
        2 => JobState::Failed,
        _ => JobState::Cancelled,
    }
}

async fn first_lease_phase(
    store: &JobStore,
    budget: u32,
    initial_ms: i64,
    heartbeats: &[(u16, u16)],
    value: u16,
) -> Result<(Lease, i64, i64), TestCaseError> {
    store
        .submit(spec(JOB, budget, value), START)
        .await
        .map_err(failure)?;
    let lease = store
        .claim("worker", "same-token", START, initial_ms)
        .await
        .map_err(failure)?
        .ok_or_else(|| failure("initial claim missing"))?;
    let mut now = START;
    let mut deadline = START.saturating_add(initial_ms);
    for (elapsed, extension) in heartbeats.iter().take(8) {
        now = now.saturating_add(i64::from(*elapsed));
        let lease_ms = 10_000_i64.saturating_add(i64::from(*extension));
        let result = store.heartbeat(&lease, now, lease_ms).await;
        if now < deadline {
            prop_assert!(!result.map_err(failure)?);
            deadline = deadline.max(now.saturating_add(lease_ms));
        } else {
            prop_assert!(matches!(result, Err(JobError::LostLease)));
        }
        let record = store.get(JOB).await.map_err(failure)?;
        prop_assert_eq!(record.lease_until, Some(deadline));
        prop_assert_eq!(record.state, JobState::Running);
        prop_assert_eq!(record.attempt, 1);
        prop_assert_eq!(store.history(JOB).await.map_err(failure)?.len(), 1);
    }
    Ok((lease, now, deadline))
}

async fn resumed_lease_phase(
    store: &JobStore,
    mut old: Lease,
    now: i64,
    deadline: i64,
    budget: u32,
    value: u16,
) -> TestCaseResult {
    let persisted = store.get(JOB).await.map_err(failure)?;
    prop_assert_eq!(persisted.lease_until, Some(deadline));
    prop_assert_eq!(persisted.payload, Some(json!({"value":value})));
    let expiry = now.max(deadline);
    prop_assert!(matches!(
        store.heartbeat(&old, expiry, 10_000).await,
        Err(JobError::LostLease)
    ));
    prop_assert!(matches!(
        store
            .complete(&old, completion(JobState::Succeeded, value), expiry)
            .await,
        Err(JobError::LostLease)
    ));
    let replacement = store
        .claim("worker", "same-token", expiry, 10_000)
        .await
        .map_err(failure)?;
    prop_assert_eq!(replacement.is_some(), budget > 1);
    if let Some(replacement) = replacement {
        prop_assert_eq!(replacement.job.attempt, 2);
        // The entire public presentation can be copied from the new owner.
        // It still must not replace the old lease's immutable capability.
        old.job = replacement.job.clone();
        let before =
            serde_json::to_value(store.get(JOB).await.map_err(failure)?).map_err(failure)?;
        prop_assert!(matches!(
            store.heartbeat(&old, expiry, 10_000).await,
            Err(JobError::LostLease)
        ));
        prop_assert!(matches!(
            store
                .complete(&old, completion(JobState::Failed, value), expiry)
                .await,
            Err(JobError::LostLease)
        ));
        prop_assert_eq!(
            serde_json::to_value(store.get(JOB).await.map_err(failure)?).map_err(failure)?,
            before
        );
        let done = store
            .complete(&replacement, completion(JobState::Succeeded, value), expiry)
            .await
            .map_err(failure)?;
        prop_assert_eq!(done.state, JobState::Succeeded);
        prop_assert!(done.lease_owner.is_none() && done.lease_until.is_none());
        prop_assert!(matches!(
            store
                .complete(&replacement, completion(JobState::Succeeded, value), expiry)
                .await,
            Err(JobError::LostLease)
        ));
        let history = store.history(JOB).await.map_err(failure)?;
        prop_assert_eq!(
            history
                .iter()
                .map(|row| row.state.as_str())
                .collect::<Vec<_>>(),
            vec!["expired", "succeeded"]
        );
        prop_assert_eq!(
            history.iter().map(|row| row.attempt).collect::<Vec<_>>(),
            vec![1, 2]
        );
    } else {
        let record = store.get(JOB).await.map_err(failure)?;
        prop_assert_eq!(record.state, JobState::Failed);
        prop_assert_eq!(record.attempt, 1);
        prop_assert!(record.lease_owner.is_none() && record.lease_until.is_none());
        let history = store.history(JOB).await.map_err(failure)?;
        prop_assert_eq!(
            history.first().map(|row| row.state.as_str()),
            Some("expired")
        );
    }
    Ok(())
}

async fn lease_reopen_case(
    budget: u32,
    initial_ms: i64,
    heartbeats: &[(u16, u16)],
    value: u16,
) -> TestCaseResult {
    let folder = tempfile::tempdir().map_err(failure)?;
    let path = folder.path().join("leases.sqlite");
    let store = JobStore::open(&path).await.map_err(failure)?;
    let first = first_lease_phase(&store, budget, initial_ms, heartbeats, value).await;
    store.close().await;
    let (lease, now, deadline) = first?;
    let reopened = JobStore::open(&path).await.map_err(failure)?;
    let result = resumed_lease_phase(&reopened, lease, now, deadline, budget, value).await;
    reopened.close().await;
    result
}

#[derive(Clone, Copy, Debug)]
struct FinishCommand {
    outcome: u8,
    cancel: bool,
    additional: u32,
    backwards_ms: u16,
}

fn finish_command() -> impl Strategy<Value = FinishCommand> {
    (0_u8..4, any::<bool>(), 1_u32..=4, 0_u16..=90).prop_map(
        |(outcome, cancel, additional, backwards_ms)| FinishCommand {
            outcome,
            cancel,
            additional,
            backwards_ms,
        },
    )
}

async fn terminal_commands(
    store: &JobStore,
    initial_budget: u32,
    commands: &[FinishCommand],
    value: u16,
) -> TestCaseResult {
    let original = spec(JOB, initial_budget, value);
    store
        .submit(original.clone(), START)
        .await
        .map_err(failure)?;
    let mut outcomes = Vec::new();
    let mut budget = initial_budget;
    let mut now = START;
    for command in commands.iter().take(8) {
        let lease = store
            .claim("worker", "token", now, 10_000)
            .await
            .map_err(failure)?
            .ok_or_else(|| failure("authorized retry not claimable"))?;
        let attempt = u32::try_from(outcomes.len().saturating_add(1)).map_err(failure)?;
        prop_assert_eq!(lease.job.attempt, attempt);
        prop_assert_eq!(lease.job.max_attempts, budget);
        let activity_time = now.saturating_add(100);
        prop_assert!(
            !store
                .heartbeat(&lease, activity_time, 10_000)
                .await
                .map_err(failure)?
        );
        if command.cancel {
            let pending = store.cancel(JOB, activity_time).await.map_err(failure)?;
            prop_assert_eq!(pending.state, JobState::Running);
            prop_assert!(pending.cancel_requested);
            prop_assert!(
                store
                    .heartbeat(&lease, activity_time, 10_000)
                    .await
                    .map_err(failure)?
            );
        }
        let requested = terminal_state(command.outcome);
        let expected = if command.cancel {
            JobState::Cancelled
        } else {
            requested
        };
        let completed_at = activity_time.saturating_sub(i64::from(command.backwards_ms));
        let result = store
            .complete(&lease, completion(requested, value), completed_at)
            .await
            .map_err(failure)?;
        outcomes.push(expected);
        prop_assert_eq!(result.state, expected);
        prop_assert_eq!(result.updated_at, activity_time);
        prop_assert_eq!(
            &result.result,
            &Some(json!({"value":value,"reported":requested}))
        );
        prop_assert!(result.lease_owner.is_none() && result.lease_until.is_none());
        let history = store.history(JOB).await.map_err(failure)?;
        prop_assert_eq!(
            history
                .iter()
                .map(|row| row.state.as_str())
                .collect::<Vec<_>>(),
            outcomes
                .iter()
                .map(|state| state.as_str())
                .collect::<Vec<_>>()
        );
        prop_assert_eq!(
            history.iter().map(|row| row.attempt).collect::<Vec<_>>(),
            (1..=attempt).collect::<Vec<_>>()
        );
        prop_assert!(
            history
                .iter()
                .all(|row| row.finished_at.is_some_and(|time| time >= row.started_at))
        );

        // Submission cannot resurrect or rewrite completed work, even after an
        // explicit retry changed the mutable remaining attempt budget.
        let snapshot = serde_json::to_value(&result).map_err(failure)?;
        let mut repeated = original.clone();
        repeated.available_at = activity_time.saturating_add(50_000);
        let replay = store
            .submit(repeated, activity_time)
            .await
            .map_err(failure)?;
        prop_assert_eq!(
            serde_json::to_value(replay).map_err(failure)?,
            snapshot.clone()
        );
        let mut conflict = original.clone();
        conflict.payload = json!({"different":value});
        prop_assert!(matches!(
            store.submit(conflict, activity_time).await,
            Err(JobError::Conflict)
        ));
        prop_assert_eq!(
            serde_json::to_value(store.get(JOB).await.map_err(failure)?).map_err(failure)?,
            snapshot
        );

        now = activity_time.saturating_add(1);
        let retry = store.retry(JOB, command.additional, now).await;
        if expected == JobState::Succeeded {
            prop_assert!(matches!(retry, Err(JobError::Invalid(_))));
            prop_assert!(
                store
                    .claim("next", "next", now, 10_000)
                    .await
                    .map_err(failure)?
                    .is_none()
            );
            break;
        }
        let queued = retry.map_err(failure)?;
        budget = attempt.saturating_add(command.additional);
        prop_assert_eq!(queued.state, JobState::Queued);
        prop_assert_eq!(queued.max_attempts, budget);
        prop_assert_eq!(queued.attempt, attempt);
        prop_assert_eq!(queued.available_at, now);
        prop_assert!(!queued.cancel_requested);
        prop_assert!(queued.result.is_none() && queued.last_error.is_none());
        prop_assert_eq!(
            store.history(JOB).await.map_err(failure)?.len(),
            outcomes.len()
        );
    }
    let listed = store.list(10).await.map_err(failure)?;
    prop_assert_eq!(listed.len(), 1);
    prop_assert!(
        listed
            .iter()
            .all(|row| row.payload.is_none() && row.result.is_none())
    );
    Ok(())
}

async fn command_case(budget: u32, commands: &[FinishCommand], value: u16) -> TestCaseResult {
    let store = JobStore::open(":memory:").await.map_err(failure)?;
    let result = terminal_commands(&store, budget, commands, value).await;
    store.close().await;
    result
}

async fn competing_claims(
    a: &JobStore,
    b: &JobStore,
    ready: &[bool],
    value: u16,
) -> TestCaseResult {
    let mut expected = BTreeSet::new();
    for (index, is_ready) in ready.iter().enumerate() {
        let id = format!("job-{index}");
        let mut input = spec(&id, 2, value);
        if *is_ready {
            expected.insert(id);
        } else {
            input.available_at = START.saturating_add(1);
        }
        a.submit(input, START).await.map_err(failure)?;
    }
    let results = stream::iter(0..ready.len().saturating_add(4))
        .map(|index| async move {
            let store = if index % 2 == 0 { a } else { b };
            store.claim("worker", "token", START, 10_000).await
        })
        .buffer_unordered(4)
        .collect::<Vec<_>>()
        .await;
    let leases = results
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .map_err(failure)?;
    let claimed: Vec<_> = leases
        .iter()
        .flatten()
        .map(|lease| lease.job.job_id.clone())
        .collect();
    let distinct: BTreeSet<_> = claimed.iter().cloned().collect();
    prop_assert_eq!(claimed.len(), distinct.len());
    prop_assert_eq!(distinct, expected);
    for (index, is_ready) in ready.iter().enumerate() {
        let id = format!("job-{index}");
        let record = b.get(&id).await.map_err(failure)?;
        prop_assert_eq!(record.attempt, u32::from(*is_ready));
        prop_assert_eq!(
            record.state,
            if *is_ready {
                JobState::Running
            } else {
                JobState::Queued
            }
        );
        prop_assert_eq!(
            b.history(&id).await.map_err(failure)?.len(),
            usize::from(*is_ready)
        );
    }
    Ok(())
}

async fn competing_case(ready: &[bool], value: u16) -> TestCaseResult {
    let folder = tempfile::tempdir().map_err(failure)?;
    let path = folder.path().join("claims.sqlite");
    let a = JobStore::open(&path).await.map_err(failure)?;
    let opened = JobStore::open(&path).await;
    let b = match opened {
        Ok(store) => store,
        Err(error) => {
            a.close().await;
            return Err(failure(error));
        }
    };
    let result = competing_claims(&a, &b, ready, value).await;
    a.close().await;
    b.close().await;
    result
}

proptest! {
    #![proptest_config(Config {
        cases: 24,
        max_shrink_iters: 96,
        max_shrink_time: 2_000,
        rng_seed: RngSeed::Fixed(0x4157_535f_4a4f_4253),
        ..Config::default()
    })]

    #[test]
    fn generated_heartbeat_sequences_preserve_fencing_across_reopen(
        budget in 1_u32..=4,
        initial_ms in 10_000_i64..=60_000,
        heartbeats in prop::collection::vec((0_u16..=30_000, 0_u16..=50_000), 0..=8),
        value in any::<u16>(),
    ) {
        runtime()?.block_on(bounded_case(lease_reopen_case(budget, initial_ms, &heartbeats, value)))?;
    }

    #[test]
    fn generated_terminal_commands_preserve_retry_and_append_only_history(
        budget in 1_u32..=4,
        commands in prop::collection::vec(finish_command(), 1..=8),
        value in any::<u16>(),
    ) {
        runtime()?.block_on(bounded_case(command_case(budget, &commands, value)))?;
    }

    #[test]
    fn concurrent_claims_never_duplicate_or_run_future_work(
        ready in prop::collection::vec(any::<bool>(), 1..=6),
        value in any::<u16>(),
    ) {
        runtime()?.block_on(bounded_case(competing_case(&ready, value)))?;
    }
}
