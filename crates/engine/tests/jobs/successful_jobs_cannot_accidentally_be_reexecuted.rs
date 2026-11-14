use super::*;

#[tokio::test]
pub(super) async fn successful_jobs_cannot_accidentally_be_reexecuted() -> TestResult {
    let store = JobStore::open(":memory:").await?;
    store.submit(spec("done", 3), 100).await?;
    let lease = store.claim("w", "t", 100, 10000).await?.ok_or("claim")?;
    store.complete(&lease, success(), 101).await?;
    assert!(store.retry("done", 1, 102).await.is_err());
    assert_eq!(store.cancel("done", 103).await?.state, JobState::Succeeded);
    assert!(store.claim("w", "next", 104, 10000).await?.is_none());
    store.close().await;
    Ok(())
}
