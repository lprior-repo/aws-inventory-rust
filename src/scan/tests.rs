//! Offline account-target and cancellation regressions. None of these tests
//! loads AWS configuration or asks a credentials provider for credentials.

use std::time::Duration;

use clap::Parser;
use inventory_aws::discovery::{DiscoveryCoverage, OrganizationAccount};

use super::*;

fn scan_options() -> Result<ScanArgs> {
    let cli = crate::cli::Cli::try_parse_from([
        "aws-inventory",
        "scan",
        "--service",
        "ec2",
        "--region",
        "us-east-1",
    ])?;
    match cli.command {
        crate::cli::Command::Scan(options) => Ok(options),
        _ => anyhow::bail!("fixture did not parse as a scan"),
    }
}

fn account(id: &str, active: bool) -> OrganizationAccount {
    OrganizationAccount {
        id: id.into(),
        arn: None,
        name: None,
        state: Some(if active { "ACTIVE" } else { "SUSPENDED" }.into()),
        eligible_for_scan: active,
        joined_method: None,
        joined_at: None,
    }
}

fn organization(accounts: Vec<OrganizationAccount>, complete: bool) -> OrganizationDiscovery {
    OrganizationDiscovery {
        accounts,
        account_coverage: DiscoveryCoverage {
            complete,
            ..DiscoveryCoverage::default()
        },
        hierarchy_requested: false,
        roots: Vec::new(),
        organizational_units: Vec::new(),
        memberships: Vec::new(),
        hierarchy_coverage: None,
    }
}

fn role(account: &str) -> Result<AssumeRoleOptions> {
    role_options(
        &AccountArgs::default(),
        discovery::account_role_arn("aws", account, "audit/Inventory")?,
        None,
    )
}

#[test]
fn partial_organization_listing_never_becomes_an_implicit_subset_scan() {
    let report = organization(vec![account("123456789012", true)], false);
    assert!(select_accounts(&report, &AccountArgs::default()).is_err());
    let narrowed = AccountArgs {
        organization_role: Some("audit/Inventory".into()),
        account_id: vec!["123456789012".into()],
        ..AccountArgs::default()
    };
    assert!(select_accounts(&report, &narrowed).is_err());
}

#[test]
fn only_active_observations_become_ordered_unique_targets() -> Result<()> {
    let report = organization(
        vec![
            account("323456789012", true),
            account("223456789012", false),
            account("123456789012", true),
        ],
        true,
    );
    assert_eq!(
        select_accounts(&report, &AccountArgs::default())?,
        vec!["123456789012", "323456789012"]
    );
    let selected = AccountArgs {
        organization_role: Some("audit/Inventory".into()),
        account_id: vec!["323456789012".into(), "323456789012".into()],
        ..AccountArgs::default()
    };
    assert_eq!(select_accounts(&report, &selected)?, vec!["323456789012"]);
    Ok(())
}

#[test]
fn explicit_inactive_missing_and_empty_selections_fail_before_job_creation() {
    let report = organization(
        vec![
            account("123456789012", true),
            account("223456789012", false),
        ],
        true,
    );
    for id in ["223456789012", "923456789012"] {
        let selected = AccountArgs {
            organization_role: Some("audit/Inventory".into()),
            account_id: vec![id.into()],
            ..AccountArgs::default()
        };
        assert!(select_accounts(&report, &selected).is_err());
    }
    assert!(select_accounts(&organization(vec![], true), &AccountArgs::default()).is_err());
    assert!(
        select_accounts(
            &organization(vec![account("223456789012", false)], true),
            &AccountArgs::default(),
        )
        .is_err()
    );
}

#[test]
fn account_ceiling_applies_after_explicit_filtering() -> Result<()> {
    let report = organization(
        vec![account("123456789012", true), account("223456789012", true)],
        true,
    );
    let mut options = AccountArgs {
        organization_role: Some("audit/Inventory".into()),
        max_accounts: 1,
        ..AccountArgs::default()
    };
    assert!(select_accounts(&report, &options).is_err());
    options.account_id.push("223456789012".into());
    assert_eq!(select_accounts(&report, &options)?, vec!["223456789012"]);
    Ok(())
}

#[test]
fn role_targets_share_a_source_profile_but_keep_separate_scan_identities() -> Result<()> {
    let options = scan_options()?;
    let first = job(
        &options,
        Some("source-fixture".into()),
        Some(role("123456789012")?),
    );
    let second = job(
        &options,
        Some("source-fixture".into()),
        Some(role("223456789012")?),
    );
    assert_ne!(first.scan_id, second.scan_id);
    assert_eq!(first.options.profile, second.options.profile);
    assert_eq!(
        first
            .options
            .assume_role
            .as_ref()
            .map(|role| role.role_arn.as_str()),
        Some("arn:aws:iam::123456789012:role/audit/Inventory")
    );
    assert_eq!(
        second
            .options
            .assume_role
            .as_ref()
            .map(|role| role.role_arn.as_str()),
        Some("arn:aws:iam::223456789012:role/audit/Inventory")
    );
    Ok(())
}

