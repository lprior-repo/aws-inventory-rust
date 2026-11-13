//! Behavior: expired_cancelled_work_is_not_requeued.
use super::*;

#[tokio::test]
pub(super) async fn expired_cancelled_work_is_not_requeued() -> TestResult {
    let store = JobStore::open(":memory:").await?;
    store.submit(spec("stop", 3), 100).await?;
    let lease = store.claim("w", "t", 100, 10000).await?.ok_or("claim")?;
    store.cancel("stop", 101).await?;
    assert!(store.claim("next", "next", 10100, 10000).await?.is_none());
    assert_eq!(store.get("stop").await?.state, JobState::Cancelled);
    assert!(matches!(
        store.complete(&lease, success(), 10101).await,
        Err(JobError::LostLease)
    ));
    store.close().await;
    Ok(())
}
