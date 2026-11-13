//! Behavior: metadata_heavy_resources_split_by_bytes_without_losing_rows.
use super::*;

#[tokio::test]
pub(super) async fn metadata_heavy_resources_split_by_bytes_without_losing_rows() -> TestResult {
    let folder = tempfile::tempdir()?;
    // Twelve ordinary 256 KiB property values exceed one graph batch while
    // remaining well below the per-resource and per-page inventory limits.
    let metadata = "x".repeat(262_144);
    let store = frozen_store(&folder.path().join("inventory.sqlite"), 12, &metadata).await?;
    let output = folder.path().join("large.jsonl");
    let summary = graph::export_snapshot(&store, SCAN, "large_metadata", &output).await?;
    let batches = read_batches(&output).await?;
    let resources = batches
        .iter()
        .filter(|batch| batch.stage == "resources")
        .collect::<Vec<_>>();
    assert!(resources.len() >= 2);
    assert_eq!(resources.iter().map(|batch| batch.rows).sum::<u64>(), 12);
    assert!(
        batches
            .iter()
            .all(|batch| batch.rows <= 256 && batch.query.len() <= QUERY_LIMIT)
    );
    for index in 0..12 {
        let field = format!("id:\"{}\"", resource_id(index));
        assert_eq!(
            resources
                .iter()
                .filter(|batch| batch.query.contains(&field))
                .count(),
            1
        );
    }
    assert_eq!(summary.resources, 12);
    assert_eq!(summary.edges, 1);
    assert_eq!(u64::try_from(batches.len())?, summary.batches);
    store.close().await;
    Ok(())
}
