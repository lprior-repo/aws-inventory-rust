use super::*;

pub(super) async fn first_lease_phase(
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

pub(super) async fn resumed_lease_phase(
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
            .complete(
                &old,
                completion(JobState::Succeeded, value).map_err(failure)?,
                expiry
            )
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

        old.job = replacement.job.clone();
        let before =
            serde_json::to_value(store.get(JOB).await.map_err(failure)?).map_err(failure)?;
        prop_assert!(matches!(
            store.heartbeat(&old, expiry, 10_000).await,
            Err(JobError::LostLease)
        ));
        prop_assert!(matches!(
            store
                .complete(
                    &old,
                    completion(JobState::Failed, value).map_err(failure)?,
                    expiry
                )
                .await,
            Err(JobError::LostLease)
        ));
        prop_assert_eq!(
            serde_json::to_value(store.get(JOB).await.map_err(failure)?).map_err(failure)?,
            before
        );
        let done = store
            .complete(
                &replacement,
                completion(JobState::Succeeded, value).map_err(failure)?,
                expiry,
            )
            .await
            .map_err(failure)?;
        prop_assert_eq!(done.state, JobState::Succeeded);
        prop_assert!(done.lease_owner.is_none() && done.lease_until.is_none());
        prop_assert!(matches!(
            store
                .complete(
                    &replacement,
                    completion(JobState::Succeeded, value).map_err(failure)?,
                    expiry
                )
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

pub(super) async fn lease_reopen_case(
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
