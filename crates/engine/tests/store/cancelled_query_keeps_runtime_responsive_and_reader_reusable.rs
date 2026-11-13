//! Behavior: cancelled_query_keeps_runtime_responsive_and_reader_reusable.
use super::*;

#[tokio::test]
pub(super) async fn cancelled_query_keeps_runtime_responsive_and_reader_reusable() -> TestResult {
    let directory = tempfile::tempdir()?;
    let store = Store::open(directory.path().join("cancel.sqlite")).await?;
    let query = store.query(
        "WITH RECURSIVE n(x) AS (VALUES(1) UNION ALL SELECT x+1 FROM n) SELECT sum(x) FROM n",
        1,
    );
    let cancelled = tokio::time::timeout(std::time::Duration::from_millis(5), query).await;
    assert!(cancelled.is_err());
    store.begin_scan("writer-still-works", 1).await?;
    let response = store.query("SELECT sqlite_version() AS version", 1).await?;
    assert!(
        response
            .get(0)
            .and_then(|row| row.get("version"))
            .and_then(serde_json::Value::as_str)
            .is_some_and(|version| !version.is_empty())
    );
    assert_eq!(
        store.scan_status("writer-still-works").await?,
        ScanStatus::Running
    );
    store.close().await;
    Ok(())
}
