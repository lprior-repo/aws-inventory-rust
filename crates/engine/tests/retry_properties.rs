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

fn retryable_step() -> impl Strategy<Value = Step> {
    prop_oneof![
        Just(Step::Transient),
        Just(Step::Throttled),
        Just(Step::AttemptTimeout),
    ]
}

fn step() -> impl Strategy<Value = Step> {
    prop_oneof![
        Just(Step::Success),
        retryable_step(),
        Just(Step::Permanent),
        Just(Step::Denied),
        Just(Step::Cancelled),
    ]
}

/// The specification is a prefix query: the first terminal response wins, or
/// exactly the caller's attempt budget is consumed. It models no pacing state.
fn oracle(tape: &[Step], budget: usize) -> (usize, Terminal) {
    tape.iter()
        .take(budget)
        .enumerate()
        .find_map(|(index, step)| {
            let terminal = match step {
                Step::Success => Terminal::Success(index),
                Step::Permanent => Terminal::Permanent,
                Step::Denied => Terminal::Denied,
                Step::Cancelled => Terminal::Cancelled,
                _ => return None,
            };
            Some((index.saturating_add(1), terminal))
        })
        .unwrap_or((budget, Terminal::Exhausted))
}

fn terminal(result: &Result<usize, ApiError>) -> Terminal {
    match result {
        Ok(index) => Terminal::Success(*index),
        Err(ApiError::Permanent(_)) => Terminal::Permanent,
        Err(ApiError::Denied(_)) => Terminal::Denied,
        Err(ApiError::Cancelled) => Terminal::Cancelled,
        Err(ApiError::Exhausted(_)) => Terminal::Exhausted,
        Err(_) => Terminal::Unexpected,
    }
}

async fn respond(step: Option<Step>, index: usize) -> Result<usize, ApiError> {
    match step {
        Some(Step::Success) => Ok(index),
        Some(Step::Transient) => Err(ApiError::Transient("synthetic transient".into())),
        Some(Step::Throttled) => Err(ApiError::Throttled("synthetic throttle".into())),
        Some(Step::AttemptTimeout) => pending().await,
        Some(Step::Permanent) => Err(ApiError::Permanent("synthetic permanent".into())),
        Some(Step::Denied) => Err(ApiError::Denied("synthetic denial".into())),
        Some(Step::Cancelled) => Err(ApiError::Cancelled),
        None => Err(ApiError::InvalidConfig("outcome tape was overrun".into())),
    }
}

async fn check_tape(tape: &[Step], budget: u32, seed: u64) -> TestCaseResult {
    let executor = ApiExecutor::with_seed(config(budget), seed).map_err(failure)?;
    let calls = Cell::new(0_usize);
    let (expected_calls, expected_result) = oracle(tape, usize::try_from(budget).map_err(failure)?);
    let result = timeout(
        CASE_DEADLINE,
        executor.execute("script", || {
            let index = calls.get();
            calls.set(index.saturating_add(1));
            respond(tape.get(index).copied(), index)
        }),
    )
    .await
    .map_err(failure)?;
    prop_assert_eq!(terminal(&result), expected_result);
    prop_assert_eq!(calls.get(), expected_calls);
    let stats = executor.stats();
    let attempts = u64::try_from(expected_calls).map_err(failure)?;
    prop_assert_eq!(stats.attempts, attempts);
    prop_assert_eq!(stats.retries, attempts.saturating_sub(1));
    prop_assert_eq!(stats.in_flight, 0);
    prop_assert_eq!(stats.peak_in_flight, 1);
    prop_assert_eq!(stats.active_buckets, 1);
    prop_assert_eq!(
        stats.throttles,
        u64::try_from(
            tape.iter()
                .take(expected_calls)
                .filter(|step| **step == Step::Throttled)
                .count()
        )
        .map_err(failure)?
    );
    prop_assert_eq!(
        stats.timeouts,
        u64::try_from(
            tape.iter()
                .take(expected_calls)
                .filter(|step| **step == Step::AttemptTimeout)
                .count()
        )
        .map_err(failure)?
    );
    prop_assert_eq!(stats.successes, u64::from(result.is_ok()));
    prop_assert_eq!(
        stats.cancelled,
        u64::from(expected_result == Terminal::Cancelled)
    );
    prop_assert_eq!(
        stats.failures,
        u64::from(result.is_err() && expected_result != Terminal::Cancelled)
    );
    prop_assert_eq!(stats.server_delays, 0);
    prop_assert_eq!(stats.rejected_retry_hints, 0);

    // Both reservations must be reusable after every terminal class, including
    // a timed-out pending attempt and operation-originated cancellation.
    for bucket in ["script", "independent"] {
        let followup = timeout(
            CASE_DEADLINE,
            executor.execute(bucket, || future::ready(Ok(71_u8))),
        )
        .await
        .map_err(failure)?
        .map_err(failure)?;
        prop_assert_eq!(followup, 71);
        prop_assert_eq!(executor.stats().in_flight, 0);
    }
    prop_assert_eq!(executor.stats().attempts, attempts.saturating_add(2));
    Ok(())
}

