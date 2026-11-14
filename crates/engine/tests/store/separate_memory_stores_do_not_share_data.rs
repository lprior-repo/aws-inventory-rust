use super::*;

#[tokio::test]
pub(super) async fn separate_memory_stores_do_not_share_data() -> TestResult {
    let first = Store::open(":memory:").await?;
    let second = Store::open(":memory:").await?;
    first.begin_scan("private", 1).await?;
    assert!(matches!(
        second.summary("private").await,
        Err(StoreError::MissingScan(_))
    ));
    first.close().await;
    second.close().await;
    Ok(())
}
