use super::*;

#[tokio::test]
pub(super) async fn query_bounds_native_columns_and_decoded_cells() -> TestResult {
    let store = Store::open(":memory:").await?;
    let columns = (0..257)
        .map(|index| format!("1 AS c{index}"))
        .collect::<Vec<_>>()
        .join(",");
    assert!(store.query(&format!("SELECT {columns}"), 1).await.is_err());

    let cell_columns = (0..100)
        .map(|index| format!("x AS c{index}"))
        .collect::<Vec<_>>()
        .join(",");
    let cell_query = |rows| {
        format!(
            "WITH RECURSIVE n(x) AS (VALUES(1) UNION ALL SELECT x+1 FROM n WHERE x<{rows})
             SELECT {cell_columns} FROM n ORDER BY x"
        )
    };
    let at_limit = store.query(&cell_query(1_000), 1_000).await?;
    let rows = at_limit
        .as_array()
        .ok_or(StoreError::InvalidInput("query result must be an array"))?;
    assert_eq!(rows.len(), 1_000);
    assert!(
        rows.iter()
            .all(|row| row.as_object().is_some_and(|columns| columns.len() == 100))
    );
    let over_limit = store.query(&cell_query(1_001), 1_001).await;
    assert!(
        matches!(&over_limit, Err(StoreError::Limit("query cells"))),
        "expected the query-cell limit, received {over_limit:?}"
    );
    for query in [
        "SELECT x'ff' AS blob",
        "SELECT CAST(x'ff' AS TEXT) AS invalid_utf8",
        "SELECT 1e999 AS nonfinite",
    ] {
        assert!(store.query(query, 1).await.is_err());
    }
    store.close().await;
    Ok(())
}
