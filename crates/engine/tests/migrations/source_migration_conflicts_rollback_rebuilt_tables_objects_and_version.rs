use super::*;

#[tokio::test]
pub(super) async fn source_migration_conflicts_rollback_rebuilt_tables_objects_and_version()
-> TestResult {
    for conflict in [
        "CREATE TABLE new_scope_checkpoints(owner_data TEXT); INSERT INTO new_scope_checkpoints VALUES('keep');",
        "CREATE TABLE aws_cloudcontrol_resource(owner_data TEXT); INSERT INTO aws_cloudcontrol_resource VALUES('keep');",
        "CREATE VIEW aws_ecs_task_definition AS SELECT owner_data FROM user_owned;",
    ] {
        let directory = tempfile::tempdir()?;
        let path = directory.path().join("source-conflict.sqlite");
        populated_legacy(&path, 2).await?;
        let mut connection = connect(&path).await?;
        sqlx::raw_sql("CREATE TABLE user_owned(owner_data TEXT); INSERT INTO user_owned VALUES('preserve-me');
            CREATE INDEX user_index ON resource_observations(name);
            CREATE VIEW user_view AS SELECT resource_id FROM resource_observations;
            CREATE TRIGGER user_trigger AFTER INSERT ON user_owned BEGIN SELECT count(*) FROM edge_observations; END;")
            .execute(&mut connection).await?;
        sqlx::raw_sql(conflict).execute(&mut connection).await?;
        let data_before = inventory_snapshot(&mut connection).await?;
        let schema_before = schema_snapshot(&mut connection).await?;
        connection.close().await?;
        assert!(matches!(Store::open(&path).await, Err(StoreError::Sqlx(_))));
        let mut connection = connect(&path).await?;
        assert_eq!(inventory_snapshot(&mut connection).await?, data_before);
        assert_eq!(schema_snapshot(&mut connection).await?, schema_before);
        let version: i64 = sqlx::query_scalar("PRAGMA user_version")
            .fetch_one(&mut connection)
            .await?;
        assert_eq!(version, 2);
        let marker: String = sqlx::query_scalar("SELECT owner_data FROM user_owned")
            .fetch_one(&mut connection)
            .await?;
        assert_eq!(marker, "preserve-me");
        let priority: i64 = sqlx::query_scalar(
            "SELECT DISTINCT source_priority FROM resource_observations WHERE source='direct'",
        )
        .fetch_one(&mut connection)
        .await?;
        assert_eq!(priority, 3);
        connection.close().await?;
    }
    Ok(())
}
