use std::sync::atomic::{AtomicUsize, Ordering};

use aws_credential_types::{Credentials, provider::SharedCredentialsProvider};
use aws_types::sdk_config::TimeSource;
use inventory_engine::limiter::ExecutorConfig;

use super::pages::{self, Page};
use super::*;

fn options() -> DiscoveryOptions {
    DiscoveryOptions {
        profile: None,
        region: "us-east-1".into(),
        organization: true,
        hierarchy: true,
        config_aggregators: true,
        resource_explorer: true,
        max_pages: 100,
        page_size: 100,
    }
}

fn config() -> SdkConfig {
    SdkConfig::builder()
        .behavior_version(BehaviorVersion::latest())
        .region(Region::new("us-east-1"))
        .build()
}

fn client<'a>(config: &'a SdkConfig, executor: &'a ApiExecutor) -> DiscoveryClient<'a> {
    DiscoveryClient {
        config,
        executor,
        account: "123456789012",
        partition: "aws",
        region: "us-east-1",
        max_pages: 10,
        page_size: 100,
    }
}

fn role_options() -> AssumeRoleOptions {
    AssumeRoleOptions {
        role_arn: "arn:aws:iam::123456789012:role/audit/Inventory".into(),
        session_name: "inventory-fixture".into(),
        external_id: None,
        duration_seconds: 3_600,
    }
}

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

