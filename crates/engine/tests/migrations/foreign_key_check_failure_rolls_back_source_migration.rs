use super::*;

#[tokio::test]
pub(super) async fn foreign_key_check_failure_rolls_back_source_migration() -> TestResult {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("broken-reference.sqlite");
    populated_legacy(&path, 2).await?;
    let mut connection = connect(&path).await?;
    sqlx::raw_sql("CREATE TABLE user_orphan(parent TEXT REFERENCES scans(scan_id));
        PRAGMA foreign_keys=OFF; INSERT INTO user_orphan VALUES('missing-parent'); PRAGMA foreign_keys=ON;")
        .execute(&mut connection).await?;
    let data_before = inventory_snapshot(&mut connection).await?;
    let schema_before = schema_snapshot(&mut connection).await?;
    connection.close().await?;
    assert!(matches!(
        Store::open(&path).await,
        Err(StoreError::Conflict("migration foreign key check failed"))
    ));
    let mut connection = connect(&path).await?;
    assert_eq!(inventory_snapshot(&mut connection).await?, data_before);
    assert_eq!(schema_snapshot(&mut connection).await?, schema_before);
    let version: i64 = sqlx::query_scalar("PRAGMA user_version")
        .fetch_one(&mut connection)
        .await?;
    assert_eq!(version, 2);
    let orphans: i64 = sqlx::query_scalar("SELECT count(*) FROM pragma_foreign_key_check")
        .fetch_one(&mut connection)
        .await?;
    assert_eq!(orphans, 1);
    connection.close().await?;
    Ok(())
}
