use super::*;

#[tokio::test]
pub(super) async fn schedule_and_deterministic_claim_order() -> TestResult {
    let store = JobStore::open(":memory:").await?;
    store.submit(spec("z", 2), 100).await?;
    store.submit(spec("a", 2), 100).await?;
    assert!(store.claim("worker", "early", 99, 10000).await?.is_none());
    let first = store
        .claim("worker", "one", 100, 10000)
        .await?
        .ok_or("missing claim")?;
    assert_eq!(first.job.job_id, "a");
    assert_eq!(first.job.attempt, 1);
    let second = store
        .claim("worker", "two", 100, 10000)
        .await?
        .ok_or("missing second")?;
    assert_eq!(second.job.job_id, "z");
    assert!(store.claim("worker", "third", 100, 10000).await?.is_none());
    store.close().await;
    Ok(())
}
