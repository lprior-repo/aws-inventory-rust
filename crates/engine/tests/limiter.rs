use std::collections::BTreeMap;
use std::error::Error;
use std::future::{Future, pending};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use futures_util::{StreamExt, TryStreamExt, stream};
use inventory_engine::limiter::{
    AdaptiveLimits, ApiError, ApiExecutor, BucketLimits, BucketPolicy, ExecutorConfig, MAX_BUCKETS,
    MAX_RATE_PERIOD_MS, MAX_RETRY_AFTER_MS, PeriodLimits, RetryAfter, RetryKind,
};
use tokio::sync::Barrier;
use tokio::time::{Instant, sleep, timeout};

type TestResult = Result<(), Box<dyn Error>>;

async fn bounded_barrier_case<T>(
    future: impl Future<Output = Result<T, ApiError>>,
) -> Result<T, Box<dyn Error>> {
    timeout(Duration::from_secs(30), future)
        .await
        .map_err(|_| "engine limiter barrier test exceeded its 30-second virtual deadline")?
        .map_err(Into::into)
}

fn require_error(
    result: Result<(), ApiError>,
    expected: fn(&ApiError) -> bool,
) -> Result<(), ApiError> {
    match result {
        Err(error) if expected(&error) => Ok(()),
        Err(error) => Err(error),
        Ok(()) => Err(ApiError::Permanent(
            "expected the exercised API failure".into(),
        )),
    }
}

fn config() -> ExecutorConfig {
    ExecutorConfig {
        global_concurrency: 4,
        per_bucket_concurrency: 4,
        requests_per_second: 10,
        max_attempts: 3,
        attempt_timeout_ms: 5_000,
        max_backoff_ms: 1,
    }
}

fn record(starts: &Mutex<Vec<Instant>>) -> Result<(), ApiError> {
    starts
        .lock()
        .map_err(|_| ApiError::Permanent("test start recorder poisoned".into()))?
        .push(Instant::now());
    Ok(())
}

fn recorded(starts: &Mutex<Vec<Instant>>) -> Result<Vec<Instant>, ApiError> {
    Ok(starts
        .lock()
        .map_err(|_| ApiError::Permanent("test start recorder poisoned".into()))?
        .clone())
}

fn spaced(starts: &[Instant], minimum: Duration) {
    for (first, second) in starts.iter().zip(starts.iter().skip(1)) {
        assert!(second.duration_since(*first) >= minimum);
    }
}

fn period_policy(requests: u32, period_ms: u64, concurrency: usize) -> BucketPolicy {
    BucketPolicy::PerPeriod(PeriodLimits {
        requests,
        period_ms,
        concurrency,
    })
}

fn directed(kind: RetryKind, retry_after: RetryAfter) -> ApiError {
    ApiError::RetryAfter {
        kind,
        message: "fixture server delay".into(),
        retry_after,
    }
}

