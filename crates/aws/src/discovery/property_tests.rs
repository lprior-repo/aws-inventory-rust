//! Role identity, partial-report and lazy SDK configuration invariants.

use aws_credential_types::Credentials;
use aws_types::sdk_config::TimeSource;
use inventory_engine::limiter::ExecutorConfig;
use proptest::prelude::*;
use proptest::test_runner::{Config, RngSeed, TestCaseError};

use super::*;

fn configuration() -> Config {
    Config {
        cases: 128,
        max_shrink_iters: 4_096,
        rng_seed: RngSeed::Fixed(0x4449_5343_4f56_4552),
        ..Config::default()
    }
}

fn role() -> AssumeRoleOptions {
    AssumeRoleOptions {
        role_arn: "arn:aws:iam::000000000000:role/fixture/Audit".into(),
        session_name: "inventory-fixture".into(),
        external_id: None,
        duration_seconds: 3_600,
    }
}

fn failure(index: usize) -> DiscoveryFailure {
    DiscoveryFailure {
        operation: format!("operation-{index}"),
        parent_id: None,
        category: "invalid_response".into(),
        message: format!("fixture-{index}"),
    }
}

proptest! {
    #![proptest_config(configuration())]

    #[test]
    fn account_role_construction_round_trips_without_changing_the_identity(
        account in "[0-9]{12}",
        partition in prop::sample::select(vec!["aws", "aws-cn", "aws-us-gov", "aws-iso", "aws-iso-b", "aws-iso-e", "aws-iso-f", "aws-eusc"]),
        segments in prop::collection::vec("[A-Za-z][A-Za-z0-9_+=,.@-]{0,15}", 1..6),
    ) {
        let path = segments.join("/");
        let arn = account_role_arn(partition, &account, &path)?;
        prop_assert_eq!(&arn, &format!("arn:{partition}:iam::{account}:role/{path}"));
        prop_assert!(validate_role_arn(&arn).is_ok());
        let mut options = role();
        options.role_arn = arn;
        prop_assert!(options.validate().is_ok());
    }

    #[test]
    fn generated_account_ids_preserve_leading_zeroes_and_reject_non_ascii_digits(
        tail in "[0-9]{11}",
        replacement in prop::sample::select(vec!["x", ":", "０", "١", " ", "\n"]),
    ) {
        let account = format!("0{tail}");
        prop_assert_eq!(account_role_arn("aws", &account, "Audit")?, format!("arn:aws:iam::{account}:role/Audit"));
        prop_assert!(validate_account_id(&format!("{replacement}{tail}")).is_err(), "invalid leading account character accepted");
        prop_assert!(validate_account_id(&tail).is_err());
        prop_assert!(validate_account_id(&format!("00{tail}")).is_err(), "thirteen-digit account accepted");
    }

    #[test]
    fn role_paths_reject_interpretive_or_control_characters(
        name in "[A-Za-z][A-Za-z0-9]{1,20}",
        bad in prop::sample::select(vec!["*", "?", "%2f", ":", "\\", "\n", "\r", "\0", "é", " "]),
    ) {
        prop_assert!(account_role_arn("aws", "000000000000", &format!("team/{name}{bad}")).is_err(), "forbidden role path suffix accepted");
        for path in [format!("/{name}"), format!("{name}/"), format!("{name}//Audit"), format!("{name}/../Audit"), format!("{name}/./Audit")] {
            prop_assert!(account_role_arn("aws", "000000000000", &path).is_err());
        }
    }

    #[test]
    fn role_path_prefix_and_role_name_have_independent_length_bounds(
        prefix_length in 508_usize..514,
        name_length in 62_usize..67,
    ) {
        let path = format!("{}/{}", "p".repeat(prefix_length), "r".repeat(name_length));
        prop_assert_eq!(validate_role_path(&path).is_ok(), prefix_length <= 510 && name_length <= 64);
    }

    #[test]
    fn iam_role_arns_cannot_redirect_to_another_service_region_or_resource(
        account in "[0-9]{12}",
        name in "[A-Za-z][A-Za-z0-9]{1,20}",
        invalid in prop::sample::select(vec!["arn:aws:sts::", "arn:aws:iam:us-east-1:", "url:aws:iam::", "arn:AWS:iam::"]),
        resource in prop::sample::select(vec!["user", "group", "policy", "instance-profile"]),
    ) {
        prop_assert!(validate_role_arn(&format!("{invalid}{account}:role/{name}")).is_err(), "invalid role ARN prefix accepted");
        prop_assert!(validate_role_arn(&format!("arn:aws:iam::{account}:{resource}/{name}")).is_err(), "non-role resource accepted");
        prop_assert!(validate_role_arn(&format!("arn:aws:iam::{account}:role/{name}:suffix")).is_err(), "unexpected ARN field accepted");
    }

    #[test]
    fn role_sessions_have_explicit_chained_session_bounds(duration in 0_u32..5_000, name_length in 0_usize..68) {
        let mut options = role();
        options.duration_seconds = duration;
        options.session_name = "r".repeat(name_length);
        prop_assert_eq!(options.validate().is_ok(), (900..=3_600).contains(&duration) && (2..=64).contains(&name_length));
    }

    #[test]
    fn external_id_validation_never_puts_the_id_into_debug_or_error_messages(
        secret in "fixture:[A-Za-z0-9_/+=,.@-]{2,64}",
        invalid in any::<bool>(),
    ) {
        let mut options = role();
        let secret = if invalid { format!("{secret} invalid") } else { secret };
        options.external_id = Some(secret.clone());
        let debug = format!("{options:?}");
        prop_assert!(!debug.contains(&secret));
        prop_assert!(debug.contains("external_id_present: true"));
        let result = options.validate();
        prop_assert_eq!(result.is_ok(), !invalid);
        if let Err(error) = result {
            prop_assert!(!error.to_string().contains(&secret));
        }
    }

    #[test]
    fn credential_deadlines_preserve_stricter_limits_and_bound_missing_limits(
        operation in prop::option::of(0_u64..120_000),
        attempt in prop::option::of(0_u64..120_000),
        connect in prop::option::of(0_u64..120_000),
        read in prop::option::of(0_u64..120_000),
    ) {
        let mut builder = TimeoutConfig::builder();
        builder.set_operation_timeout(operation.map(Duration::from_millis));
        builder.set_operation_attempt_timeout(attempt.map(Duration::from_millis));
        builder.set_connect_timeout(connect.map(Duration::from_millis));
        builder.set_read_timeout(read.map(Duration::from_millis));
        let bounded = credential_timeouts(Some(&builder.build()));
        for (actual, requested, ceiling) in [
            (bounded.operation_timeout(), operation, 30_000),
            (bounded.operation_attempt_timeout(), attempt, 30_000),
            (bounded.connect_timeout(), connect, 10_000),
            (bounded.read_timeout(), read, 30_000),
        ] {
            let actual = actual.ok_or_else(|| TestCaseError::fail("credential deadline disappeared"))?;
            prop_assert!(actual <= Duration::from_millis(ceiling));
            match requested {
                Some(value) if value <= ceiling => prop_assert_eq!(actual, Duration::from_millis(value)),
                _ => prop_assert_eq!(actual, Duration::from_millis(ceiling)),
            }
        }
    }

    #[test]
    fn diagnostic_capacity_does_not_hide_failure_or_cancellation(
        ordinary in 0_usize..1_100,
        trailing in 0_usize..16,
    ) {
        let mut coverage = DiscoveryCoverage::success();
        for index in 0..ordinary {
            coverage.fail("ListFixture", Some("parent"), &data_error(&format!("fixture-{index}")));
        }
        coverage.fail("ListFixture", None, &AwsScanError::Api(ApiError::Cancelled));
        for _ in 0..trailing {
            coverage.fail("ListFixture", None, &AwsScanError::Api(ApiError::Denied("fixture".into())));
        }
        let total = ordinary + 1 + trailing;
        prop_assert!(!coverage.complete);
        prop_assert!(coverage.cancelled);
        prop_assert_eq!(coverage.failures.len(), total.min(MAX_FAILURES));
        prop_assert_eq!(coverage.suppressed_failures, u64::try_from(total.saturating_sub(MAX_FAILURES))?);
        prop_assert_eq!(u64::try_from(coverage.failures.len())? + coverage.suppressed_failures, u64::try_from(total)?);
    }

    #[test]
    fn merging_partial_coverage_preserves_totals_diagnostics_and_cancellation(
        left_count in 0_usize..=1_024,
        right_count in 0_usize..=1_024,
        left_suppressed in 0_u64..100,
        right_suppressed in 0_u64..100,
        left_pages in any::<u64>(),
        right_pages in any::<u64>(),
        left_items in any::<u64>(),
        right_items in any::<u64>(),
        left_complete in any::<bool>(),
        right_complete in any::<bool>(),
        left_cancelled in any::<bool>(),
        right_cancelled in any::<bool>(),
    ) {
        let mut left = DiscoveryCoverage {
            pages: left_pages, items_seen: left_items, complete: left_complete,
            cancelled: left_cancelled, failures: (0..left_count).map(failure).collect(),
            suppressed_failures: left_suppressed,
        };
        let right = DiscoveryCoverage {
            pages: right_pages, items_seen: right_items, complete: right_complete,
            cancelled: right_cancelled, failures: (left_count..left_count + right_count).map(failure).collect(),
            suppressed_failures: right_suppressed,
        };
        left.merge(right);
        let retained = (left_count + right_count).min(MAX_FAILURES);
        prop_assert_eq!(left.pages, left_pages.saturating_add(right_pages));
        prop_assert_eq!(left.items_seen, left_items.saturating_add(right_items));
        prop_assert_eq!(left.complete, left_complete && right_complete);
        prop_assert_eq!(left.cancelled, left_cancelled || right_cancelled);
        prop_assert_eq!(left.failures, (0..retained).map(failure).collect::<Vec<_>>());
        prop_assert_eq!(left.suppressed_failures, left_suppressed + right_suppressed + u64::try_from(left_count + right_count - retained)?);
    }

    #[test]
    fn only_explicitly_selected_sources_produce_an_identifier(identifier in "[A-Za-z][A-Za-z0-9:/_-]{0,128}") {
        let selected = SourceSelection::Selected { identifier: identifier.clone(), reason: "fixture".into() };
        prop_assert_eq!(selected.selected_identifier(), Some(identifier.as_str()));
        for other in [SourceSelection::NotRequested, SourceSelection::Absent, SourceSelection::Ambiguous { candidates: vec![identifier] }, SourceSelection::Unavailable { reason: "fixture".into() }] {
            prop_assert!(other.selected_identifier().is_none());
        }
    }
}

