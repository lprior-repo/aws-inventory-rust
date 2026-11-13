//! Behavior: expired_attempt_is_recovered_and_old_owner_is_fenced.
use super::*;

#[tokio::test]
pub(super) async fn expired_attempt_is_recovered_and_old_owner_is_fenced() -> TestResult {
    let store = JobStore::open(":memory:").await?;
    store.submit(spec("recover", 3), 100).await?;
    let mut old = store
        .claim("same-worker", "same-token", 100, 10000)
        .await?
        .ok_or("old lease")?;
    let new = store
        .claim("same-worker", "same-token", 10100, 10000)
        .await?
        .ok_or("new lease")?;
    assert_eq!(new.job.attempt, 2);
    // Presentation metadata cannot forge the immutable fencing capability.
    old.job.attempt = new.job.attempt;
    assert!(matches!(
        store.heartbeat(&old, 10101, 10000).await,
        Err(JobError::LostLease)
    ));
    assert!(matches!(
        store.complete(&old, success(), 10101).await,
        Err(JobError::LostLease)
    ));
    let done = store.complete(&new, success(), 10102).await?;
    assert_eq!(done.state, JobState::Succeeded);
    let history = store.history("recover").await?;
    assert_eq!(
        history.iter().map(|v| v.state.as_str()).collect::<Vec<_>>(),
        vec!["expired", "succeeded"]
    );
    store.close().await;
    Ok(())
}
