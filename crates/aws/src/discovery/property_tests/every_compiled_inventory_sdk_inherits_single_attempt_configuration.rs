use super::*;

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
