//! discovery options reject unbounded or meaningless requests behavior.
use super::*;

#[test]
fn discovery_options_reject_unbounded_or_meaningless_requests() -> Result<(), AwsScanError> {
    options().validate()?;
    let mut value = options();
    value.organization = false;
    assert!(value.validate().is_err());
    value.hierarchy = false;
    value.config_aggregators = false;
    value.resource_explorer = false;
    assert!(value.validate().is_err());
    value = options();
    value.max_pages = 10_001;
    assert!(value.validate().is_err());
    value = options();
    value.region = "us-east-1\n".into();
    assert!(value.validate().is_err());
    value = options();
    value.profile = Some("".into());
    assert!(value.validate().is_err());
    Ok(())
}

#[test]
fn role_arn_construction_preserves_partition_and_nested_role_path() -> Result<(), AwsScanError> {
    assert_eq!(
        account_role_arn("aws-us-gov", "123456789012", "service-role/Audit")?,
        "arn:aws-us-gov:iam::123456789012:role/service-role/Audit"
    );
    for invalid_path in [
        "", "/Audit", "Audit/", "a//b", "../Audit", "a/../b", "a/*", "a?b", "a%2fb", "a\nb",
    ] {
        assert!(account_role_arn("aws", "123456789012", invalid_path).is_err());
    }
    assert!(account_role_arn("aws", "123", "Audit").is_err());
    assert!(account_role_arn("aws:iam", "123456789012", "Audit").is_err());
    for partition in ["aws-invalid-", "aws--invalid", "awsinvalid"] {
        assert!(account_role_arn(partition, "123456789012", "Audit").is_err());
    }
    Ok(())
}

#[test]
fn role_options_reject_wrong_service_region_and_session_bounds() -> Result<(), AwsScanError> {
    role_options().validate()?;
    for arn in [
        "arn:aws:sts::123456789012:role/Audit",
        "arn:aws:iam:us-east-1:123456789012:role/Audit",
        "arn:aws:iam::123456789012:user/Audit",
        "arn:aws:iam::123456789012:role/",
    ] {
        let mut options = role_options();
        options.role_arn = arn.into();
        assert!(options.validate().is_err());
    }
    let mut options = role_options();
    options.session_name = "x".into();
    assert!(options.validate().is_err());
    options = role_options();
    options.duration_seconds = 3_601;
    assert!(options.validate().is_err());
    options = role_options();
    options.external_id = Some("bad id".into());
    assert!(options.validate().is_err());
    Ok(())
}

#[test]
fn external_id_is_redacted_from_debug_output() {
    let mut options = role_options();
    options.external_id = Some("fixture-secret-never-log".into());
    let debug = format!("{options:?}");
    assert!(!debug.contains("fixture-secret-never-log"));
    assert!(debug.contains("external_id_present: true"));
}

#[test]
fn credential_provider_timeouts_preserve_stricter_caller_bounds() {
    let existing = TimeoutConfig::builder()
        .operation_timeout(Duration::from_secs(3))
        .operation_attempt_timeout(Duration::from_secs(120))
        .connect_timeout(Duration::from_secs(2))
        .build();
    let bounded = credential_timeouts(Some(&existing));
    assert_eq!(bounded.operation_timeout(), Some(Duration::from_secs(3)));
    assert_eq!(
        bounded.operation_attempt_timeout(),
        Some(CONFIGURATION_TIMEOUT)
    );
    assert_eq!(bounded.connect_timeout(), Some(Duration::from_secs(2)));
    assert_eq!(bounded.read_timeout(), Some(CONFIGURATION_TIMEOUT));
    assert_eq!(
        credential_timeouts(None).operation_timeout(),
        Some(CONFIGURATION_TIMEOUT)
    );
}

#[tokio::test]
async fn precancelled_discovery_stops_before_configuration_loading()
-> Result<(), Box<dyn std::error::Error>> {
    let executor = Arc::new(ApiExecutor::new(ExecutorConfig::default())?);
    executor.cancel();
    assert!(matches!(
        discover(options(), Arc::clone(&executor)).await,
        Err(AwsScanError::Api(ApiError::Cancelled))
    ));
    assert_eq!(executor.stats().attempts, 0);
    Ok(())
}

#[tokio::test]
async fn role_configuration_honors_precancellation_and_requires_loaded_base()
-> Result<(), Box<dyn std::error::Error>> {
    let executor = ApiExecutor::new(ExecutorConfig::default())?;
    assert!(matches!(
        assume_role_config(&config(), &role_options(), &executor).await,
        Err(AwsScanError::InvalidOptions(_))
    ));
    executor.cancel();
    assert!(matches!(
        assume_role_config(&config(), &role_options(), &executor).await,
        Err(AwsScanError::Api(ApiError::Cancelled))
    ));
    assert_eq!(executor.stats().attempts, 0);
    Ok(())
}

#[tokio::test]
async fn role_configuration_is_lazy_and_keeps_sdk_attempts_at_one()
-> Result<(), Box<dyn std::error::Error>> {
    let executor = ApiExecutor::new(ExecutorConfig::default())?;
    let base = config()
        .to_builder()
        .time_source(FixedTime)
        .credentials_provider(SharedCredentialsProvider::new(Credentials::new(
            "fixture-access",
            "fixture-secret",
            None,
            None,
            "fixture",
        )))
        .retry_config(RetryConfig::standard().with_max_attempts(5))
        .build();
    let assumed = assume_role_config(&base, &role_options(), &executor).await?;
    assert!(assumed.credentials_provider().is_some());
    assert_eq!(
        assumed.retry_config().map(|retry| retry.max_attempts()),
        Some(1)
    );
    assert_eq!(executor.stats().attempts, 0);
    Ok(())
}

#[tokio::test]
async fn organizations_empty_page_with_token_is_not_terminal()
-> Result<(), Box<dyn std::error::Error>> {
    let config = config();
    let executor = ApiExecutor::new(ExecutorConfig::default())?;
    let client = client(&config, &executor);
    let calls = AtomicUsize::new(0);
    let listing = pages::collect(&client, "fixture", None, 10, |token| {
        let call = calls.fetch_add(1, Ordering::SeqCst);
        async move {
            match call {
                0 => {
                    assert!(token.is_none());
                    Ok(pages::page(vec![], Some("next".into())))
                }
                1 => {
                    assert_eq!(token.as_deref(), Some("next"));
                    Ok(pages::page(vec![Ok("account".to_owned())], None))
                }
                _ => Err(data_error("unexpected request")),
            }
        }
    })
    .await;
    assert_eq!(calls.load(Ordering::SeqCst), 2);
    assert_eq!(listing.items, vec!["account"]);
    assert_eq!(listing.coverage.pages, 2);
    assert!(listing.coverage.complete);
    Ok(())
}