#[derive(Debug)]
struct FixedTime;

impl TimeSource for FixedTime {
    fn now(&self) -> std::time::SystemTime {
        std::time::UNIX_EPOCH
    }
}

proptest! {
    #![proptest_config(Config { cases: 32, ..configuration() })]

    #[test]
    fn every_compiled_inventory_sdk_inherits_single_attempt_configuration(
        original_attempts in 1_u32..=64,
        region in prop::sample::select(vec!["us-east-1", "eu-west-1", "us-gov-west-1", "cn-north-1"]),
        origin in 0_u8..7,
    ) {
        let runtime = tokio::runtime::Builder::new_current_thread().enable_time().build()?;
        runtime.block_on(async {
            let executor = ApiExecutor::new(ExecutorConfig::default())?;
            let mut builder = SdkConfig::builder()
                .behavior_version(BehaviorVersion::latest())
                .region(Region::new(region))
                .endpoint_url("https://127.0.0.1:1")
                .time_source(FixedTime)
                .credentials_provider(SharedCredentialsProvider::new(Credentials::new(
                    "EMULATOR_PROPERTY_FIXTURE", "fixture-no-network", None, None, "property-fixture",
                )))
                .retry_config(RetryConfig::standard().with_max_attempts(original_attempts));
            let origin = match origin {
                0 => aws_types::origin::Origin::unknown(),
                1 => aws_types::origin::Origin::shared_config(),
                2 => aws_types::origin::Origin::shared_environment_variable(),
                3 => aws_types::origin::Origin::shared_profile_file(),
                4 => aws_types::origin::Origin::imds(),
                5 => aws_types::origin::Origin::service_config(),
                _ => aws_types::origin::Origin::service_profile_file(),
            };
            builder.insert_origin("retry_config", origin);
            let base = builder.build();
            let assumed = assume_role_config(&base, &role(), &executor).await?;
            prop_assert_eq!(base.retry_config().map(|value| value.max_attempts()), Some(original_attempts));
            prop_assert_eq!(assumed.retry_config().map(|value| value.max_attempts()), Some(1));
            prop_assert_eq!(assumed.endpoint_url(), base.endpoint_url());
            prop_assert_eq!(assumed.region(), base.region());
            prop_assert!(assumed.credentials_provider().is_some());
            prop_assert!(assumed.get_origin("retry_config").is_client_config());

            // These are the same official from-shared-config conversions used
            // by Client::new. Constructing them must not fetch credentials.
            let mut configurations = Vec::new();
            macro_rules! check_sdk {
                ($sdk:ident, $shared:expr) => {{
                    let config = $sdk::config::Builder::from($shared).build();
                    configurations.push((stringify!($sdk), config.retry_config().map(|value| value.max_attempts())));
                }};
            }
            for shared in [assumed.clone(), assumed.to_builder().region(Region::new("us-east-1")).build()] {
                check_sdk!(aws_sdk_sts, &shared);
                check_sdk!(aws_sdk_ec2, &shared);
                check_sdk!(aws_sdk_dynamodb, &shared);
                check_sdk!(aws_sdk_lambda, &shared);
                check_sdk!(aws_sdk_ecs, &shared);
                check_sdk!(aws_sdk_eks, &shared);
                check_sdk!(aws_sdk_elasticloadbalancingv2, &shared);
                check_sdk!(aws_sdk_config, &shared);
                check_sdk!(aws_sdk_resourceexplorer2, &shared);
                check_sdk!(aws_sdk_cloudcontrol, &shared);
                check_sdk!(aws_sdk_s3, &shared);
                check_sdk!(aws_sdk_iam, &shared);
                check_sdk!(aws_sdk_route53, &shared);
                check_sdk!(aws_sdk_cloudfront, &shared);
                check_sdk!(aws_sdk_organizations, &shared);
            }
            prop_assert_eq!(configurations.len(), 30);
            prop_assert!(configurations.iter().all(|(_, attempts)| *attempts == Some(1)), "service retry settings: {:?}", configurations);
            prop_assert_eq!(executor.stats().attempts, 0);
            prop_assert_eq!(executor.stats().in_flight, 0);
            Ok::<(), TestCaseError>(())
        })?;
    }
}
