use super::*;

#[tokio::test]
pub(super) async fn freshest_observation_wins_and_latest_within_source_are_deterministic()
-> TestResult {
    let store = Store::open(":memory:").await?;
    store.begin_scan("one", 1).await?;
    for (collector, source, time) in [
        ("explorer", Source::ResourceExplorer, 100),
        ("direct-a", Source::Direct, 10),
        ("config", Source::AwsConfig, 200),
        ("direct-b", Source::Direct, 20),
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
            .query("SELECT count(*) AS total FROM latest_resource_sources", 10)
            .await?,
        json!([{"total":3}])
    );
    assert_eq!(
        store
            .query("SELECT count(*) AS total FROM resource_observations", 10)
            .await?,
        json!([{"total":4}])
    );

    assert_eq!(store.summary("one").await?.resources, 1);
    assert_eq!(
        store
            .query("SELECT source,observed_at FROM scan_resources", 10)
            .await?,
        json!([{"source":"direct", "observed_at":20}])
    );
    Ok(())
}
