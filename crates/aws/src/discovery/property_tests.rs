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

#[derive(Debug)]
struct FixedTime;

impl TimeSource for FixedTime {
    fn now(&self) -> std::time::SystemTime {
        std::time::UNIX_EPOCH
    }
}

mod account_role_construction_round_trips_without_changing_the_identity;
mod every_compiled_inventory_sdk_inherits_single_attempt_configuration;
