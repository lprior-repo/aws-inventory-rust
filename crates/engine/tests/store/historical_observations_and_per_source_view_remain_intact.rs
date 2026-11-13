//! Behavior: historical_observations_and_per_source_view_remain_intact.
use super::*;

#[tokio::test]
pub(super) async fn historical_observations_and_per_source_view_remain_intact() -> TestResult {
    let store = Store::open(":memory:").await?;
    store.begin_scan("one", 1).await?;
    for (collector, source, time) in [
        ("direct", Source::Direct, 10),
        ("config", Source::AwsConfig, 200),
        ("explorer", Source::ResourceExplorer, 50),
    ] {
        store
            .apply_page(
                "one",
                page(collector, source, vec![resource("i-1", source, time)]),
            )
            .await?;
    }
    assert_eq!(
        store
            .query("SELECT source,observed_at FROM latest_resources", 10)
            .await?,
        json!([{"source":"aws_config", "observed_at":200}])
    );
    assert_eq!(
        store
            .query("SELECT count(*) AS total FROM resource_observations", 10)
            .await?,
        json!([{"total":3}])
    );
    // The per-source view still exposes each source's own newest observation.
    assert_eq!(
        store
            .query(
                "SELECT source,source_priority,observed_at FROM latest_resource_sources
                 ORDER BY source",
                10
            )
            .await?,
        json!([
            {"source":"aws_config", "source_priority":2, "observed_at":200},
            {"source":"direct", "source_priority":4, "observed_at":10},
            {"source":"resource_explorer", "source_priority":1, "observed_at":50}
        ])
    );
    Ok(())
}
