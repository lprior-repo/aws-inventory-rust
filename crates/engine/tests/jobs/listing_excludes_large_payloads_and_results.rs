//! Behavior: listing_excludes_large_payloads_and_results.
use super::*;

#[tokio::test]
pub(super) async fn listing_excludes_large_payloads_and_results() -> TestResult {
    let store = JobStore::open(":memory:").await?;
    store.submit(spec("visible", 1), 100).await?;
    let lease = store
        .claim("w", "secret-lease-token", 100, 10000)
        .await?
        .ok_or("claim")?;
    store.complete(&lease, success(), 101).await?;
    let full = store.get("visible").await?;
    assert!(full.payload.is_some() && full.result.is_some());
    let list = store.list(1).await?;
    let row = list.first().ok_or("list row")?;
    assert!(row.payload.is_none() && row.result.is_none());
    assert!(!serde_json::to_string(&full)?.contains("secret-lease-token"));
    store.close().await;
    Ok(())
}
