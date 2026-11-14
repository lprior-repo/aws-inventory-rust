use super::*;

#[tokio::test]
pub(super) async fn schema_object_bound_rejects_before_losing_existing_objects() -> TestResult {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("schema-bound.sqlite");
    populated_legacy(&path, 2).await?;
    let mut connection = connect(&path).await?;
    let statements: String = (0..1_024)
        .map(|index| format!("CREATE VIEW user_bound_{index} AS SELECT {index} AS n;\n"))
        .collect();

    sqlx::raw_sql(AssertSqlSafe(statements))
        .execute(&mut connection)
        .await?;
    let schema_before = schema_snapshot(&mut connection).await?;
    connection.close().await?;
    assert!(matches!(
        Store::open(&path).await,
        Err(StoreError::Limit("migration schema objects"))
    ));
    let mut connection = connect(&path).await?;
    assert_eq!(schema_snapshot(&mut connection).await?, schema_before);
    let version: i64 = sqlx::query_scalar("PRAGMA user_version")
        .fetch_one(&mut connection)
        .await?;
    assert_eq!(version, 2);
    let rows: i64 = sqlx::query_scalar("SELECT count(*) FROM resource_observations")
        .fetch_one(&mut connection)
        .await?;
    assert_eq!(rows, 8);
    connection.close().await?;
    Ok(())
}
