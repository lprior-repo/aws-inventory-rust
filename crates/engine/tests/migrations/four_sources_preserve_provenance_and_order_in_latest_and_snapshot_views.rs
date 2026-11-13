//! Behavior: four_sources_preserve_provenance_and_order_in_latest_and_snapshot_views.
use super::*;

#[tokio::test]
pub(super) async fn four_sources_preserve_provenance_and_order_in_latest_and_snapshot_views()
-> TestResult {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("sources.sqlite");
    populated_legacy(&path, 2).await?;
    let store = Store::open(&path).await?;
    store.begin_scan("new-sources", 50).await?;
    for input in [
        source_page(
            Source::CloudControl,
            &["preferred-direct", "preferred-cloud", "new-all"],
            100,
        ),
        source_page(Source::AwsConfig, &["preferred-cloud", "new-all"], 200),
        source_page(
            Source::ResourceExplorer,
            &["preferred-cloud", "new-all"],
            300,
        ),
        source_page(Source::Direct, &["new-all"], 60),
    ] {
        store.apply_page("new-sources", input).await?;
    }
    store
        .finish_scan("new-sources", ScanStatus::Complete, 400)
        .await?;
    let expected = json!([
        {"id":"new-all","source":"resource_explorer","source_priority":1,"observed_at":300},
        {"id":"preferred-cloud","source":"resource_explorer","source_priority":1,"observed_at":300},
        {"id":"preferred-config","source":"resource_explorer","source_priority":1,"observed_at":30},
        {"id":"preferred-direct","source":"cloud_control","source_priority":3,"observed_at":100},
        {"id":"preferred-explorer","source":"resource_explorer","source_priority":1,"observed_at":30}
    ]);
    assert_eq!(
        store
            .query(
                "SELECT id,source,source_priority,observed_at FROM latest_resources ORDER BY id",
                10
            )
            .await?,
        expected
    );
    assert_eq!(store.query("SELECT from_id AS id,source,source_priority,observed_at FROM latest_edges ORDER BY from_id", 10).await?, expected);
    let snapshot = json!([
        {"id":"new-all","source":"direct","source_priority":4},
        {"id":"preferred-cloud","source":"cloud_control","source_priority":3},
        {"id":"preferred-direct","source":"cloud_control","source_priority":3}
    ]);
    assert_eq!(store.query("SELECT id,source,source_priority FROM scan_resources WHERE scan_id='new-sources' ORDER BY id", 10).await?, snapshot);
    assert_eq!(store.query("SELECT from_id AS id,source,source_priority FROM scan_edges WHERE scan_id='new-sources' ORDER BY from_id", 10).await?, snapshot);
    assert_eq!(
        store
            .query("SELECT count(*) AS total FROM latest_resource_sources", 1)
            .await?,
        json!([{"total":14}])
    );
    assert_eq!(store.query("SELECT id,source,source_priority,cloudformation_type FROM aws_cloudcontrol_resource ORDER BY id", 5).await?, json!([
        {"id":"new-all","source":"cloud_control","source_priority":3,"cloudformation_type":"AWS::EC2::Instance"},
        {"id":"preferred-cloud","source":"cloud_control","source_priority":3,"cloudformation_type":"AWS::EC2::Instance"},
        {"id":"preferred-direct","source":"cloud_control","source_priority":3,"cloudformation_type":"AWS::EC2::Instance"}
    ]));
    store.close().await;
    Ok(())
}
