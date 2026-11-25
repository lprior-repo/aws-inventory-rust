#[path = "seed_inventory/collection.rs"]
mod collection;
#[path = "seed_inventory/compute.rs"]
mod compute;
#[path = "seed_inventory/draft.rs"]
mod draft;
#[path = "seed_inventory/global.rs"]
mod global;
#[path = "seed_inventory/messaging.rs"]
mod messaging;
#[path = "seed_inventory/network.rs"]
mod network;
#[path = "seed_inventory/runtime.rs"]
mod runtime;
#[path = "seed_inventory/storage.rs"]
mod storage;

use inventory_harness::inventory_seed::{ScenarioSeedOracle, SeedError, SeedScenario};
use runtime::{NATIVE_SLOT, NativeRuntime};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const ACCOUNT: &str = "000000000000";
const PARTITION: &str = "aws";
const REGION: &str = "us-east-1";

#[derive(Clone, Copy)]
struct Fixture {
    name: &'static str,
    build: fn() -> Result<SeedScenario, SeedError>,
    check: fn(&ScenarioSeedOracle, &str, &str) -> TestResult,
}

const EC2: Fixture = Fixture {
    name: "ec2-network",
    build: network::ec2_network,
    check: network::assert_ec2_network_oracle,
};
const ELBV2: Fixture = Fixture {
    name: "elbv2",
    build: network::elbv2,
    check: network::assert_elbv2_oracle,
};
const CLOUDWATCH: Fixture = Fixture {
    name: "cloudwatch",
    build: || network::cloudwatch(ACCOUNT, REGION, PARTITION),
    check: network::assert_cloudwatch_oracle,
};
const LAMBDA: Fixture = Fixture {
    name: "lambda",
    build: || compute::lambda(PARTITION, REGION, ACCOUNT),
    check: compute::assert_lambda_oracle,
};
const EKS: Fixture = Fixture {
    name: "eks",
    build: || compute::eks(PARTITION, REGION, ACCOUNT),
    check: compute::assert_eks_oracle,
};
const ECS: Fixture = Fixture {
    name: "ecs",
    build: || compute::ecs(PARTITION, REGION, ACCOUNT),
    check: compute::assert_ecs_oracle,
};
const S3: Fixture = Fixture {
    name: "s3",
    build: storage::s3_scenario,
    check: storage::assert_s3_oracle,
};
const KMS: Fixture = Fixture {
    name: "kms",
    build: storage::kms_scenario,
    check: storage::assert_kms_oracle,
};
const DYNAMODB: Fixture = Fixture {
    name: "dynamodb",
    build: storage::dynamodb_scenario,
    check: storage::assert_dynamodb_oracle,
};
const RDS: Fixture = Fixture {
    name: "rds",
    build: storage::rds_scenario,
    check: storage::assert_rds_oracle,
};
const SQS: Fixture = Fixture {
    name: "sqs",
    build: messaging::sqs_scenario,
    check: messaging::assert_sqs_oracle,
};
const SNS: Fixture = Fixture {
    name: "sns",
    build: messaging::sns_scenario,
    check: messaging::assert_sns_oracle,
};
const SFN: Fixture = Fixture {
    name: "sfn",
    build: messaging::sfn_scenario,
    check: messaging::assert_sfn_oracle,
};
const IAM: Fixture = Fixture {
    name: "iam",
    build: global::iam,
    check: global::assert_iam_oracle,
};
const ORGANIZATIONS: Fixture = Fixture {
    name: "organizations",
    build: global::organizations,
    check: global::assert_organizations_oracle,
};
const ROUTE53: Fixture = Fixture {
    name: "route53",
    build: global::route53,
    check: global::assert_route53_oracle,
};
const CLOUDFRONT: Fixture = Fixture {
    name: "cloudfront",
    build: global::cloudfront,
    check: global::assert_cloudfront_oracle,
};
const FIXTURES: [Fixture; 17] = [
    EC2,
    ELBV2,
    CLOUDWATCH,
    LAMBDA,
    EKS,
    ECS,
    S3,
    KMS,
    DYNAMODB,
    RDS,
    SQS,
    SNS,
    SFN,
    IAM,
    ORGANIZATIONS,
    ROUTE53,
    CLOUDFRONT,
];

