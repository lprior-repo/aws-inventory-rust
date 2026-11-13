//! Behavior: coverage_registry_upgrades_version_seven.
use super::*;

#[tokio::test]
pub(super) async fn coverage_registry_upgrades_version_seven() -> TestResult {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("v7.sqlite");
    let store = Store::open(&path).await?;
    store.close().await;
    let mut connection = connect(&path).await?;
    sqlx::raw_sql(
        "DROP TABLE coverage_registry;
         DROP TABLE accounts;
         DROP VIEW aws_cc_app_config_experiment_definition;
         DROP VIEW aws_cc_eks_access_entry;
         DROP VIEW aws_cc_eks_pod_identity_association;
         PRAGMA user_version=7;",
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
    store
        .register_account("111111111111", "aws", None, 10)
        .await?;
    let accounts = store.list_accounts().await?;
    assert_eq!(accounts.len(), 1);
    let Some(account) = accounts.first() else {
        return Err("upgraded account missing".into());
    };
    assert_eq!(account.account_id, "111111111111");
    store.close().await;
    Ok(())
}
