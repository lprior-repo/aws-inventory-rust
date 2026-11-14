use super::*;

pub(super) async fn competing_claims(
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

pub(super) async fn competing_case(ready: &[bool], value: u16) -> TestCaseResult {
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
