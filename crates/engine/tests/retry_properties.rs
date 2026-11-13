//! Generated retry contracts use an outcome-tape oracle, not executor internals.
//! Every case has a fixed seed, bounded input/shrinking, and paused Tokio time.

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::future::pending;
use std::time::Duration;

use futures_util::{FutureExt, StreamExt, future, stream};
use inventory_engine::limiter::{
    ApiError, ApiExecutor, BucketPolicy, ExecutorConfig, MAX_ATTEMPTS, MAX_RETRY_AFTER_MS,
    PeriodLimits, RetryAfter, RetryKind,
};
use proptest::prelude::*;
use proptest::test_runner::{Config, RngSeed, TestCaseError, TestCaseResult};
use tokio::runtime::{Builder, Runtime};
use tokio::time::{Instant, advance, timeout};

const TAPE_LENGTH: usize = 12;
const CASE_DEADLINE: Duration = Duration::from_secs(60);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Step {
    Success,
    Transient,
    Throttled,
    AttemptTimeout,
    Permanent,
    Denied,
    Cancelled,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Terminal {
    Success(usize),
    Permanent,
    Denied,
    Cancelled,
    Exhausted,
    Unexpected,
}

fn failure(error: impl std::fmt::Display) -> TestCaseError {
    TestCaseError::fail(error.to_string())
}

fn virtual_runtime() -> Result<Runtime, TestCaseError> {
    Builder::new_current_thread()
        .enable_all()
        .start_paused(true)
        .build()
        .map_err(failure)
}

fn config(max_attempts: u32) -> ExecutorConfig {
    ExecutorConfig {
        global_concurrency: 1,
        per_bucket_concurrency: 1,
        requests_per_second: 1_000,
        max_attempts,
        attempt_timeout_ms: 5,
        max_backoff_ms: 100,
    }
}

proptest! {
    #![proptest_config(Config {
        cases: 64,
        max_shrink_iters: 256,
        max_shrink_time: 2_000,
        rng_seed: RngSeed::Fixed(0x4157_535f_5245_5452),
        ..Config::default()
    })]

    #[test]
    fn arbitrary_outcome_tapes_stop_at_the_first_terminal_result(
        tape in prop::collection::vec(step(), TAPE_LENGTH),
        budget in 1_u32..=MAX_ATTEMPTS,
        seed in any::<u64>(),
    ) {
        virtual_runtime()?.block_on(check_tape(&tape, budget, seed))?;
    }

    #[test]
    fn retryable_tapes_exhaust_exactly_the_configured_budget(
        tape in prop::collection::vec(retryable_step(), TAPE_LENGTH),
        budget in 1_u32..=MAX_ATTEMPTS,
        seed in any::<u64>(),
    ) {
        virtual_runtime()?.block_on(check_tape(&tape, budget, seed))?;
    }

    #[test]
    fn accepted_server_delays_are_lower_bounds_without_occupying_other_buckets(
        delay_ms in prop_oneof![Just(0_u64), 1_u64..=10_000, Just(MAX_RETRY_AFTER_MS)],
        retry_kind in 0_u8..3,
        seed in any::<u64>(),
    ) {
        virtual_runtime()?.block_on(check_delay(delay_ms, kind(retry_kind), seed))?;
    }

    #[test]
    fn rejected_server_hints_close_only_the_affected_bucket(
        hint in prop_oneof![
            Just(RetryAfter::Invalid),
            Just(RetryAfter::Excessive),
            (1_u64..=100_000).prop_map(|extra| RetryAfter::Delay(Duration::from_millis(MAX_RETRY_AFTER_MS.saturating_add(extra)))),
        ],
        retry_kind in 0_u8..3,
        seed in any::<u64>(),
    ) {
        virtual_runtime()?.block_on(check_rejected_hint(hint, kind(retry_kind), seed))?;
    }

    #[test]
    fn dropping_generated_waiters_releases_all_local_and_global_capacity(
        limits in (1_usize..=4).prop_flat_map(|global| (Just(global), 1_usize..=global)),
        assignments in prop::collection::vec(any::<bool>(), 1..=8),
    ) {
        virtual_runtime()?.block_on(check_permits(limits.0, limits.1, &assignments, false))?;
    }

    #[test]
    fn cancellation_drains_generated_waiters_and_stops_admission(
        limits in (1_usize..=4).prop_flat_map(|global| (Just(global), 1_usize..=global)),
        assignments in prop::collection::vec(any::<bool>(), 1..=8),
    ) {
        virtual_runtime()?.block_on(check_permits(limits.0, limits.1, &assignments, true))?;
    }

    #[test]
    fn every_generated_rational_rate_bounds_concurrent_dispatch(
        requests in 1_u32..=16,
        period_ms in 1_u64..=250,
        count in 2_usize..=8,
    ) {
        virtual_runtime()?.block_on(check_rational_pacing(requests, period_ms, count))?;
    }
}
#[path = "retry_properties/oracle.rs"]
mod oracle;
use oracle::*;
#[path = "retry_properties/tape.rs"]
mod tape;
use tape::*;
#[path = "retry_properties/hints.rs"]
mod hints;
use hints::*;
#[path = "retry_properties/permits.rs"]
mod permits;
use permits::*;
#[path = "retry_properties/pacing.rs"]
mod pacing;
use pacing::*;
