use super::*;

#[tokio::test]
pub(super) async fn query_caps_rows_columns_output_and_instruction_work() -> TestResult {
    let store = Store::open(":memory:").await?;
    assert!(matches!(
        store.query("SELECT 1 UNION ALL SELECT 2", 1).await,
        Err(StoreError::Limit(_))
    ));
    assert!(store.query("SELECT 1", 0).await.is_err());
    assert!(
        store
            .query("SELECT 1", MAX_QUERY_ROWS.saturating_add(1))
            .await
            .is_err()
    );
    assert!(matches!(
        store
            .query("SELECT printf('%9000000s','a') AS large", 1)
            .await,
        Err(StoreError::Limit("SQLite value bytes"))
    ));
    assert!(matches!(
        store
            .query(
                "SELECT printf('%4500000s','a') AS large UNION ALL SELECT printf('%4500000s','b')",
                2
            )
            .await,
        Err(StoreError::Limit(_))
    ));
    assert!(store.query("SELECT 1 AS same,2 AS same", 10).await.is_err());
    assert!(store.query(
        "WITH RECURSIVE n(x) AS (VALUES(1) UNION ALL SELECT x+1 FROM n) SELECT sum(x) FROM n", 1,
    ).await.is_err());
    store.begin_scan("after-interruption", 1).await?;
    assert_eq!(
        store.query("SELECT upper('ok') AS value", 10).await?,
        json!([{"value":"OK"}])
    );
    Ok(())
}
