use super::*;

#[tokio::test]
pub(super) async fn unsupported_managed_table_extensions_are_rejected_without_losing_values()
-> TestResult {
    for table in ["resource_observations", "edge_observations"] {
        let directory = tempfile::tempdir()?;
        let path = directory.path().join("extended-managed-table.sqlite");
        populated_legacy(&path, 2).await?;
        let mut connection = connect(&path).await?;

        sqlx::raw_sql(AssertSqlSafe(format!(
            "ALTER TABLE {table} ADD COLUMN user_note TEXT;
            UPDATE {table} SET user_note='preserve application-owned value';"
        )))
        .execute(&mut connection)
        .await?;
        let data_before = inventory_snapshot(&mut connection).await?;
        let schema_before = schema_snapshot(&mut connection).await?;
        let notes_before: Vec<String> = sqlx::query_scalar(AssertSqlSafe(format!(
            "SELECT user_note FROM {table} ORDER BY user_note"
        )))
        .fetch_all(&mut connection)
        .await?;
        connection.close().await?;

        assert!(Store::open(&path).await.is_err());
        let mut connection = connect(&path).await?;
        assert_eq!(inventory_snapshot(&mut connection).await?, data_before);
        assert_eq!(schema_snapshot(&mut connection).await?, schema_before);
        let notes_after: Vec<String> = sqlx::query_scalar(AssertSqlSafe(format!(
            "SELECT user_note FROM {table} ORDER BY user_note"
        )))
        .fetch_all(&mut connection)
        .await?;
        assert_eq!(notes_after, notes_before);
        let version: i64 = sqlx::query_scalar("PRAGMA user_version")
            .fetch_one(&mut connection)
            .await?;
        assert_eq!(version, 2);
        connection.close().await?;
    }
    Ok(())
}
