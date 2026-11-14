use super::*;

#[tokio::test]
pub(super) async fn queued_cancellation_never_runs_and_running_cancellation_wins_completion()
-> TestResult {
    let store = JobStore::open(":memory:").await?;
    store.submit(spec("queued", 2), 100).await?;
    assert_eq!(
        store.cancel("queued", 101).await?.state,
        JobState::Cancelled
    );
    assert!(store.claim("w", "t", 102, 10000).await?.is_none());
    store.submit(spec("running", 2), 100).await?;
    let lease = store.claim("w", "run", 102, 10000).await?.ok_or("claim")?;
    assert_eq!(store.cancel("running", 103).await?.state, JobState::Running);
    assert!(store.heartbeat(&lease, 104, 10000).await?);
    let final_job = store.complete(&lease, success(), 105).await?;
    assert_eq!(final_job.state, JobState::Cancelled);
    assert_eq!(
        store
            .history("running")
            .await?
            .first()
            .map(|v| v.state.as_str()),
        Some("cancelled")
    );
    store.close().await;
    Ok(())
}
