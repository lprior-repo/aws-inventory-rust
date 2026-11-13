//! Behavior: account onboarding and coverage-registry state are explicit and durable.
use std::error::Error;

use inventory_engine::store::{AccountStatus, CoverageRecord, CoverageState, Store, StoreError};
use serde_json::json;

type TestResult = Result<(), Box<dyn Error>>;

#[tokio::test]
async fn account_and_coverage_registry_round_trip() -> TestResult {
    let directory = tempfile::tempdir()?;
    let store = Store::open(directory.path().join("registry.sqlite")).await?;
    assert_eq!(
        store
            .query("SELECT user_version AS version FROM pragma_user_version", 1)
            .await?,
        json!([{"version":13}])
    );
    store
        .register_account("111111111111", "aws", None, 10)
        .await?;
    store
        .register_account(
            "222222222222",
            "aws",
            Some("arn:aws:iam::222222222222:role/read"),
            11,
        )
        .await?;
    store
        .set_account_status("222222222222", AccountStatus::Suspended, 12)
        .await?;
    let accounts = store.list_accounts().await?;
    assert_eq!(accounts.len(), 2);
    let Some(first) = accounts.first() else {
        return Err("first account missing".into());
    };
    let Some(second) = accounts.get(1) else {
        return Err("second account missing".into());
    };
    assert_eq!(first.account_id, "111111111111");
    assert_eq!(first.status, AccountStatus::Onboarded);
    assert_eq!(second.status, AccountStatus::Suspended);
    assert!(second.role_arn.is_some());

    let record = CoverageRecord {
        account_id: "111111111111".into(),
        region: "us-east-1".into(),
        collector: "ec2".into(),
        state: CoverageState::Complete,
        last_scan_id: Some("scan-1".into()),
        last_attempt_at: Some(20),
        last_success_at: Some(21),
        last_error_category: None,
    };
    store.record_coverage(&record).await?;
    let denied = CoverageRecord {
        state: CoverageState::Denied,
        region: "eu-west-1".into(),
        last_error_category: Some("access_denied".into()),
        ..record.clone()
    };
    store.record_coverage(&denied).await?;
    let rows = store.coverage("111111111111").await?;
    assert_eq!(rows.len(), 2);
    let Some(first) = rows.first() else {
        return Err("first coverage row missing".into());
    };
    let Some(second) = rows.get(1) else {
        return Err("second coverage row missing".into());
    };
    assert_eq!(first, &denied);
    assert_eq!(second, &record);
    let summary = store.coverage_summary().await?;
    assert_eq!(summary.total, 2);
    assert_eq!(summary.by_state.get("complete"), Some(&1));
    assert_eq!(summary.by_state.get("denied"), Some(&1));
    store.close().await;
    Ok(())
}

#[tokio::test]
async fn invalid_registry_rows_are_rejected() -> TestResult {
    let directory = tempfile::tempdir()?;
    let store = Store::open(directory.path().join("registry.sqlite")).await?;
    assert!(matches!(
        store.register_account("", "aws", None, 1).await,
        Err(StoreError::Validation(_))
    ));
    assert!(matches!(
        store
            .set_account_status("999999999999", AccountStatus::Removed, 1)
            .await,
        Err(StoreError::MissingAccount(_))
    ));
    assert!(matches!(
        store
            .record_coverage(&CoverageRecord {
                account_id: "111111111111".into(),
                region: "us-east-1".into(),
                collector: "ec2".into(),
                state: CoverageState::Complete,
                last_scan_id: None,
                last_attempt_at: Some(-1),
                last_success_at: None,
                last_error_category: None,
            })
            .await,
        Err(StoreError::InvalidInput(_))
    ));
    store.close().await;
    Ok(())
}