fn kind(index: u8) -> RetryKind {
    match index {
        0 => RetryKind::Throttled,
        1 => RetryKind::Transient,
        _ => RetryKind::Timeout,
    }
}

fn directed(kind: RetryKind, retry_after: RetryAfter) -> ApiError {
    ApiError::RetryAfter {
        kind,
        message: "synthetic server directive".into(),
        retry_after,
    }
}

async fn check_delay(delay_ms: u64, retry_kind: RetryKind, seed: u64) -> TestCaseResult {
    let executor = ApiExecutor::with_seed(config(2), seed).map_err(failure)?;
    let starts = RefCell::new(Vec::new());
    let peer_start = Cell::new(None);
    let delayed = executor.execute("delayed", || {
        let index = starts.borrow().len();
        starts.borrow_mut().push(Instant::now());
        future::ready(if index == 0 {
            Err(directed(
                retry_kind,
                RetryAfter::Delay(Duration::from_millis(delay_ms)),
            ))
        } else {
            Ok(())
        })
    });
    let peer = executor.execute("unrelated", || {
        peer_start.set(Some(Instant::now()));
        future::ready(Ok(()))
    });
    let deadline = Duration::from_millis(delay_ms).saturating_add(CASE_DEADLINE);
    let (delayed, peer) = timeout(deadline, async { tokio::join!(delayed, peer) })
        .await
        .map_err(failure)?;
    delayed.map_err(failure)?;
    peer.map_err(failure)?;
    let starts = starts.into_inner();
    prop_assert_eq!(starts.len(), 2);
    let first = starts
        .first()
        .copied()
        .ok_or_else(|| failure("missing first attempt"))?;
    let second = starts
        .last()
        .copied()
        .ok_or_else(|| failure("missing retry"))?;
    prop_assert!(second.duration_since(first) >= Duration::from_millis(delay_ms));
    let peer = peer_start
        .get()
        .ok_or_else(|| failure("unrelated bucket never ran"))?;
    prop_assert!(peer.duration_since(first) <= Duration::from_millis(1));
    let stats = executor.stats();
    prop_assert_eq!(stats.attempts, 3);
    prop_assert_eq!(stats.retries, 1);
    prop_assert_eq!(stats.server_delays, 1);
    prop_assert_eq!(
        stats.throttles,
        u64::from(retry_kind == RetryKind::Throttled)
    );
    prop_assert_eq!(stats.timeouts, u64::from(retry_kind == RetryKind::Timeout));
    prop_assert_eq!(stats.successes, 2);
    prop_assert_eq!(stats.failures, 0);
    prop_assert_eq!(stats.in_flight, 0);
    Ok(())
}

async fn check_rejected_hint(hint: RetryAfter, retry_kind: RetryKind, seed: u64) -> TestCaseResult {
    let executor = ApiExecutor::with_seed(config(MAX_ATTEMPTS), seed).map_err(failure)?;
    let calls = Cell::new(0_u32);
    for _ in 0..3 {
        let result = executor
            .execute("closed", || {
                calls.set(calls.get().saturating_add(1));
                future::ready(Err::<(), _>(directed(retry_kind, hint)))
            })
            .await;
        prop_assert!(matches!(result, Err(ApiError::RetryAfterRejected(_))));
    }
    prop_assert_eq!(calls.get(), 1);
    executor
        .execute("open", || future::ready(Ok(())))
        .await
        .map_err(failure)?;
    let stats = executor.stats();
    prop_assert_eq!(stats.attempts, 2);
    prop_assert_eq!(stats.retries, 0);
    prop_assert_eq!(stats.rejected_retry_hints, 1);
    prop_assert_eq!(stats.failures, 3);
    prop_assert_eq!(stats.successes, 1);
    prop_assert_eq!(stats.in_flight, 0);
    Ok(())
}

fn occupancy(assignments: &[bool], global: usize, local: usize) -> usize {
    let first = assignments
        .iter()
        .filter(|value| **value)
        .count()
        .min(local);
    let second = assignments
        .iter()
        .filter(|value| !**value)
        .count()
        .min(local);
    first.saturating_add(second).min(global)
}

