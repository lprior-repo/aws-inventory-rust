use super::*;
use crate::json_test::first as first_row;

#[tokio::test]
async fn diff_ignores_observation_time_and_object_key_order() -> Result<()> {
    let before = Store::open(":memory:").await?;
    let after = Store::open(":memory:").await?;
    let first = resource(
        "a",
        "fixture",
        serde_json::from_str("{\"a\":1,\"b\":[2,3]}")?,
    );
    let mut second = resource(
        "a",
        "fixture",
        serde_json::from_str("{\"b\":[2,3],\"a\":1}")?,
    );
    second.observed_at = 11;
    let old_edge = edge("a", "b", "REFERS_TO");
    let mut new_edge = old_edge.clone();
    new_edge.observed_at = 12;
    snapshot(
        &before,
        "before",
        vec![first],
        vec![old_edge],
        ScanStatus::Complete,
    )
    .await?;
    snapshot(
        &after,
        "after",
        vec![second],
        vec![new_edge],
        ScanStatus::Complete,
    )
    .await?;
    let result = diff::report(&before, &after, "before", "after", 100, 100).await?;
    assert_eq!(result["resources"]["counts"]["unchanged"], 1);
    assert_eq!(result["resources"]["counts"]["changed"], 0);
    assert_eq!(result["relationships"]["counts"]["unchanged"], 1);
    assert_eq!(result["resources"]["rows"], json!([]));
    before.close().await;
    after.close().await;
    Ok(())
}

#[tokio::test]
async fn diff_reports_tags_arrays_and_provider_timestamps_as_data() -> Result<()> {
    let before = Store::open(":memory:").await?;
    let after = Store::open(":memory:").await?;
    let first = resource("a", "fixture", json!({"items":[1,2],"updated_at":1}));
    let mut second = resource("a", "fixture", json!({"items":[2,1],"updated_at":2}));
    second.tags.insert("owner".into(), "synthetic".into());
    snapshot(&before, "before", vec![first], vec![], ScanStatus::Complete).await?;
    snapshot(&after, "after", vec![second], vec![], ScanStatus::Complete).await?;
    let result = diff::report(&before, &after, "before", "after", 100, 100).await?;
    assert_eq!(result["resources"]["counts"]["changed"], 1);
    assert_eq!(
        first_row(&result["resources"]["rows"])["changed_fields"],
        json!(["properties", "tags"])
    );
    before.close().await;
    after.close().await;
    Ok(())
}

#[tokio::test]
async fn diff_absence_is_not_deletion_even_for_complete_scans() -> Result<()> {
    let before = Store::open(":memory:").await?;
    let after = Store::open(":memory:").await?;
    snapshot(
        &before,
        "before",
        vec![resource("a", "fixture", json!({}))],
        vec![edge("a", "missing", "REFERS_TO")],
        ScanStatus::Complete,
    )
    .await?;
    snapshot(
        &after,
        "after",
        vec![resource("b", "fixture", json!({}))],
        vec![],
        ScanStatus::Complete,
    )
    .await?;
    let result = diff::report(&before, &after, "before", "after", 100, 100).await?;
    assert_eq!(result["resources"]["counts"]["newly_observed"], 1);
    assert_eq!(result["resources"]["counts"]["no_longer_observed"], 1);
    assert_eq!(result["relationships"]["counts"]["no_longer_observed"], 1);
    assert_eq!(
        first_row(&result["resources"]["rows"])["change"],
        "no_longer_observed"
    );
    assert_eq!(
        first_row(&result["resources"]["rows"])["deletion_inferred"],
        false
    );
    assert_eq!(nth(&result["resources"]["rows"], 1)["after"]["id"], "b");
    before.close().await;
    after.close().await;
    Ok(())
}

#[tokio::test]
async fn diff_keeps_partial_inputs_and_source_changes_explicit() -> Result<()> {
    let before = Store::open(":memory:").await?;
    let after = Store::open(":memory:").await?;
    let first = resource("a", "fixture", json!({"x":1}));
    let mut second = first.clone();
    second.source = Source::CloudControl;
    snapshot(&before, "before", vec![first], vec![], ScanStatus::Complete).await?;
    snapshot(&after, "after", vec![second], vec![], ScanStatus::Partial).await?;
    let result = diff::report(&before, &after, "before", "after", 100, 100).await?;
    assert_eq!(result["status"], "partial");
    assert_eq!(result["after_scan"]["status"], "partial");
    assert_eq!(result["resources"]["counts"]["source_changed"], 1);
    assert_eq!(
        first_row(&result["resources"]["rows"])["changed_fields"],
        json!(["source"])
    );
    before.close().await;
    after.close().await;
    Ok(())
}