#[path = "limiter/accepted_hint_boundary_and_retry_exhaustion_remain_bounded.rs"]
mod accepted_hint_boundary_and_retry_exhaustion_remain_bounded;
#[path = "limiter/adaptive_policy.rs"]
mod adaptive_policy;
#[path = "limiter/adaptive_recovery_stops_at_the_bucket_policy_ceiling.rs"]
mod adaptive_recovery_stops_at_the_bucket_policy_ceiling;
#[path = "limiter/backoff_holds_no_global_permit.rs"]
mod backoff_holds_no_global_permit;
#[path = "limiter/bucket_keys_have_a_hard_byte_bound.rs"]
mod bucket_keys_have_a_hard_byte_bound;
#[path = "limiter/bucket_policies_validate_limits_and_exact_key_bounds.rs"]
mod bucket_policies_validate_limits_and_exact_key_bounds;
#[path = "limiter/bucket_policy_json_rejects_unknown_fields.rs"]
mod bucket_policy_json_rejects_unknown_fields;
#[path = "limiter/bucket_policy_map_has_a_hard_entry_bound.rs"]
mod bucket_policy_map_has_a_hard_entry_bound;
#[path = "limiter/cancellation_is_observable_before_and_after_the_signal.rs"]
mod cancellation_is_observable_before_and_after_the_signal;
#[path = "limiter/cancellation_wakes_active_calls_and_global_waiters.rs"]
mod cancellation_wakes_active_calls_and_global_waiters;
#[path = "limiter/cancellation_wakes_backoff_before_another_attempt.rs"]
mod cancellation_wakes_backoff_before_another_attempt;
#[path = "limiter/cancellation_wakes_rate_and_local_reservation_waiters.rs"]
mod cancellation_wakes_rate_and_local_reservation_waiters;
#[path = "limiter/cancelling_a_server_cooldown_wakes_all_bucket_waiters.rs"]
mod cancelling_a_server_cooldown_wakes_all_bucket_waiters;
#[path = "limiter/dropping_a_call_releases_capacity_and_records_cancellation.rs"]
mod dropping_a_call_releases_capacity_and_records_cancellation;
#[path = "limiter/every_concurrent_start_obeys_bucket_rate.rs"]
mod every_concurrent_start_obeys_bucket_rate;
#[path = "limiter/eviction.rs"]
mod eviction;
#[path = "limiter/final_attempt_retry_hint_delays_queued_peers.rs"]
mod final_attempt_retry_hint_delays_queued_peers;
#[path = "limiter/fractional_and_sub_one_rps_policies_pace_every_concurrent_start.rs"]
mod fractional_and_sub_one_rps_policies_pace_every_concurrent_start;
#[path = "limiter/fractional_rates_round_up_instead_of_exceeding_budget.rs"]
mod fractional_rates_round_up_instead_of_exceeding_budget;
#[path = "limiter/fractional_throttle_recovery_never_exceeds_the_original_ceiling.rs"]
mod fractional_throttle_recovery_never_exceeds_the_original_ceiling;
#[path = "limiter/global_contention_cannot_release_accumulated_rate_tickets_as_a_burst.rs"]
mod global_contention_cannot_release_accumulated_rate_tickets_as_a_burst;
#[path = "limiter/independent_buckets_reach_but_never_exceed_global_concurrency.rs"]
mod independent_buckets_reach_but_never_exceed_global_concurrency;
#[path = "limiter/invalid_and_excessive_hints_close_only_the_affected_bucket.rs"]
mod invalid_and_excessive_hints_close_only_the_affected_bucket;
#[path = "limiter/local_concurrency_is_independent_of_the_larger_global_limit.rs"]
mod local_concurrency_is_independent_of_the_larger_global_limit;
#[path = "limiter/pacing_one_bucket_does_not_occupy_global_capacity.rs"]
mod pacing_one_bucket_does_not_occupy_global_capacity;
#[path = "limiter/policies_can_raise_or_lower_local_concurrency_within_global_capacity.rs"]
mod policies_can_raise_or_lower_local_concurrency_within_global_capacity;
#[path = "limiter/policies_match_exactly_and_preserve_default_rates.rs"]
mod policies_match_exactly_and_preserve_default_rates;
#[path = "limiter/pre_cancelled_executor_never_invokes_the_operation.rs"]
mod pre_cancelled_executor_never_invokes_the_operation;
#[path = "limiter/rational_policy_json_preserves_integer_shape_and_rejects_mixed_or_unknown_fields.rs"]
mod rational_policy_json_preserves_integer_shape_and_rejects_mixed_or_unknown_fields;
#[path = "limiter/rational_policy_maps_accept_the_boundary_and_never_truncate_overflow.rs"]
mod rational_policy_maps_accept_the_boundary_and_never_truncate_overflow;
#[path = "limiter/rational_policy_validates_fractional_floor_ceiling_and_payload_bounds.rs"]
mod rational_policy_validates_fractional_floor_ceiling_and_payload_bounds;
#[path = "limiter/rejected_hint_cancels_already_active_same_bucket_operations.rs"]
mod rejected_hint_cancels_already_active_same_bucket_operations;
#[path = "limiter/rejected_hint_wakes_a_peer_already_queued_for_global_capacity.rs"]
mod rejected_hint_wakes_a_peer_already_queued_for_global_capacity;
#[path = "limiter/rejected_hint_wakes_existing_cooldown_and_local_capacity_waiters.rs"]
mod rejected_hint_wakes_existing_cooldown_and_local_capacity_waiters;
#[path = "limiter/retries_consume_the_same_rate_budget_as_initial_calls.rs"]
mod retries_consume_the_same_rate_budget_as_initial_calls;
#[path = "limiter/retry_exhaustion_stops_at_the_configured_attempt_count.rs"]
mod retry_exhaustion_stops_at_the_configured_attempt_count;
#[path = "limiter/retry_jitter_is_reproducible_and_bounded.rs"]
mod retry_jitter_is_reproducible_and_bounded;
#[path = "limiter/server_hint_during_global_wait_is_rechecked_without_holding_a_global_slot.rs"]
mod server_hint_during_global_wait_is_rechecked_without_holding_a_global_slot;
#[path = "limiter/shared_cooldown_releases_global_capacity_and_retries_keep_their_budget.rs"]
mod shared_cooldown_releases_global_capacity_and_retries_keep_their_budget;
#[path = "limiter/successive_server_hints_extend_but_never_shorten_shared_cooldowns.rs"]
mod successive_server_hints_extend_but_never_shorten_shared_cooldowns;
#[path = "limiter/terminal_errors_are_not_retried.rs"]
mod terminal_errors_are_not_retried;
#[path = "limiter/throttle_feedback_during_global_wait_rechecks_pacing_without_a_held_permit.rs"]
mod throttle_feedback_during_global_wait_rechecks_pacing_without_a_held_permit;
#[path = "limiter/throttling_halves_rate_and_success_recovers_additively.rs"]
mod throttling_halves_rate_and_success_recovers_additively;
#[path = "limiter/timed_out_attempts_are_cancelled_retried_and_bounded.rs"]
mod timed_out_attempts_are_cancelled_retried_and_bounded;
#[path = "limiter/validates_every_configuration_bound.rs"]
mod validates_every_configuration_bound;
