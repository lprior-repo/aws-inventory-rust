use super::*;

#[tokio::test]
pub(super) async fn edge_failure_also_rolls_back_resources_and_checkpoint() -> TestResult {
    let store = Store::open(":memory:").await?;
    store.begin_scan("one", 1).await?;
    let mut broken = page(
        "ec2",
        Source::Direct,
        vec![resource("i-1", Source::Direct, 10)],
    );
    broken.edges = vec![relationship(), relationship()];
    assert!(store.apply_page("one", broken.clone()).await.is_err());
    assert_eq!(store.summary("one").await?.resources, 0);
    assert!(store.checkpoint("one", &broken.scope).await?.is_none());
    Ok(())
}
