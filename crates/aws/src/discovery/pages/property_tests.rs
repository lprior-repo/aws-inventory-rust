use std::sync::atomic::{AtomicUsize, Ordering};

use aws_config::BehaviorVersion;
use aws_types::{SdkConfig, region::Region};
use inventory_engine::limiter::{ApiError, ApiExecutor, ExecutorConfig};
use proptest::prelude::*;
use proptest::test_runner::{Config, RngSeed, TestCaseError};

use super::*;

fn configuration() -> Config {
    Config {
        cases: 128,
        max_shrink_iters: 4_096,
        rng_seed: RngSeed::Fixed(0x5041_4745_5f41_5753),
        ..Config::default()
    }
}

fn runtime() -> Result<tokio::runtime::Runtime, TestCaseError> {
    tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .start_paused(true)
        .build()
        .map_err(|error| TestCaseError::fail(error.to_string()))
}

fn client<'a>(
    config: &'a SdkConfig,
    executor: &'a ApiExecutor,
    max_pages: u32,
) -> DiscoveryClient<'a> {
    DiscoveryClient {
        config,
        executor,
        account: "000000000000",
        partition: "aws",
        region: "us-east-1",
        max_pages,
        page_size: 1,
    }
}

fn config() -> SdkConfig {
    SdkConfig::builder()
        .behavior_version(BehaviorVersion::latest())
        .region(Region::new("us-east-1"))
        .build()
}

#[test]
fn empty_cursor_is_failure_even_when_all_rows_are_valid() {
    let mut cursor = Cursor::new();
    cursor.accept(
        page(vec![Ok("resource".to_owned())], Some(String::new())),
        "ListFixture",
        None,
        2,
        2,
    );
    assert!(cursor.done);
    assert!(!cursor.listing.coverage.complete);
    assert_eq!(cursor.listing.items, vec!["resource"]);
    assert!(cursor.seen_tokens.is_empty());
}

#[test]
fn exact_identity_byte_limit_accepts_unicode_without_truncation() {
    let identity = "é".repeat(MAX_METADATA_BYTES / 2);
    let mut cursor = Cursor::new();
    cursor.accept(
        page(vec![Ok(identity.clone())], None),
        "ListFixture",
        None,
        1,
        1,
    );
    assert!(cursor.listing.coverage.complete);
    assert_eq!(cursor.listing.items, vec![identity]);
}

mod discovery_keeps_the_first_observation_of_every_identity;
mod mapper_runs_once_per_admitted_row_and_never_for_an_oversized_page;
