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

#[derive(Debug)]
struct FixedTime;
impl TimeSource for FixedTime {
    fn now(&self) -> std::time::SystemTime {
        std::time::UNIX_EPOCH
    }
}

mod denied_followup_retains_preceding_accounts_and_partial_evidence;
mod discovery_options_reject_unbounded_or_meaningless_requests;
