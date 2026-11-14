use super::*;

#[tokio::test]
pub(super) async fn newer_observation_within_the_same_source_wins() -> TestResult {
    let store = Store::open(":memory:").await?;
    store.begin_scan("one", 1).await?;
    for (collector, time) in [("direct-a", 10), ("direct-b", 90)] {
        store
            .apply_page(
                "one",
                page(
                    collector,
                    Source::Direct,
                    vec![resource("i-1", Source::Direct, time)],
                ),
            )
            .await?;
    }
    assert_eq!(
        store
            .query("SELECT source,observed_at FROM latest_resources", 10)
            .await?,
        json!([{"source":"direct", "observed_at":90}])
    );
    assert_eq!(
        store
            .query("SELECT count(*) AS total FROM latest_resource_sources", 10)
            .await?,
        json!([{"total":1}])
    );
    assert_eq!(
        store
            .query("SELECT count(*) AS total FROM resource_observations", 10)
            .await?,
        json!([{"total":2}])
    );
    Ok(())
}
