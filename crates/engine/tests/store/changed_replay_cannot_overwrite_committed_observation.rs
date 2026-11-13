//! Behavior: changed_replay_cannot_overwrite_committed_observation.
use super::*;

#[tokio::test]
pub(super) async fn changed_replay_cannot_overwrite_committed_observation() -> TestResult {
    let store = Store::open(":memory:").await?;
    store.begin_scan("one", 1).await?;
    let input = page(
        "ec2",
        Source::Direct,
        vec![resource("i-1", Source::Direct, 10)],
    );
    store.apply_page("one", input.clone()).await?;
    let changed = page(
        "ec2",
        Source::Direct,
        vec![resource("i-1", Source::Direct, 11)],
    );
    assert!(matches!(
        store.apply_page("one", changed.clone()).await,
        Err(StoreError::Conflict(_))
    ));
    assert_eq!(store.resources("one").await?, input.resources);
    Ok(())
}
