use super::*;

#[tokio::test]
pub(super) async fn query_single_statement_check_understands_quotes_and_comments() -> TestResult {
    let store = Store::open(":memory:").await?;
    for forbidden in [
        "SELECT 1; DELETE FROM scans",
        "SELECT 1; /* hidden */ SELECT 2",
        "SELECT 1; -- hidden\nSELECT 2",
        "SELECT ';'; SELECT 2",
        "SELECT 1;;",
        "SELECT 1 /* unterminated",
        "SELECT 'unterminated",
    ] {
        assert!(
            store.query(forbidden, 10).await.is_err(),
            "accepted SQL: {forbidden}"
        );
    }
    assert_eq!(
        store
            .query(
                "SELECT '; DROP TABLE scans; --' AS value; /* harmless */",
                10
            )
            .await?,
        json!([{"value":"; DROP TABLE scans; --"}])
    );
    assert_eq!(
        store
            .query(
                "SELECT 'it''s; quoted' AS \"column;name\" -- comment;\n",
                10
            )
            .await?,
        json!([{"column;name":"it's; quoted"}])
    );
    assert_eq!(
        store
            .query("SELECT 1 AS [bracket;name], 2 AS `backtick;name`", 10)
            .await?,
        json!([{"bracket;name":1, "backtick;name":2}])
    );
    Ok(())
}
