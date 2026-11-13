//! Behavior: snapshot_export_can_exceed_the_read_only_query_row_cap.
use super::*;

#[tokio::test]
pub(super) async fn snapshot_export_can_exceed_the_read_only_query_row_cap() -> TestResult {
    tokio::time::timeout(Duration::from_secs(30), export_above_query_cap())
        .await
        .map_err(|_| "inventory export streaming test exceeded its 30-second deadline")?
}
