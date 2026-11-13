//! Behavior: graph_checkpoints_are_monotonic_and_require_a_frozen_scan.
use super::*;

#[tokio::test]
pub(super) async fn graph_checkpoints_are_monotonic_and_require_a_frozen_scan() -> TestResult {
    let store = Store::open(":memory:").await?;
    store.begin_scan("one", 1).await?;
    let mut checkpoint = GraphExportCheckpoint {
        scan_id: "one".to_owned(),
        target: "graph".to_owned(),
        snapshot_id: "snapshot-a".to_owned(),
        resources_exported: 1,
        edges_exported: 0,
        complete: false,
    };
    assert!(store.save_graph_checkpoint(&checkpoint).await.is_err());
    store.finish_scan("one", ScanStatus::Partial, 2).await?;
    store.save_graph_checkpoint(&checkpoint).await?;
    store.save_graph_checkpoint(&checkpoint).await?;
    assert_eq!(
        store.graph_checkpoint("one", "graph", "snapshot-a").await?,
        Some(checkpoint.clone())
    );
    checkpoint.resources_exported = 0;
    assert!(store.save_graph_checkpoint(&checkpoint).await.is_err());
    checkpoint.resources_exported = 2;
    checkpoint.complete = true;
    store.save_graph_checkpoint(&checkpoint).await?;
    checkpoint.complete = false;
    assert!(store.save_graph_checkpoint(&checkpoint).await.is_err());
    Ok(())
}
