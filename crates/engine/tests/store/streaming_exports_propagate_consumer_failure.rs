//! Behavior: streaming_exports_propagate_consumer_failure.
use super::*;

#[tokio::test]
pub(super) async fn streaming_exports_propagate_consumer_failure() -> TestResult {
    let store = Store::open(":memory:").await?;
    store.begin_scan("one", 1).await?;
    store
        .apply_page(
            "one",
            page(
                "ec2",
                Source::Direct,
                vec![
                    resource("i-a", Source::Direct, 10),
                    resource("i-b", Source::Direct, 10),
                ],
            ),
        )
        .await?;
    let mut visits = 0_u32;
    let error = store
        .visit_resources("one", |_| {
            visits = visits.saturating_add(1);
            Err(StoreError::Io(std::io::Error::other("consumer stopped")))
        })
        .await;
    assert!(matches!(error, Err(StoreError::Io(_))));
    assert_eq!(visits, 1);
    assert_eq!(store.resources("one").await?.len(), 2);
    Ok(())
}
