//! Behavior: observation_failure_rolls_back_page_checkpoint_edges_and_other_observations.
use super::*;

#[tokio::test]
pub(super) async fn observation_failure_rolls_back_page_checkpoint_edges_and_other_observations()
-> TestResult {
    let store = Store::open(":memory:").await?;
    store.begin_scan("one", 1).await?;
    let duplicate = resource("i-1", Source::Direct, 10);
    let mut broken = page("ec2", Source::Direct, vec![duplicate.clone(), duplicate]);
    broken.edges.push(relationship());
    assert!(store.apply_page("one", broken.clone()).await.is_err());
    assert!(store.checkpoint("one", &broken.scope).await?.is_none());
    assert_eq!(store.summary("one").await?.resources, 0);
    assert_eq!(store.summary("one").await?.edges, 0);
    assert_eq!(
        store
            .query("SELECT count(*) AS total FROM collection_pages", 10)
            .await?,
        json!([{"total": 0}])
    );
    broken.resources.truncate(1);
    store.apply_page("one", broken.clone()).await?;
    assert_eq!(store.summary("one").await?.resources, 1);
    assert_eq!(store.summary("one").await?.edges, 1);
    Ok(())
}
