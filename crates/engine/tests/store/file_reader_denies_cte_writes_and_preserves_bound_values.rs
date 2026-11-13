//! Behavior: file_reader_denies_cte_writes_and_preserves_bound_values.
use super::*;

#[tokio::test]
pub(super) async fn file_reader_denies_cte_writes_and_preserves_bound_values() -> TestResult {
    let directory = tempfile::tempdir()?;
    let store = Store::open(directory.path().join("readonly.sqlite")).await?;
    store.begin_scan("quoted ' value", 1).await?;
    assert!(
        store
            .query(
                "WITH x AS (SELECT 1) DELETE FROM scans RETURNING scan_id",
                10
            )
            .await
            .is_err()
    );
    assert!(
        store
            .query("SELECT 1; ATTACH ':memory:' AS injected", 10)
            .await
            .is_err()
    );
    assert_eq!(
        store
            .query("/* prefix */ SELECT scan_id FROM scans; -- suffix", 10)
            .await?,
        json!([{"scan_id":"quoted ' value"}])
    );
    store.close().await;
    Ok(())
}
