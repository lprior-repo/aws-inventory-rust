use super::*;

#[tokio::test]
pub(super) async fn equal_observation_timestamps_use_source_priority_tie_break() -> TestResult {
    let first = Store::open(":memory:").await?;
    let second = Store::open(":memory:").await?;
    first.begin_scan("one", 1).await?;
    second.begin_scan("one", 1).await?;
    let config = page(
        "config",
        Source::AwsConfig,
        vec![resource("i-1", Source::AwsConfig, 50)],
    );
    let cloud = page(
        "cloud",
        Source::CloudControl,
        vec![resource("i-1", Source::CloudControl, 50)],
    );
    let direct = page(
        "direct",
        Source::Direct,
        vec![resource("i-1", Source::Direct, 50)],
    );
    for input in [config.clone(), cloud.clone(), direct.clone()] {
        first.apply_page("one", input).await?;
    }
    for input in [direct, cloud, config] {
        second.apply_page("one", input).await?;
    }
    let expected = json!([{"source":"direct", "source_priority":4, "observed_at":50}]);
    for store in [&first, &second] {
        assert_eq!(
            store
                .query(
                    "SELECT source,source_priority,observed_at FROM latest_resources",
                    10
                )
                .await?,
            expected
        );
    }
    Ok(())
}
