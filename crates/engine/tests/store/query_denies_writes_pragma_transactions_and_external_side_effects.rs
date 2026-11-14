use super::*;

#[tokio::test]
pub(super) async fn query_denies_writes_pragma_transactions_and_external_side_effects() -> TestResult
{
    let store = Store::open(":memory:").await?;
    store.begin_scan("one", 1).await?;
    for forbidden in [
        "DELETE FROM scans",
        "UPDATE scans SET status='complete'",
        "DROP TABLE scans",
        "CREATE TABLE injected(x)",
        "CREATE TEMP TABLE injected(x)",
        "BEGIN",
        "COMMIT",
        "PRAGMA query_only=OFF",
        "PRAGMA user_version=9",
        "ATTACH DATABASE ':memory:' AS other",
        "VACUUM",
        "SELECT load_extension('untrusted')",
        "SELECT writefile('/tmp/untrusted', 'x')",
        "WITH n AS (SELECT 1) DELETE FROM scans RETURNING scan_id",
        "SELECT * FROM pragma_query_only('OFF')",
    ] {
        assert!(
            store.query(forbidden, 10).await.is_err(),
            "accepted forbidden SQL: {forbidden}"
        );
    }
    assert_eq!(store.scan_status("one").await?, ScanStatus::Running);
    store.begin_scan("after-denials", 2).await?;
    assert_eq!(
        store
            .query("SELECT count(*) AS total FROM scans", 10)
            .await?,
        json!([{"total":2}])
    );
    Ok(())
}
