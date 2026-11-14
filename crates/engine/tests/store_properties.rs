use std::collections::{BTreeMap, BTreeSet};
use std::future::Future;
use std::time::Duration;

use inventory_core::{
    CollectionFailure, CollectionPage, CollectionScope, Edge, Resource, ScanStatus, Source,
};
use inventory_engine::store::{Checkpoint, Store, StoreError};
use proptest::prelude::*;
use proptest::test_runner::{Config, RngSeed, TestCaseError, TestCaseResult};
use serde_json::json;
use tokio::runtime::{Builder, Runtime};

const SCAN: &str = "property-scan";

fn failure(error: impl std::fmt::Display) -> TestCaseError {
    TestCaseError::fail(error.to_string())
}

fn runtime() -> Result<Runtime, TestCaseError> {
    Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(failure)
}

async fn bounded_case(case: impl Future<Output = TestCaseResult>) -> TestCaseResult {
    tokio::time::timeout(Duration::from_secs(10), case)
        .await
        .map_err(|_| failure("durable store property exceeded its 10-second case deadline"))?
}

fn scope() -> CollectionScope {
    CollectionScope {
        account_id: "111111111111".into(),
        region: "us-east-1".into(),
        collector: "property_collector".into(),
        parent_id: None,
        source: Source::Direct,
    }
}

fn resource(page: u32, index: u16, value: u16) -> Resource {
    Resource {
        id: format!("fixture-{page}-{index}"),
        account_id: "111111111111".into(),
        region: "us-east-1".into(),
        kind: "aws_ec2_instance".into(),
        name: Some(format!("value-{value}")),
        properties: json!({"value":value,"revision":0}),
        tags: BTreeMap::from([("fixture".into(), value.to_string())]),
        source: Source::Direct,
        observed_at: i64::from(page),
    }
}

fn pages(values: &[u16]) -> Result<Vec<CollectionPage>, TestCaseError> {
    values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            let page_number = u32::try_from(index.saturating_add(1)).map_err(failure)?;
            let resources: Vec<_> = (0..value
                .checked_rem(3)
                .ok_or_else(|| failure("invalid fixture count"))?
                .saturating_add(1))
                .map(|item| resource(page_number, item, *value))
                .collect();
            let edges = resources
                .iter()
                .map(|resource| Edge {
                    from_id: resource.id.clone(),
                    to_id: format!("external-{page_number}"),
                    relation: "IN_VPC".into(),
                    source: Source::Direct,
                    observed_at: i64::from(page_number),
                    properties: json!({"fixture_value":value}),
                })
                .collect();
            Ok(CollectionPage {
                scope: scope(),
                page_number,
                next_token: (index.saturating_add(1) < values.len())
                    .then(|| format!("next-{page_number}")),
                resources,
                edges,
            })
        })
        .collect()
}

proptest! {
    #![proptest_config(Config {
        cases: 24,
        max_shrink_iters: 96,
        max_shrink_time: 2_000,
        rng_seed: RngSeed::Fixed(0x4157_535f_5041_4745),
        ..Config::default()
    })]

    #[test]
    fn generated_page_replay_failure_and_finalization_commands_match_the_ledger(
        values in prop::collection::vec(any::<u16>(), 2..=5),
        actions in prop::collection::vec((any::<u8>(), 0_u8..=10), 1..=16),
        split in 0_usize..=16,
    ) {
        runtime()?.block_on(bounded_case(page_case(&values, &actions, split)))?;
    }

    #[test]
    fn generated_resource_and_edge_constraint_failures_roll_back_the_whole_page(
        value in any::<u16>(),
        duplicate_edge in any::<bool>(),
    ) {
        runtime()?.block_on(bounded_case(rollback_case(value, duplicate_edge)))?;
    }
}
#[path = "store_properties/ledger.rs"]
mod ledger;
use ledger::*;
#[path = "store_properties/commands.rs"]
mod commands;
use commands::*;