async fn occupy_then_release(
    executor: &ApiExecutor,
    assignments: &[bool],
    expected: usize,
    cancel: bool,
) -> TestCaseResult {
    let before = executor.stats();
    let starts = Cell::new(0_usize);
    let mut calls: Vec<_> = assignments
        .iter()
        .map(|first| {
            Box::pin(
                executor.execute(if *first { "first" } else { "second" }, || {
                    starts.set(starts.get().saturating_add(1));
                    pending::<Result<(), ApiError>>()
                }),
            )
        })
        .collect();
    // At most eight callers and nine polling rounds. A millisecond of virtual
    // time admits the next paced sender without approaching attempt timeout.
    for _ in 0..9 {
        advance(Duration::from_millis(1)).await;
        for call in &mut calls {
            prop_assert!(call.as_mut().now_or_never().is_none());
        }
    }
    prop_assert_eq!(starts.get(), expected);
    prop_assert_eq!(
        executor.stats().in_flight,
        u64::try_from(expected).map_err(failure)?
    );
    if cancel {
        executor.cancel();
        let outcomes = timeout(Duration::from_secs(1), future::join_all(calls))
            .await
            .map_err(failure)?;
        prop_assert!(
            outcomes
                .iter()
                .all(|result| *result == Err(ApiError::Cancelled))
        );
    } else {
        drop(calls);
    }
    let after = executor.stats();
    prop_assert_eq!(after.in_flight, 0);
    prop_assert_eq!(
        after.cancelled.saturating_sub(before.cancelled),
        u64::try_from(assignments.len()).map_err(failure)?
    );
    prop_assert_eq!(after.failures, before.failures);
    prop_assert_eq!(
        after.attempts.saturating_sub(before.attempts),
        u64::try_from(expected).map_err(failure)?
    );
    Ok(())
}

async fn check_permits(
    global: usize,
    local: usize,
    assignments: &[bool],
    cancel: bool,
) -> TestCaseResult {
    let executor = ApiExecutor::with_seed(
        ExecutorConfig {
            global_concurrency: global,
            per_bucket_concurrency: local,
            attempt_timeout_ms: 5_000,
            requests_per_second: 100_000,
            ..config(1)
        },
        19,
    )
    .map_err(failure)?;
    let expected = occupancy(assignments, global, local);
    occupy_then_release(&executor, assignments, expected, cancel).await?;
    if cancel {
        let called = Cell::new(false);
        let result = executor
            .execute("new", || {
                called.set(true);
                future::ready(Ok(()))
            })
            .await;
        prop_assert_eq!(result, Err(ApiError::Cancelled));
        prop_assert!(!called.get());
    } else {
        // Reattaining the entire expected occupancy catches lost permits even
        // when a leaked permit would still leave room for one successful call.
        occupy_then_release(&executor, assignments, expected, false).await?;
    }
    prop_assert_eq!(executor.stats().in_flight, 0);
    prop_assert!(executor.stats().peak_in_flight <= u64::try_from(global).map_err(failure)?);
    Ok(())
}

async fn check_rational_pacing(requests: u32, period_ms: u64, count: usize) -> TestCaseResult {
    let executor = ApiExecutor::with_rate_policies(
        ExecutorConfig {
            global_concurrency: 4,
            per_bucket_concurrency: 4,
            ..config(1)
        },
        BTreeMap::from([(
            "paced".to_owned(),
            BucketPolicy::PerPeriod(PeriodLimits {
                requests,
                period_ms,
                concurrency: 4,
            }),
        )]),
    )
    .map_err(failure)?;
    let starts = RefCell::new(Vec::new());
    let outcomes = timeout(
        CASE_DEADLINE,
        stream::iter(0..count)
            .map(|_| {
                executor.execute("paced", || {
                    starts.borrow_mut().push(Instant::now());
                    future::ready(Ok(()))
                })
            })
            .buffer_unordered(4)
            .collect::<Vec<_>>(),
    )
    .await
    .map_err(failure)?;
    prop_assert!(outcomes.iter().all(Result::is_ok));
    let starts = starts.into_inner();
    prop_assert_eq!(starts.len(), count);
    for (first, second) in starts.iter().zip(starts.iter().skip(1)) {
        // Multiplication is an independent rate inequality; it does not copy
        // the implementation's ceiling-division or adaptive-rate algorithm.
        let elapsed = second.duration_since(*first).as_nanos();
        prop_assert!(
            elapsed.saturating_mul(u128::from(requests))
                >= u128::from(period_ms).saturating_mul(1_000_000)
        );
    }
    prop_assert_eq!(
        executor.stats().attempts,
        u64::try_from(count).map_err(failure)?
    );
    prop_assert_eq!(executor.stats().in_flight, 0);
    Ok(())
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
