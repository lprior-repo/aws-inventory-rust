use super::*;

#[tokio::test]
pub(super) async fn newer_config_observation_outranks_older_direct_observation() -> TestResult {
    let store = Store::open(":memory:").await?;
    store.begin_scan("one", 1).await?;
    for (collector, source, time) in [
        ("direct", Source::Direct, 10),
        ("config", Source::AwsConfig, 200),
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
        json!([{"source":"aws_config", "source_priority":2, "observed_at":200}])
    );

    assert_eq!(
        store
            .query("SELECT source,observed_at FROM scan_resources", 10)
            .await?,
        json!([{"source":"direct", "observed_at":10}])
    );
    Ok(())
}