#[derive(Debug)]
struct FixedTime;
impl TimeSource for FixedTime {
    fn now(&self) -> std::time::SystemTime {
        std::time::UNIX_EPOCH
    }
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

#[tokio::test]
async fn denied_followup_retains_preceding_accounts_and_partial_evidence()
-> Result<(), Box<dyn std::error::Error>> {
    let config = config();
    let executor = ApiExecutor::new(ExecutorConfig::default())?;
    let client = client(&config, &executor);
    let listing = pages::collect(
        &client,
        "organizations:ListAccounts",
        None,
        10,
        |token| async move {
            if token.is_none() {
                Ok(pages::page(
                    vec![Ok("first".to_owned())],
                    Some("next".into()),
                ))
            } else {
                Err(AwsScanError::Api(ApiError::Denied("fixture denial".into())))
            }
        },
    )
    .await;
    assert_eq!(listing.items, vec!["first"]);
    assert!(!listing.coverage.complete);
    assert_eq!(
        listing
            .coverage
            .failures
            .first()
            .map(|failure| failure.operation.as_str()),
        Some("organizations:ListAccounts")
    );
    Ok(())
}

#[tokio::test]
async fn malformed_and_duplicate_items_do_not_discard_valid_siblings()
-> Result<(), Box<dyn std::error::Error>> {
    let config = config();
    let executor = ApiExecutor::new(ExecutorConfig::default())?;
    let client = client(&config, &executor);
    let listing = pages::collect(&client, "fixture", None, 10, |_| async {
        Ok(pages::page(
            vec![
                Ok("first".to_owned()),
                Err(data_error("malformed")),
                Ok("first".to_owned()),
                Ok("last".to_owned()),
            ],
            None,
        ))
    })
    .await;
    assert_eq!(listing.items, vec!["first", "last"]);
    assert_eq!(listing.coverage.items_seen, 4);
    assert_eq!(listing.coverage.failures.len(), 2);
    assert!(!listing.coverage.complete);
    Ok(())
}

#[tokio::test]
async fn continuation_cycles_and_page_limits_are_explicit_failures()
-> Result<(), Box<dyn std::error::Error>> {
    let config = config();
    let executor = ApiExecutor::new(ExecutorConfig::default())?;
    let mut client = client(&config, &executor);
    let calls = AtomicUsize::new(0);
    let listing = pages::collect(&client, "fixture", None, 10, |_| {
        calls.fetch_add(1, Ordering::SeqCst);
        async { Ok(pages::page::<String>(vec![], Some("cycle".into()))) }
    })
    .await;
    assert_eq!(calls.load(Ordering::SeqCst), 2);
    assert!(!listing.coverage.complete);
    assert!(
        listing
            .coverage
            .failures
            .iter()
            .any(|failure| failure.message.contains("cycle"))
    );
    client.max_pages = 1;
    let limited = pages::collect(&client, "fixture", None, 10, |_| async {
        Ok(pages::page(
            vec![Ok("retained".to_owned())],
            Some("more".into()),
        ))
    })
    .await;
    assert_eq!(limited.items, vec!["retained"]);
    assert!(!limited.coverage.complete);
    assert!(
        limited
            .coverage
            .failures
            .iter()
            .any(|failure| failure.message.contains("page limit"))
    );
    Ok(())
}

#[tokio::test]
async fn precancelled_pagination_makes_no_fetch_call() -> Result<(), Box<dyn std::error::Error>> {
    let config = config();
    let executor = ApiExecutor::new(ExecutorConfig::default())?;
    executor.cancel();
    let client = client(&config, &executor);
    let calls = AtomicUsize::new(0);
    let listing = pages::collect(&client, "fixture", None, 10, |_| {
        calls.fetch_add(1, Ordering::SeqCst);
        async { Ok(pages::page::<String>(vec![], None)) }
    })
    .await;
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    assert!(listing.coverage.cancelled);
    assert!(!listing.coverage.complete);
    assert!(listing.items.is_empty());
    Ok(())
}

#[tokio::test]
async fn cancellation_between_pages_preserves_completed_results()
-> Result<(), Box<dyn std::error::Error>> {
    let config = config();
    let executor = ApiExecutor::new(ExecutorConfig::default())?;
    let client = client(&config, &executor);
    let calls = AtomicUsize::new(0);
    let listing = pages::collect(&client, "fixture", None, 10, |_| {
        calls.fetch_add(1, Ordering::SeqCst);
        executor.cancel();
        async {
            Ok(pages::page(
                vec![Ok("retained".to_owned())],
                Some("more".into()),
            ))
        }
    })
    .await;
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(listing.items, vec!["retained"]);
    assert!(listing.coverage.cancelled);
    assert!(!listing.coverage.complete);
    Ok(())
}

#[tokio::test]
async fn item_limit_and_invalid_token_never_claim_complete()
-> Result<(), Box<dyn std::error::Error>> {
    let config = config();
    let executor = ApiExecutor::new(ExecutorConfig::default())?;
    let client = client(&config, &executor);
    let limited = pages::collect(&client, "fixture", None, 1, |_| async {
        Ok(pages::page(
            vec![Ok("one".to_owned()), Ok("two".to_owned())],
            None,
        ))
    })
    .await;
    assert!(limited.items.is_empty());
    assert!(!limited.coverage.complete);
    let invalid_token = pages::collect(&client, "fixture", None, 10, |_| async {
        Ok(pages::page::<String>(vec![], Some(String::new())))
    })
    .await;
    assert!(!invalid_token.coverage.complete);
    Ok(())
}

#[test]
fn oversized_sdk_page_is_rejected_before_mapping() {
    let mapped = AtomicUsize::new(0);
    let result = Page::map(vec!["fixture"; 1_001], None, |value| {
        mapped.fetch_add(1, Ordering::SeqCst);
        Ok(value.to_owned())
    });
    assert!(result.is_err());
    assert_eq!(mapped.load(Ordering::SeqCst), 0);
}

#[test]
fn failure_diagnostic_caps_preserve_suppressed_count_and_partial_status() {
    let mut coverage = DiscoveryCoverage::success();
    for _ in 0..MAX_FAILURES.saturating_add(7) {
        coverage.fail("fixture", None, &data_error("failure"));
    }
    assert_eq!(coverage.failures.len(), MAX_FAILURES);
    assert_eq!(coverage.suppressed_failures, 7);
    assert!(!coverage.complete);
    let mut combined = DiscoveryCoverage::success();
    combined.merge(coverage.clone());
    combined.merge(coverage);
    assert_eq!(combined.failures.len(), MAX_FAILURES);
    assert_eq!(
        combined.suppressed_failures,
        u64::try_from(MAX_FAILURES).unwrap_or(0).saturating_add(14)
    );
    assert!(!combined.complete);
}
