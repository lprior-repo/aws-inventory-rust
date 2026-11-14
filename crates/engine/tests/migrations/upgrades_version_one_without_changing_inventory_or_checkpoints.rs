use super::*;

#[tokio::test]
pub(super) async fn upgrades_version_one_without_changing_inventory_or_checkpoints() -> TestResult {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("legacy.sqlite");
    legacy_database(&path).await?;
    let store = Store::open(&path).await?;
    assert_eq!(store.summary("legacy").await?.resources, 1);
    assert_eq!(store.scan_status("legacy").await?, ScanStatus::Complete);
    assert_eq!(
        store
            .query("SELECT user_version AS version FROM pragma_user_version", 1)
            .await?,
        json!([{"version":CURRENT_SCHEMA_VERSION}])
    );
    assert_eq!(
        store
            .query("SELECT instance_id FROM aws_ec2_instance", 2)
            .await?,
        json!([{"instance_id":"i-legacy"}])
    );
    assert_eq!(
        store
            .query(
                "SELECT target,snapshot_id,complete FROM graph_export_checkpoints",
                2
            )
            .await?,
        json!([{"target":"falkordb","snapshot_id":"legacy-snapshot","complete":1}])
    );
    assert_eq!(
        store
            .query("SELECT count(*) AS total FROM aws_elbv2_listener", 1)
            .await?,
        json!([{"total":0}])
    );
    store.close().await;
    let reopened = Store::open(&path).await?;
    assert_eq!(reopened.summary("legacy").await?.resources, 1);
    assert_eq!(
        reopened
            .query("SELECT count(*) AS total FROM collection_pages", 1)
            .await?,
        json!([{"total":1}])
    );
    reopened.close().await;
    Ok(())
}
