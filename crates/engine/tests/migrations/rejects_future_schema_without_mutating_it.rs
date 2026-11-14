use super::*;

#[tokio::test]
pub(super) async fn rejects_future_schema_without_mutating_it() -> TestResult {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("future.sqlite");
    let mut connection = connect(&path).await?;
    sqlx::raw_sql("CREATE TABLE sentinel(value TEXT); INSERT INTO sentinel VALUES('future'); PRAGMA user_version=99;")
        .execute(&mut connection).await?;
    let schema_before = schema_snapshot(&mut connection).await?;
    connection.close().await?;
    assert!(matches!(
        Store::open(&path).await,
        Err(StoreError::SchemaVersion(99))
    ));
    let mut connection = connect(&path).await?;
    let version: i64 = sqlx::query_scalar("PRAGMA user_version")
        .fetch_one(&mut connection)
        .await?;
    let marker: String = sqlx::query_scalar("SELECT value FROM sentinel")
        .fetch_one(&mut connection)
        .await?;
    assert_eq!((version, marker.as_str()), (99, "future"));
    assert_eq!(schema_snapshot(&mut connection).await?, schema_before);
    connection.close().await?;
    Ok(())
}
