use super::*;
use inventory_core::{catalog::runtime::RuntimeCatalog, inventory::*};
use inventory_engine::inventory::{BinaryStore, SchemaRegistry, SweepId, WorkSpec};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn scope(service: &str, operation: &str) -> TestResult<ScopeKey> {
    Ok(ScopeKey::new(
        AccountId::parse("123456789012")?,
        Region::Global,
        ServiceId::parse(service)?,
        OperationId::parse(operation)?,
        None,
    )?)
}

async fn seed(store: &BinaryStore, sweep: &SweepId, scope: ScopeKey) -> TestResult<WorkLease> {
    store
        .seed_work(WorkSpec::new(sweep.clone(), scope, None, Vec::new())?)
        .await?;
    Ok(store
        .claim(sweep, now_ms()?, 60_000)
        .await?
        .ok_or("work absent")?)
}

async fn fixture() -> TestResult<(tempfile::TempDir, BinaryStore, SweepId, WorkLease)> {
    fixture_for(
        "organizations",
        "2016-11-28",
        &["ListAccounts"],
        "ListAccounts",
    )
    .await
}

async fn fixture_for(
    service: &str,
    date: &str,
    operations: &[&str],
    operation: &str,
) -> TestResult<(tempfile::TempDir, BinaryStore, SweepId, WorkLease)> {
    let compiled = crate::consumer_fixture::compile(service, date, operations)?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let directory = tempfile::tempdir()?;
    let registry = SchemaRegistry::new(catalog.raw_bytes(), catalog.schemas().to_vec())?;
    let store = BinaryStore::create(&directory.path().join("absence.sqlite"), registry).await?;
    let sweep = SweepId::new("absence-settlement".to_owned())?;
    let lease = seed(&store, &sweep, scope(service, operation)?).await?;
    Ok((directory, store, sweep, lease))
}

#[tokio::test]
async fn given_absent_feature_when_settled_then_durable_scope_completes_without_resources()
-> TestResult {
    let (_directory, store, sweep, lease) = fixture().await?;
    let result = finish_error(
        &store,
        lease,
        SweepError::Absent("AWSOrganizationsNotInUseException".into()),
        None,
    )
    .await?;
    assert_eq!(result.outcomes.complete, 1);
    assert_eq!(result.outcomes.failed, 0);
    assert_eq!(result.rows, 0);
    let stored = store.sweep_summary(&sweep).await?;
    assert_eq!(stored.complete_scopes, 1);
    assert_eq!(stored.work_pending, 0);
    Ok(())
}

#[tokio::test]
async fn given_access_denied_when_settled_then_scope_is_not_reported_as_empty() -> TestResult {
    let (directory, store, sweep, lease) = fixture_for(
        "iam",
        "2010-05-08",
        &["ListUsers", "GetLoginProfile"],
        "GetLoginProfile",
    )
    .await?;
    let error = SweepError::Api(inventory_engine::limiter::ApiError::Denied(
        "AccessDeniedException".into(),
    ));
    let result = finish_error(&store, lease, error, None).await?;
    assert_eq!(result.outcomes.inaccessible, 1);
    assert_eq!(result.outcomes.complete, 0);
    let stored = store.sweep_summary(&sweep).await?;
    assert_eq!(stored.complete_scopes, 0);
    assert_eq!(stored.work_pending, 0);
    assert_eq!(
        stored_reasons(&directory).await?,
        [Some(
            "denied: API executor error: access denied: AccessDeniedException".to_owned()
        )]
    );
    Ok(())
}

#[tokio::test]
async fn given_missing_login_profile_when_settled_then_only_its_scope_completes() -> TestResult {
    let (directory, store, sweep, lease) = fixture_for(
        "iam",
        "2010-05-08",
        &["ListUsers", "GetLoginProfile"],
        "GetLoginProfile",
    )
    .await?;
    let result = finish_error(
        &store,
        lease,
        SweepError::Absent("NoSuchEntity".into()),
        None,
    )
    .await?;
    assert_eq!(result.outcomes.complete, 1);
    assert_eq!(result.outcomes.failed, 0);
    assert_eq!(result.rows, 0);
    let stored = store.sweep_summary(&sweep).await?;
    assert_eq!(stored.complete_scopes, 1);
    assert_eq!(stored.work_pending, 0);
    assert_eq!(stored_reasons(&directory).await?, [None]);
    Ok(())
}

#[tokio::test]
async fn given_an_undecodable_page_when_settled_then_the_reason_names_the_response_shape()
-> TestResult {
    let (directory, store, _sweep, lease) = fixture().await?;
    let error = SweepError::Decode(aws_wire::response::DecodeError::Invalid("bad member"));
    let expected = format!("decode: ListAccountsResponse: {error}");

    let result = finish_error(&store, lease, error, Some("ListAccountsResponse")).await?;

    assert_eq!(result.outcomes.failed, 1);
    assert_eq!(stored_reasons(&directory).await?, [Some(expected)]);
    Ok(())
}

/// The reason column of every stored scope outcome, read through SQL.
async fn stored_reasons(directory: &tempfile::TempDir) -> TestResult<Vec<Option<String>>> {
    let pool = sqlx::SqlitePool::connect(&format!(
        "sqlite://{}",
        directory.path().join("absence.sqlite").display()
    ))
    .await?;
    let reasons = sqlx::query_scalar("SELECT detail FROM scope_outcomes")
        .fetch_all(&pool)
        .await?;
    pool.close().await;
    Ok(reasons)
}
