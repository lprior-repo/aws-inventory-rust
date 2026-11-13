//! Behavior: newer_direct_observation_outranks_older_config_observation.
use super::*;

#[tokio::test]
pub(super) async fn newer_direct_observation_outranks_older_config_observation() -> TestResult {
    let store = Store::open(":memory:").await?;
    store.begin_scan("one", 1).await?;
    for (collector, source, time) in [
        ("config", Source::AwsConfig, 10),
        ("direct", Source::Direct, 200),
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
            .query(
                "SELECT source,source_priority,observed_at FROM latest_resources",
                10
            )
            .await?,
        json!([{"source":"direct", "source_priority":4, "observed_at":200}])
    );
    Ok(())
}
