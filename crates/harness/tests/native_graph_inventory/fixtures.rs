use super::{
    ACCOUNT, PARTITION, REGION, TestResult, compute, global, messaging, network,
    runtime::NativeRuntime, storage,
};
use inventory_harness::inventory_seed::{ScenarioSeedOracle, SeedError, SeedScenario};

struct Fixture {
    name: &'static str,
    build: fn() -> Result<SeedScenario, SeedError>,
    check: fn(&ScenarioSeedOracle, &str, &str) -> TestResult,
}

const FIXTURES: [Fixture; 17] = [
    Fixture {
        name: "ec2",
        build: network::ec2_network,
        check: network::assert_ec2_network_oracle,
    },
    Fixture {
        name: "elbv2",
        build: network::elbv2,
        check: network::assert_elbv2_oracle,
    },
    Fixture {
        name: "cloudwatch",
        build: || network::cloudwatch(ACCOUNT, REGION, PARTITION),
        check: network::assert_cloudwatch_oracle,
    },
    Fixture {
        name: "lambda",
        build: || compute::lambda(PARTITION, REGION, ACCOUNT),
        check: compute::assert_lambda_oracle,
    },
    Fixture {
        name: "eks",
        build: || compute::eks(PARTITION, REGION, ACCOUNT),
        check: compute::assert_eks_oracle,
    },
    Fixture {
        name: "ecs",
        build: || compute::ecs(PARTITION, REGION, ACCOUNT),
        check: compute::assert_ecs_oracle,
    },
    Fixture {
        name: "s3",
        build: storage::s3_scenario,
        check: storage::assert_s3_oracle,
    },
    Fixture {
        name: "kms",
        build: storage::kms_scenario,
        check: storage::assert_kms_oracle,
    },
    Fixture {
        name: "dynamodb",
        build: storage::dynamodb_scenario,
        check: storage::assert_dynamodb_oracle,
    },
    Fixture {
        name: "rds",
        build: storage::rds_scenario,
        check: storage::assert_rds_oracle,
    },
    Fixture {
        name: "sqs",
        build: messaging::sqs_scenario,
        check: messaging::assert_sqs_oracle,
    },
    Fixture {
        name: "sns",
        build: messaging::sns_scenario,
        check: messaging::assert_sns_oracle,
    },
    Fixture {
        name: "sfn",
        build: messaging::sfn_scenario,
        check: messaging::assert_sfn_oracle,
    },
    Fixture {
        name: "iam",
        build: global::iam,
        check: global::assert_iam_oracle,
    },
    Fixture {
        name: "organizations",
        build: global::organizations,
        check: global::assert_organizations_oracle,
    },
    Fixture {
        name: "route53",
        build: global::route53,
        check: global::assert_route53_oracle,
    },
    Fixture {
        name: "cloudfront",
        build: global::cloudfront,
        check: global::assert_cloudfront_oracle,
    },
];

pub(super) async fn seed_all(
    native: &NativeRuntime,
) -> TestResult<(Vec<ScenarioSeedOracle>, super::safe_fields::Expectations)> {
    let mut oracles = Vec::with_capacity(FIXTURES.len());
    let mut safe_records = super::safe_fields::Expectations::new();
    for fixture in FIXTURES {
        let scenario = (fixture.build)()?;
        let oracle = native
            .seed(scenario.clone())
            .await
            .map_err(|error| format!("{} native seeding failed: {error}", fixture.name))?;
        (fixture.check)(&oracle, ACCOUNT, REGION)?;
        for (key, fields) in super::safe_fields::from_fixture(&scenario, &oracle)? {
            assert!(
                safe_records.insert(key, fields).is_none(),
                "duplicate safe-record fixture key"
            );
        }
        oracles.push(oracle);
    }
    Ok((oracles, safe_records))
}