macro_rules! native_case {
    ($name:ident, $fixture:ident) => {
        #[tokio::test(flavor = "current_thread")]
        async fn $name() -> TestResult {
            run_fixture($fixture).await
        }
    };
}

native_case!(
    given_native_ec2_when_seeded_then_exact_network_oracle_is_observed,
    EC2
);
native_case!(
    given_native_elbv2_when_seeded_then_real_dependencies_are_observed,
    ELBV2
);
native_case!(
    given_native_cloudwatch_when_seeded_then_request_identity_is_explicit,
    CLOUDWATCH
);
native_case!(
    given_native_lambda_when_seeded_then_metadata_only_oracle_is_observed,
    LAMBDA
);
native_case!(
    given_native_eks_when_seeded_then_control_plane_dependencies_are_observed,
    EKS
);
native_case!(
    given_native_ecs_when_seeded_then_zero_task_metadata_is_observed,
    ECS
);
native_case!(
    given_native_s3_when_seeded_then_exact_global_keys_are_observed,
    S3
);
native_case!(
    given_native_kms_when_seeded_then_metadata_and_alias_provenance_are_observed,
    KMS
);
native_case!(
    given_native_dynamodb_when_seeded_then_real_table_metadata_is_observed,
    DYNAMODB
);
native_case!(
    given_native_rds_when_seeded_then_configuration_metadata_is_observed,
    RDS
);
native_case!(
    given_native_sqs_when_seeded_then_named_map_arns_are_observed,
    SQS
);
native_case!(
    given_native_sns_when_seeded_then_exact_topic_arns_are_observed,
    SNS
);
native_case!(
    given_native_sfn_when_seeded_then_state_machine_metadata_is_observed,
    SFN
);
native_case!(
    given_native_iam_when_seeded_then_exact_account_scope_is_observed,
    IAM
);
native_case!(
    given_native_organizations_when_seeded_then_fresh_org_metadata_is_observed,
    ORGANIZATIONS
);
native_case!(
    given_native_route53_when_seeded_then_returned_zone_identity_is_preserved,
    ROUTE53
);
native_case!(
    given_native_cloudfront_when_seeded_then_origin_control_metadata_is_observed,
    CLOUDFRONT
);

async fn run_fixture(fixture: Fixture) -> TestResult {
    let _permit = NATIVE_SLOT.acquire().await?;
    let native = NativeRuntime::start()?;
    seed_checked(&native, fixture).await?;
    native.stop()
}

async fn seed_checked(native: &NativeRuntime, fixture: Fixture) -> TestResult<ScenarioSeedOracle> {
    let scenario =
        (fixture.build)().map_err(|error| format!("{} fixture: {error}", fixture.name))?;
    let oracle = native
        .seed(scenario)
        .await
        .map_err(|error| format!("{} seeding: {error}", fixture.name))?;
    (fixture.check)(&oracle, ACCOUNT, REGION)
        .map_err(|error| format!("{} oracle: {error}", fixture.name))?;
    Ok(oracle)
}

#[tokio::test(flavor = "current_thread")]
async fn given_all_native_seed_oracles_when_collector_sweeps_then_sqlite_contains_every_exact_key_and_edge()
-> TestResult {
    let _permit = NATIVE_SLOT.acquire().await?;
    let native = NativeRuntime::start()?;
    let mut oracles = Vec::with_capacity(FIXTURES.len());
    for fixture in FIXTURES {
        oracles.push(seed_checked(&native, fixture).await?);
    }
    collection::compare(&native, &oracles).await?;
    native.stop()
}
