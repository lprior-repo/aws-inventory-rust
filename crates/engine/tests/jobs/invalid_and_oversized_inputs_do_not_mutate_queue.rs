//! Behavior: invalid_and_oversized_inputs_do_not_mutate_queue.
use super::*;

#[tokio::test]
pub(super) async fn invalid_and_oversized_inputs_do_not_mutate_queue() -> TestResult {
    let store = JobStore::open(":memory:").await?;
    assert!(store.submit(spec("", 3), 100).await.is_err());
    assert!(store.submit(spec("zero", 0), 100).await.is_err());
    assert!(store.submit(spec("many", 33), 100).await.is_err());
    assert!(store.submit(spec("negative", 3), -1).await.is_err());
    let mut big = spec("big", 3);
    big.payload = json!({"oversize":"x".repeat(65536)});
    assert!(store.submit(big, 100).await.is_err());
    assert!(store.claim("w", "t", i64::MAX, 10000).await.is_err());
    assert!(store.claim("w", "t", 100, 9999).await.is_err());
    assert!(store.list(0).await.is_err());
    assert!(store.list(1001).await.is_err());
    assert!(store.list(100).await?.is_empty());
    store.close().await;
    Ok(())
}