#[tokio::test]
async fn diff_detail_cap_does_not_clip_comparison_counts() -> Result<()> {
    let before = Store::open(":memory:").await?;
    let after = Store::open(":memory:").await?;
    snapshot(&before, "before", vec![], vec![], ScanStatus::Complete).await?;
    snapshot(
        &after,
        "after",
        vec![
            resource("a", "fixture", json!({})),
            resource("b", "fixture", json!({})),
        ],
        vec![],
        ScanStatus::Complete,
    )
    .await?;
    let result = diff::report(&before, &after, "before", "after", 1, 100).await?;
    assert_eq!(result["resources"]["counts"]["newly_observed"], 2);
    assert_eq!(result["resources"]["details_truncated"], true);
    assert_eq!(result["resources"]["comparison_truncated"], false);
    assert_eq!(
        result["resources"]["rows"].as_array().map(Vec::len),
        Some(1)
    );
    assert_eq!(result["status"], "complete");
    before.close().await;
    after.close().await;
    Ok(())
}

#[tokio::test]
async fn diff_comparison_cap_has_real_probe_and_partial_status() -> Result<()> {
    let before = Store::open(":memory:").await?;
    let after = Store::open(":memory:").await?;
    snapshot(&before, "before", vec![], vec![], ScanStatus::Complete).await?;
    snapshot(
        &after,
        "after",
        vec![
            resource("a", "fixture", json!({})),
            resource("b", "fixture", json!({})),
        ],
        vec![],
        ScanStatus::Complete,
    )
    .await?;
    let limited = diff::report(&before, &after, "before", "after", 100, 1).await?;
    assert_eq!(limited["resources"]["counts"]["examined"], 1);
    assert_eq!(limited["resources"]["comparison_truncated"], true);
    assert_eq!(limited["status"], "partial");
    let exact = diff::report(&before, &after, "before", "after", 100, 2).await?;
    assert_eq!(exact["resources"]["comparison_truncated"], false);
    before.close().await;
    after.close().await;
    Ok(())
}

#[tokio::test]
async fn diff_rejects_running_snapshots_and_invalid_limits() -> Result<()> {
    let before = Store::open(":memory:").await?;
    let after = Store::open(":memory:").await?;
    before.begin_scan("before", 1).await?;
    snapshot(&after, "after", vec![], vec![], ScanStatus::Complete).await?;
    assert!(
        diff::report(&before, &after, "before", "after", 100, 100)
            .await
            .is_err()
    );
    assert!(
        diff::report(&before, &after, "before", "after", 0, 100)
            .await
            .is_err()
    );
    assert!(
        diff::report(&before, &after, "before", "after", 100, 0)
            .await
            .is_err()
    );
    assert!(
        diff::report(&before, &after, "before", "after", 100, 1_000_001)
            .await
            .is_err()
    );
    before.close().await;
    after.close().await;
    Ok(())
}

#[tokio::test]
async fn public_diff_uses_two_readers_without_one_pool_deadlock() -> Result<()> {
    let path = std::env::temp_dir().join(format!(
        "inventory-analysis-{}.sqlite",
        uuid::Uuid::new_v4()
    ));
    let store = Store::open(&path).await?;
    snapshot(
        &store,
        "before",
        vec![resource("a", "fixture", json!({"x":1}))],
        vec![],
        ScanStatus::Complete,
    )
    .await?;
    snapshot(
        &store,
        "after",
        vec![resource("a", "fixture", json!({"x":2}))],
        vec![],
        ScanStatus::Complete,
    )
    .await?;
    store.close().await;
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(10),
        super::diff(path.clone(), "before".into(), "after".into(), 100, 100),
    )
    .await??;
    assert_eq!(result["resources"]["counts"]["changed"], 1);
    std::fs::remove_file(&path)?;
    Ok(())
}
