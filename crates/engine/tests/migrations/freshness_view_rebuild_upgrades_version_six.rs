//! Behavior: freshness_view_rebuild_upgrades_version_six.
use super::*;

#[tokio::test]
pub(super) async fn freshness_view_rebuild_upgrades_version_six() -> TestResult {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("v6.sqlite");
    let mut connection = connect(&path).await?;
    sqlx::raw_sql(include_str!("../../sql/001_inventory.sql"))
        .execute(&mut connection)
        .await?;
    sqlx::raw_sql(
        "INSERT INTO scans VALUES('old',1,20,'complete');
         INSERT INTO scope_checkpoints
           VALUES('old','direct','111111111111','us-east-1','ec2',NULL,'direct',1,NULL,1,0);
         INSERT INTO scope_checkpoints
           VALUES('old','config','111111111111','us-east-1','config',NULL,'aws_config',1,NULL,1,0);
         INSERT INTO collection_pages VALUES('old','direct',1,zeroblob(32),NULL,1,0);
         INSERT INTO collection_pages VALUES('old','config',1,zeroblob(32),NULL,1,0);
         INSERT INTO resource_observations
           VALUES('old','direct',1,'i-1','111111111111','us-east-1','aws_ec2_instance',
                  'worker','direct',3,10,'{}');
         INSERT INTO resource_observations
           VALUES('old','config',1,'i-1','111111111111','us-east-1','aws_ec2_instance',
                  'worker','aws_config',2,200,'{}');
         PRAGMA user_version=6;",
    )
    .execute(&mut connection)
    .await?;
    connection.close().await?;
    let store = Store::open(&path).await?;
    assert_eq!(
        store
            .query("SELECT user_version AS version FROM pragma_user_version", 1)
            .await?,
        json!([{"version":CURRENT_SCHEMA_VERSION}])
    );
    assert_eq!(
        store
            .query("SELECT source,observed_at FROM latest_resources", 1)
            .await?,
        json!([{"source":"aws_config", "observed_at":200}])
    );
    assert_eq!(
        store
            .query("SELECT count(*) AS total FROM latest_resource_sources", 1)
            .await?,
        json!([{"total":2}])
    );
    assert_eq!(
        store
            .query("SELECT count(*) AS total FROM resource_observations", 1)
            .await?,
        json!([{"total":2}])
    );
    assert_eq!(
        store
            .query("SELECT source,observed_at FROM scan_resources", 1)
            .await?,
        json!([{"source":"direct", "observed_at":10}])
    );
    store.close().await;
    Ok(())
}
