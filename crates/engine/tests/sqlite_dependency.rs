//! Deployment contracts for the SQLite library actually used by the store.
use inventory_engine::store::Store;
use serde_json::json;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[tokio::test]
async fn sqlite_runtime_includes_the_wal_page_size_security_fix() -> TestResult {
    // Given: the production store and its linked SQLite implementation.
    let store = Store::open(":memory:").await?;
    // When: query the running library rather than the system sqlite3 executable.
    let rows = store.query("SELECT sqlite_version() AS version", 1).await?;
    let text = rows
        .get(0)
        .and_then(|row| row.get("version"))
        .and_then(serde_json::Value::as_str)
        .ok_or("SQLite did not return its runtime version")?;
    let version = text
        .split('.')
        .map(str::parse::<u32>)
        .collect::<Result<Vec<_>, _>>()?;
    // Then: the runtime includes the patched SQLite release, not an old bundle.
    assert_eq!(version.len(), 3);
    assert!(version.as_slice() >= [3, 54, 1].as_slice(), "{text}");
    store.close().await;
    Ok(())
}

#[tokio::test]
async fn sqlite_dependency_upgrade_preserves_native_allocation_limits() -> TestResult {
    // Given: the production store using the configured bundled SQLite build.
    let store = Store::open(":memory:").await?;
    // When: inspect the actual native compile options through the public API.
    let rows = store
        .query(
            "SELECT sqlite_compileoption_used('MAX_LENGTH=8388608') AS length_limit, \
             sqlite_compileoption_used('MAX_SQL_LENGTH=65536') AS sql_limit, \
             sqlite_compileoption_used('MAX_COLUMN=256') AS column_limit, \
             sqlite_compileoption_used('MAX_EXPR_DEPTH=128') AS depth_limit",
            1,
        )
        .await?;
    // Then: updating SQLite has not discarded the pre-allocation safety bounds.
    assert_eq!(
        rows,
        json!([{"length_limit": 1, "sql_limit": 1, "column_limit": 1, "depth_limit": 1}])
    );
    store.close().await;
    Ok(())
}
