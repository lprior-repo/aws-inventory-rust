use super::*;

#[tokio::test]
pub(super) async fn lease_expiration_is_a_hard_commit_boundary() -> TestResult {
    let store = JobStore::open(":memory:").await?;
    store.submit(spec("boundary", 1), 100).await?;
    let lease = store.claim("w", "t", 100, 10000).await?.ok_or("claim")?;
    assert!(matches!(
        store.complete(&lease, success(), 10100).await,
        Err(JobError::LostLease)
    ));
    assert!(matches!(
        store.heartbeat(&lease, 10100, 10000).await,
        Err(JobError::LostLease)
    ));
    assert!(store.claim("new", "new", 10100, 10000).await?.is_none());
    assert_eq!(store.get("boundary").await?.state, JobState::Failed);
    store.close().await;
    Ok(())
}
