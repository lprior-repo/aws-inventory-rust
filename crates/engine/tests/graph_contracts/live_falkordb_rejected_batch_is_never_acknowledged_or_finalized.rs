//! Behavior: live_falkordb_rejected_batch_is_never_acknowledged_or_finalized.
use super::*;

#[tokio::test]
#[ignore = "requires explicit FALKORDB_TEST_URL; creates and removes an isolated test graph"]
pub(super) async fn live_falkordb_rejected_batch_is_never_acknowledged_or_finalized() -> TestResult
{
    tokio::time::timeout(Duration::from_secs(10), server_rejection_case())
        .await
        .map_err(|_| "engine graph integration test exceeded its 10-second deadline")?
}
