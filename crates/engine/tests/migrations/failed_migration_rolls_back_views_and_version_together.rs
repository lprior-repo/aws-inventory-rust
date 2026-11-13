//! Behavior: failed_migration_rolls_back_views_and_version_together.
use super::*;

#[tokio::test]
pub(super) async fn failed_migration_rolls_back_views_and_version_together() -> TestResult {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("conflict.sqlite");
    legacy_database(&path).await?;
    let mut connection = connect(&path).await?;
    sqlx::query("CREATE TABLE aws_elbv2_target_group (owner_data TEXT)")
        .execute(&mut connection)
        .await?;
    sqlx::query("INSERT INTO aws_elbv2_target_group VALUES('preserve-me')")
        .execute(&mut connection)
        .await?;
    connection.close().await?;
    assert!(matches!(Store::open(&path).await, Err(StoreError::Sqlx(_))));
    let mut connection = connect(&path).await?;
    let version: i64 = sqlx::query_scalar("PRAGMA user_version")
        .fetch_one(&mut connection)
        .await?;
    let views: i64 =
        sqlx::query_scalar("SELECT count(*) FROM sqlite_schema WHERE name='aws_vpc_route_table'")
            .fetch_one(&mut connection)
            .await?;
    let marker: String = sqlx::query_scalar("SELECT owner_data FROM aws_elbv2_target_group")
        .fetch_one(&mut connection)
        .await?;
    let observations: i64 = sqlx::query_scalar("SELECT count(*) FROM resource_observations")
        .fetch_one(&mut connection)
        .await?;
    assert_eq!((version, views, observations), (1, 0, 1));
    assert_eq!(marker, "preserve-me");
    connection.close().await?;
    Ok(())
}