#[test]
fn startup_failure_attributes_the_requested_account_without_inventing_identity() -> Result<()> {
    let known = startup_failure("known", Some("123456789012"), "assumption denied");
    let unresolved = startup_failure("unknown", None, "credentials unavailable");
    match (known.event, unresolved.event) {
        (CollectionEvent::Failure(known), CollectionEvent::Failure(unresolved)) => {
            assert_eq!(known.scope.account_id, "123456789012");
            assert_eq!(unresolved.scope.account_id, "unknown");
            assert_eq!(known.scope.collector, "scan_orchestration");
            assert_eq!(known.scope.region, "global");
            assert_eq!(known.category, "scan_failed");
            Ok(())
        }
        _ => anyhow::bail!("startup failure was not a failure event"),
    }
}

#[tokio::test]
async fn known_target_failure_remains_attributed_and_partial_in_sqlite() -> Result<()> {
    let store = Store::open(":memory:").await?;
    let result = async {
        store.begin_scan("role-denied", 1).await?;
        let failure = startup_failure(
            "role-denied",
            Some("123456789012"),
            "synthetic role-assumption denial",
        );
        demo::persist_event(&store, &failure.scan_id, failure.event).await?;
        store
            .finish_scan("role-denied", ScanStatus::Complete, 2)
            .await?;
        assert_eq!(store.scan_status("role-denied").await?, ScanStatus::Partial);
        let rows = store
            .query(
                "SELECT account_id,collector,failed FROM scope_checkpoints WHERE scan_id='role-denied'",
                10,
            )
            .await?;
        let rows = rows
            .as_array()
            .context("query must return rows as an array")?;
        assert_eq!(rows.len(), 1);
        let row = rows.first().context("missing failed account scope")?;
        assert_eq!(row.get("account_id").and_then(Value::as_str), Some("123456789012"));
        assert_eq!(row.get("collector").and_then(Value::as_str), Some("scan_orchestration"));
        assert_eq!(row.get("failed").and_then(Value::as_i64), Some(1));
        Ok::<(), anyhow::Error>(())
    }
    .await;
    store.close().await;
    result
}

#[tokio::test]
async fn precancelled_queued_roles_drain_without_loading_credentials() -> Result<()> {
    let options = scan_options()?;
    let executor = Arc::new(ApiExecutor::new(ExecutorConfig::default())?);
    executor.cancel();
    let jobs = (0..PROFILE_CONCURRENCY.saturating_add(5))
        .map(|_| {
            Ok(job(
                &options,
                Some("fixture-profile-must-not-be-loaded".into()),
                Some(role("123456789012")?),
            ))
        })
        .collect::<Result<Vec<_>>>()?;
    let expected = jobs.len();
    let (sender, mut receiver) = mpsc::channel(WRITER_QUEUE);
    let outcomes = tokio::time::timeout(
        Duration::from_secs(5),
        collect_jobs(jobs, Arc::clone(&executor), sender),
    )
    .await?;
    assert_eq!(outcomes.len(), expected);
    assert!(outcomes.iter().all(|outcome| outcome.error.is_some()));
    assert!(
        outcomes
            .iter()
            .all(|outcome| outcome.target_account.as_deref() == Some("123456789012"))
    );
    assert_eq!(executor.stats().attempts, 0);
    assert!(matches!(
        receiver.try_recv(),
        Err(mpsc::error::TryRecvError::Disconnected)
    ));
    Ok(())
}

#[tokio::test]
async fn precancelled_run_creates_no_database_and_performs_no_api_attempt() -> Result<()> {
    let mut options = scan_options()?;
    options.db = std::env::temp_dir().join(format!(
        "aws-inventory-precancel-{}.sqlite",
        uuid::Uuid::new_v4()
    ));
    let database = options.db.clone();
    options.accounts.role_arn = Some("arn:aws:iam::123456789012:role/audit/Inventory".into());
    options.accounts.external_id_env = Some("INVENTORY_FIXTURE_MUST_NOT_READ_SECRET".into());
    let executor = Arc::new(ApiExecutor::new(ExecutorConfig::default())?);
    executor.cancel();
    let result = tokio::time::timeout(
        Duration::from_secs(5),
        run_with_executor(options, Arc::clone(&executor)),
    )
    .await?;
    let error = result
        .err()
        .context("pre-cancelled scan unexpectedly succeeded")?;
    assert!(
        error
            .to_string()
            .contains("cancelled before target discovery")
    );
    assert!(!database.exists());
    assert_eq!(executor.stats().attempts, 0);
    Ok(())
}
