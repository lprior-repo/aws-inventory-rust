use super::*;

#[tokio::test]
pub(super) async fn heartbeat_prevents_early_reclaim_and_never_shortens_lease() -> TestResult {
    let store = JobStore::open(":memory:").await?;
    store.submit(spec("heart", 2), 100).await?;
    let lease = store.claim("w", "t", 100, 20000).await?.ok_or("claim")?;
    assert!(!store.heartbeat(&lease, 1000, 10000).await?);
    assert_eq!(store.get("heart").await?.lease_until, Some(20100));
    assert!(!store.heartbeat(&lease, 20000, 10000).await?);
    assert!(store.claim("other", "other", 20100, 10000).await?.is_none());
    assert_eq!(store.get("heart").await?.lease_until, Some(30000));
    assert_eq!(store.history("heart").await?.len(), 1);
    store.close().await;
    Ok(())
}
