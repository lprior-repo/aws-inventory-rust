use super::*;

#[derive(Clone, Copy, Debug)]
pub(super) struct FinishCommand {
    pub(super) outcome: u8,
    pub(super) cancel: bool,
    pub(super) additional: u32,
    pub(super) backwards_ms: u16,
}

pub(super) fn finish_command() -> impl Strategy<Value = FinishCommand> {
    (0_u8..4, any::<bool>(), 1_u32..=4, 0_u16..=90).prop_map(
        |(outcome, cancel, additional, backwards_ms)| FinishCommand {
            outcome,
            cancel,
            additional,
            backwards_ms,
        },
    )
}

pub(super) async fn terminal_commands(
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
            .complete(
                &lease,
                completion(requested, value).map_err(failure)?,
                completed_at,
            )
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

pub(super) async fn command_case(
    budget: u32,
    commands: &[FinishCommand],
    value: u16,
) -> TestCaseResult {
    let store = JobStore::open(":memory:").await.map_err(failure)?;
    let result = terminal_commands(&store, budget, commands, value).await;
    store.close().await;
    result
}
