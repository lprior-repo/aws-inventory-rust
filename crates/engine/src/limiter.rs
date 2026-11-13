//! Bounded admission and retries for cancellation-safe, read-only API calls.
//!
//! A bucket is an account/region/quota family, which can contain one or more
//! API operations according to the service's actual throttling rules.
//! The registry is bounded and locked only during lookup/registration. Each
//! bucket has a fair dispatch reservation: only its next sender may wait for
//! global capacity. No mutex guard crosses an await, and global capacity is
//! never retained during pacing or backoff. This module spawns no tasks.

use std::collections::{BTreeMap, HashMap, hash_map::RandomState};
use std::future::Future;
use std::hash::BuildHasher;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::{Semaphore, SemaphorePermit};
use tokio::time::{Instant, sleep, sleep_until, timeout};
use tokio_util::sync::CancellationToken;

/// Hard admission bounds, independent of caller configuration.
pub const MAX_CONCURRENCY: usize = 4_096;
pub const MAX_ATTEMPTS: u32 = 10;
pub const MAX_REQUESTS_PER_SECOND: u32 = 100_000;
pub const MAX_DURATION_MS: u64 = 300_000;
pub const MAX_RATE_PERIOD_MS: u64 = 300_000;
pub const MAX_RETRY_AFTER_MS: u64 = 300_000;
pub const MAX_BUCKETS: usize = 65_536;
pub const MAX_PENDING_OPERATIONS: usize = 65_536;
pub const MAX_BUCKET_KEY_BYTES: usize = 512;
const MAX_ADMISSION_ROUNDS: usize = MAX_CONCURRENCY.saturating_add(1);
const BASE_BACKOFF_MS: u64 = 100;
const MAX_ERROR_CHARS: usize = 1_024;
const NANOS_PER_SECOND: u64 = 1_000_000_000;
const NANOS_PER_MILLISECOND: u64 = 1_000_000;

/// Classification retained when a service supplies its own retry deadline.
#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum RetryKind {
    #[error("throttled")]
    Throttled,
    #[error("transient failure")]
    Transient,
    #[error("timeout")]
    Timeout,
}

/// A parsed server directive. Invalid and excessive hints must not be dropped
/// by an adapter: the executor closes that bucket instead of retrying early.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetryAfter {
    Delay(Duration),
    Invalid,
    Excessive,
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum ApiError {
    #[error("AWS throttled the request: {0}")]
    Throttled(String),
    #[error("transient API failure: {0}")]
    Transient(String),
    #[error("access denied: {0}")]
    Denied(String),
    #[error("permanent API failure: {0}")]
    Permanent(String),
    #[error("API attempt timed out")]
    Timeout,
    #[error("API execution cancelled")]
    Cancelled,
    #[error("API execution budget exhausted: {0}")]
    Exhausted(String),
    #[error("invalid executor configuration: {0}")]
    InvalidConfig(String),
    #[error("{kind}: {message} (server retry directive: {retry_after:?})")]
    RetryAfter {
        kind: RetryKind,
        message: String,
        retry_after: RetryAfter,
    },
    #[error("server retry directive rejected; bucket closed: {0}")]
    RetryAfterRejected(String),
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct ExecutorConfig {
    pub global_concurrency: usize,
    pub per_bucket_concurrency: usize,
    /// Maximum starts per second per bucket, including retries.
    pub requests_per_second: u32,
    pub max_attempts: u32,
    pub attempt_timeout_ms: u64,
    pub max_backoff_ms: u64,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            global_concurrency: 64,
            per_bucket_concurrency: 8,
            requests_per_second: 5,
            max_attempts: 5,
            attempt_timeout_ms: 30_000,
            max_backoff_ms: 30_000,
        }
    }
}

impl ExecutorConfig {
    fn validate(self) -> Result<Self, ApiError> {
        if !(1..=MAX_CONCURRENCY).contains(&self.global_concurrency)
            || !(1..=self.global_concurrency).contains(&self.per_bucket_concurrency)
        {
            return Err(ApiError::InvalidConfig(
                "require 1 <= per_bucket_concurrency <= global_concurrency <= 4096".into(),
            ));
        }
        if !(1..=MAX_REQUESTS_PER_SECOND).contains(&self.requests_per_second)
            || !(1..=MAX_ATTEMPTS).contains(&self.max_attempts)
        {
            return Err(ApiError::InvalidConfig(
                "require RPS in 1..=100000 and attempts in 1..=10".into(),
            ));
        }
        if !(1..=MAX_DURATION_MS).contains(&self.attempt_timeout_ms)
            || !(1..=MAX_DURATION_MS).contains(&self.max_backoff_ms)
        {
            return Err(ApiError::InvalidConfig(
                "timeout and backoff must be in 1..=300000 milliseconds".into(),
            ));
        }
        Ok(self)
    }
}

/// Immutable limits for one exact account/region/quota-family key.
/// Other keys retain the executor defaults; no wildcard expansion is performed.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BucketLimits {
    pub requests_per_second: u32,
    pub concurrency: usize,
}

impl BucketLimits {
    fn validate(self, global_concurrency: usize) -> Result<(), ApiError> {
        if !(1..=MAX_REQUESTS_PER_SECOND).contains(&self.requests_per_second)
            || !(1..=global_concurrency).contains(&self.concurrency)
        {
            return Err(ApiError::InvalidConfig(
                "bucket policy requires RPS in 1..=100000 and concurrency in 1..=global_concurrency".into(),
            ));
        }
        Ok(())
    }

    fn defaults(config: ExecutorConfig) -> Self {
        Self {
            requests_per_second: config.requests_per_second,
            concurrency: config.per_bucket_concurrency,
        }
    }
}

/// Exact requests-per-period ceiling, evenly spaced rather than burst tokens.
/// The minimum supported rate is one request per 300,000 milliseconds.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PeriodLimits {
    pub requests: u32,
    pub period_ms: u64,
    pub concurrency: usize,
}

/// Accepts the existing integer JSON shape or an exact rational rate. Both
/// shapes reject unknown fields, so mixed policies cannot silently fall back.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(untagged)]
pub enum BucketPolicy {
    PerSecond(BucketLimits),
    PerPeriod(PeriodLimits),
}

impl From<BucketLimits> for BucketPolicy {
    fn from(limits: BucketLimits) -> Self {
        Self::PerSecond(limits)
    }
}

#[derive(Clone, Copy, Debug)]
struct ValidatedLimits {
    concurrency: usize,
    rate: AdaptiveRate,
}

impl BucketPolicy {
    fn validate(self, global_concurrency: usize) -> Result<ValidatedLimits, ApiError> {
        match self {
            Self::PerSecond(limits) => {
                limits.validate(global_concurrency)?;
                Ok(ValidatedLimits {
                    concurrency: limits.concurrency,
                    rate: AdaptiveRate::PerSecond {
                        current: limits.requests_per_second,
                        maximum: limits.requests_per_second,
                    },
                })
            }
            Self::PerPeriod(limits) => limits.validate(global_concurrency),
        }
    }
}

impl PeriodLimits {
    fn validate(self, global_concurrency: usize) -> Result<ValidatedLimits, ApiError> {
        if !(1..=MAX_REQUESTS_PER_SECOND).contains(&self.requests)
            || !(1..=MAX_RATE_PERIOD_MS).contains(&self.period_ms)
            || !(1..=global_concurrency).contains(&self.concurrency)
        {
            return Err(ApiError::InvalidConfig(
                "period policy requires requests in 1..=100000, period_ms in 1..=300000, and concurrency in 1..=global_concurrency".into(),
            ));
        }
        let numerator = u64::from(self.requests).checked_mul(1_000);
        let maximum = u64::from(MAX_REQUESTS_PER_SECOND).checked_mul(self.period_ms);
        if numerator.is_none() || maximum.is_none() || numerator > maximum {
            return Err(ApiError::InvalidConfig(
                "period policy exceeds 100000 requests per second".into(),
            ));
        }
        let period_ns = self
            .period_ms
            .checked_mul(NANOS_PER_MILLISECOND)
            .ok_or_else(|| ApiError::InvalidConfig("rate period overflow".into()))?;
        let interval_ns = ceil_div(period_ns, u64::from(self.requests))?;
        Ok(ValidatedLimits {
            concurrency: self.concurrency,
            rate: AdaptiveRate::PerPeriod {
                interval_ns,
                minimum_ns: interval_ns,
                recovery_rps: 0,
            },
        })
    }
}

fn validate_key(key: &str) -> Result<(), ApiError> {
    if key.is_empty() || key.len() > MAX_BUCKET_KEY_BYTES {
        return Err(ApiError::InvalidConfig(
            "bucket key must be 1..=512 bytes".into(),
        ));
    }
    Ok(())
}

fn validate_policies(
    config: ExecutorConfig,
    policies: BTreeMap<String, BucketPolicy>,
) -> Result<BTreeMap<String, ValidatedLimits>, ApiError> {
    if policies.len() > MAX_BUCKETS {
        return Err(ApiError::InvalidConfig(
            "at most 65536 bucket policies are permitted".into(),
        ));
    }
    policies
        .into_iter()
        .take(MAX_BUCKETS)
        .map(|(key, limits)| {
            validate_key(&key)?;
            Ok((key, limits.validate(config.global_concurrency)?))
        })
        .collect()
}

/// Cumulative counters and current occupancy. Concurrent reads are approximate;
/// after all caller-owned futures have completed/dropped, the snapshot is final.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub struct ExecutorStats {
    pub attempts: u64,
    pub retries: u64,
    pub throttles: u64,
    pub server_delays: u64,
    pub rejected_retry_hints: u64,
    pub timeouts: u64,
    pub successes: u64,
    pub failures: u64,
    pub cancelled: u64,
    pub in_flight: u64,
    pub peak_in_flight: u64,
    pub active_buckets: u64,
}

#[derive(Debug, Default)]
struct Counters {
    attempts: AtomicU64,
    retries: AtomicU64,
    throttles: AtomicU64,
    server_delays: AtomicU64,
    rejected_retry_hints: AtomicU64,
    timeouts: AtomicU64,
    successes: AtomicU64,
    failures: AtomicU64,
    cancelled: AtomicU64,
    in_flight: AtomicU64,
    peak_in_flight: AtomicU64,
    active_buckets: AtomicU64,
}

impl Counters {
    fn snapshot(&self) -> ExecutorStats {
        ExecutorStats {
            attempts: self.attempts.load(Ordering::Relaxed),
            retries: self.retries.load(Ordering::Relaxed),
            throttles: self.throttles.load(Ordering::Relaxed),
            server_delays: self.server_delays.load(Ordering::Relaxed),
            rejected_retry_hints: self.rejected_retry_hints.load(Ordering::Relaxed),
            timeouts: self.timeouts.load(Ordering::Relaxed),
            successes: self.successes.load(Ordering::Relaxed),
            failures: self.failures.load(Ordering::Relaxed),
            cancelled: self.cancelled.load(Ordering::Relaxed),
            in_flight: self.in_flight.load(Ordering::Relaxed),
            peak_in_flight: self.peak_in_flight.load(Ordering::Relaxed),
            active_buckets: self.active_buckets.load(Ordering::Relaxed),
        }
    }
}

fn increment(counter: &AtomicU64) -> u64 {
    counter
        .update(Ordering::Relaxed, Ordering::Relaxed, |value| {
            value.saturating_add(1)
        })
        .saturating_add(1)
}

fn ceil_div(numerator: u64, denominator: u64) -> Result<u64, ApiError> {
    numerator
        .checked_add(denominator.saturating_sub(1))
        .and_then(|value| value.checked_div(denominator))
        .ok_or_else(|| ApiError::Permanent("invalid internal rate arithmetic".into()))
}

#[derive(Clone, Copy, Debug)]
enum AdaptiveRate {
    PerSecond {
        current: u32,
        maximum: u32,
    },
    PerPeriod {
        interval_ns: u64,
        minimum_ns: u64,
        recovery_rps: u32,
    },
}

impl AdaptiveRate {
    fn interval_ns(self) -> Result<u64, ApiError> {
        match self {
            Self::PerSecond { current, .. } => ceil_div(NANOS_PER_SECOND, u64::from(current)),
            Self::PerPeriod {
                interval_ns,
                minimum_ns,
                recovery_rps,
            } => period_interval(interval_ns, minimum_ns, recovery_rps),
        }
    }

    fn throttle(&mut self) -> Result<(), ApiError> {
        let previous_interval = self.interval_ns()?;
        match self {
            Self::PerSecond { current, .. } => *current = current.saturating_div(2).max(1),
            Self::PerPeriod {
                interval_ns,
                recovery_rps,
                ..
            } => {
                *interval_ns = previous_interval
                    .saturating_mul(2)
                    .min(MAX_RATE_PERIOD_MS.saturating_mul(NANOS_PER_MILLISECOND));
                *recovery_rps = 0;
            }
        }
        Ok(())
    }

    fn recover(&mut self) {
        match self {
            Self::PerSecond { current, maximum } => {
                *current = current.saturating_add(1).min(*maximum)
            }
            Self::PerPeriod { recovery_rps, .. } => {
                // Retaining +1 RPS credits prevents nanosecond rounding from
                // discarding every increment at high configured rates.
                *recovery_rps = recovery_rps.saturating_add(1).min(MAX_REQUESTS_PER_SECOND);
            }
        }
    }
}

fn period_interval(base_ns: u64, minimum_ns: u64, recovery_rps: u32) -> Result<u64, ApiError> {
    let numerator = u128::from(NANOS_PER_SECOND).checked_mul(u128::from(base_ns));
    let denominator = u128::from(base_ns)
        .checked_mul(u128::from(recovery_rps))
        .and_then(|increment| u128::from(NANOS_PER_SECOND).checked_add(increment));
    let next = numerator
        .zip(denominator)
        .and_then(|(numerator, denominator)| {
            numerator
                .checked_add(denominator.saturating_sub(1))
                .and_then(|value| value.checked_div(denominator))
        })
        .and_then(|value| u64::try_from(value).ok())
        .ok_or_else(|| ApiError::Permanent("adaptive rate arithmetic overflow".into()))?;
    Ok(next.max(minimum_ns))
}

#[derive(Clone, Copy, Debug)]
enum RejectedHint {
    Invalid,
    Excessive,
}

impl RejectedHint {
    fn detail(self) -> &'static str {
        match self {
            Self::Invalid => "invalid server retry hint",
            Self::Excessive => {
                "server retry delay exceeds 300000 milliseconds or the monotonic clock range"
            }
        }
    }

    fn error(self) -> ApiError {
        ApiError::RetryAfterRejected(self.detail().into())
    }
}

#[derive(Debug)]
struct PacingState {
    last_start: Option<Instant>,
    cooldown_until: Option<Instant>,
    rejected: Option<RejectedHint>,
    rate: AdaptiveRate,
}

impl PacingState {
    fn due_at(&self) -> Result<Option<Instant>, ApiError> {
        if let Some(rejected) = self.rejected {
            return Err(rejected.error());
        }
        let interval = Duration::from_nanos(self.rate.interval_ns()?);
        let paced = self
            .last_start
            .map(|last| {
                last.checked_add(interval)
                    .ok_or_else(|| ApiError::Permanent("pacing deadline overflow".into()))
            })
            .transpose()?;
        Ok(paced.max(self.cooldown_until))
    }

    fn apply_hint(&mut self, hint: RetryAfter, now: Instant) -> Result<(), RejectedHint> {
        let delay = match hint {
            RetryAfter::Delay(delay) if delay <= Duration::from_millis(MAX_RETRY_AFTER_MS) => delay,
            RetryAfter::Delay(_) | RetryAfter::Excessive => return Err(RejectedHint::Excessive),
            RetryAfter::Invalid => return Err(RejectedHint::Invalid),
        };
        let deadline = now.checked_add(delay).ok_or(RejectedHint::Excessive)?;
        self.cooldown_until = self.cooldown_until.max(Some(deadline));
        Ok(())
    }

    fn feedback<T>(&mut self, result: &Result<T, ApiError>, now: Instant) -> Result<(), ApiError> {
        if result.is_ok() {
            self.rate.recover();
        } else if result.as_ref().err().and_then(retry_kind) == Some(RetryKind::Throttled) {
            self.rate.throttle()?;
        }
        if let Err(ApiError::RetryAfter {
            kind,
            message,
            retry_after,
        }) = result
            && let Err(reason) = self.apply_hint(*retry_after, now)
        {
            self.rejected = Some(reason);
            return Err(ApiError::RetryAfterRejected(format!(
                "{}; {kind}: {}",
                reason.detail(),
                bounded_message(message)
            )));
        }
        Ok(())
    }
}

#[derive(Debug)]
struct Bucket {
    in_flight: Semaphore,
    dispatch: Semaphore,
    state: Mutex<PacingState>,
    rejected: CancellationToken,
}

impl Bucket {
    fn new(limits: ValidatedLimits) -> Self {
        Self {
            in_flight: Semaphore::new(limits.concurrency),
            dispatch: Semaphore::new(1),
            state: Mutex::new(PacingState {
                last_start: None,
                cooldown_until: None,
                rejected: None,
                rate: limits.rate,
            }),
            rejected: CancellationToken::new(),
        }
    }

    fn due_at(&self) -> Result<Option<Instant>, ApiError> {
        self.state.lock().map_err(|_| poisoned_pacing())?.due_at()
    }

    fn try_start(&self, now: Instant) -> Result<bool, ApiError> {
        let mut state = self.state.lock().map_err(|_| poisoned_pacing())?;
        if state.due_at()?.is_some_and(|due| due > now) {
            return Ok(false);
        }
        state.last_start = Some(now);
        Ok(true)
    }

    fn feedback<T>(&self, result: &Result<T, ApiError>) -> Result<(), ApiError> {
        let outcome = self
            .state
            .lock()
            .map_err(|_| poisoned_pacing())?
            .feedback(result, Instant::now());
        if matches!(outcome, Err(ApiError::RetryAfterRejected(_))) {
            self.rejected.cancel();
        }
        outcome
    }

    fn rejection_error(&self) -> ApiError {
        match self.state.lock() {
            Ok(state) => state.rejected.map_or_else(
                || ApiError::Permanent("missing bucket rejection cause".into()),
                RejectedHint::error,
            ),
            Err(_) => poisoned_pacing(),
        }
    }
}

fn poisoned_pacing() -> ApiError {
    ApiError::Permanent("bucket pacing state poisoned".into())
}

#[derive(Debug)]
pub struct ApiExecutor {
    config: ExecutorConfig,
    global: Semaphore,
    pending: Semaphore,
    buckets: Mutex<HashMap<String, Arc<Bucket>>>,
    policies: BTreeMap<String, ValidatedLimits>,
    cancellation: CancellationToken,
    counters: Counters,
    jitter_seed: u64,
    nonce: AtomicU64,
}

impl ApiExecutor {
    pub fn new(config: ExecutorConfig) -> Result<Self, ApiError> {
        Self::with_seed(config, RandomState::new().hash_one("inventory-api-jitter"))
    }

    /// Supply a seed for reproducible retry schedules in tests/replay.
    pub fn with_seed(config: ExecutorConfig, jitter_seed: u64) -> Result<Self, ApiError> {
        Self::build(config, BTreeMap::new(), jitter_seed)
    }

    /// Apply validated, immutable limits to exact bucket keys. This does not
    /// discover AWS quotas or share limits with other processes.
    pub fn with_bucket_policies(
        config: ExecutorConfig,
        policies: BTreeMap<String, BucketLimits>,
    ) -> Result<Self, ApiError> {
        if policies.len() > MAX_BUCKETS {
            return Err(ApiError::InvalidConfig(
                "at most 65536 bucket policies are permitted".into(),
            ));
        }
        Self::with_rate_policies(
            config,
            policies
                .into_iter()
                .take(MAX_BUCKETS)
                .map(|(key, limits)| (key, BucketPolicy::from(limits)))
                .collect(),
        )
    }

    /// Add exact requests-per-period policies while preserving the integer
    /// policy shape. All policy validation precedes request admission.
    pub fn with_rate_policies(
        config: ExecutorConfig,
        policies: BTreeMap<String, BucketPolicy>,
    ) -> Result<Self, ApiError> {
        Self::build(
            config,
            policies,
            RandomState::new().hash_one("inventory-api-jitter"),
        )
    }

    fn build(
        config: ExecutorConfig,
        policies: BTreeMap<String, BucketPolicy>,
        jitter_seed: u64,
    ) -> Result<Self, ApiError> {
        let config = config.validate()?;
        let policies = validate_policies(config, policies)?;
        Ok(Self {
            config,
            global: Semaphore::new(config.global_concurrency),
            pending: Semaphore::new(MAX_PENDING_OPERATIONS),
            buckets: Mutex::new(HashMap::new()),
            policies,
            cancellation: CancellationToken::new(),
            counters: Counters::default(),
            jitter_seed,
            nonce: AtomicU64::new(0),
        })
    }

    /// Cancel all active operations and admission/backoff waits. Cancellation is
    /// permanent for this executor. The caller owns and must drain its futures.
    pub fn cancel(&self) {
        self.cancellation.cancel();
    }

    /// Check cancellation before admitting orchestration work outside API calls.
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.cancellation.is_cancelled()
    }

    /// Observe cancellation during configuration or other caller-owned waits.
    /// The returned future owns no task and is safe to drop.
    #[tracing::instrument(skip_all)]
    pub async fn cancelled(&self) {
        self.cancellation.cancelled().await;
    }

    #[must_use]
    pub fn stats(&self) -> ExecutorStats {
        self.counters.snapshot()
    }

    /// Execute a repeatable, drop-safe, nonblocking operation; SDK retries must
    /// be disabled. Tokio timeouts cannot preempt blocking future polling.
    /// This function does not buffer `T`; payload limits belong to the adapter.
    /// Dropping this future releases every reservation and counts cancellation.
    #[tracing::instrument(skip(self, operation), fields(quota_bucket = bucket))]
    pub async fn execute<T, F, Fut>(&self, bucket: &str, operation: F) -> Result<T, ApiError>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<T, ApiError>>,
    {
        let mut record = RequestRecord::new(&self.counters);
        let result = self.execute_inner(bucket, operation).await;
        record.complete(&result);
        result
    }

    #[tracing::instrument(skip_all)]
    async fn execute_inner<T, F, Fut>(&self, key: &str, mut operation: F) -> Result<T, ApiError>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<T, ApiError>>,
    {
        self.ensure_running()?;
        let _pending = self.pending.try_acquire().map_err(|_| {
            ApiError::Exhausted("executor pending-operation capacity reached".into())
        })?;
        let bucket = self.bucket(key)?;
        // Wrapping is intentional for a jitter nonce, not a resource counter.
        let nonce = self.nonce.fetch_add(1, Ordering::Relaxed);
        for attempt in 0..MAX_ATTEMPTS {
            if attempt >= self.config.max_attempts {
                break;
            }
            let result = self.attempt(&bucket, attempt, &mut operation).await;
            match result {
                Ok(value) => return Ok(value),
                Err(error) if retryable(&error) => {
                    if attempt.saturating_add(1) == self.config.max_attempts {
                        return Err(exhausted(error));
                    }
                    self.backoff(&bucket, nonce, attempt).await?;
                }
                Err(error) => return Err(error),
            }
        }
        Err(ApiError::Exhausted("static attempt ceiling reached".into()))
    }

    fn bucket(&self, key: &str) -> Result<Arc<Bucket>, ApiError> {
        validate_key(key)?;
        let mut buckets = self
            .buckets
            .lock()
            .map_err(|_| ApiError::Permanent("bucket registry poisoned".into()))?;
        if let Some(bucket) = buckets.get(key) {
            return Ok(Arc::clone(bucket));
        }
        if buckets.len() == MAX_BUCKETS {
            return Err(ApiError::Exhausted(
                "bucket registry capacity reached".into(),
            ));
        }
        buckets
            .try_reserve(1)
            .map_err(|_| ApiError::Exhausted("bucket registry allocation failed".into()))?;
        let limits = match self.policies.get(key) {
            Some(limits) => *limits,
            None => BucketPolicy::from(BucketLimits::defaults(self.config))
                .validate(self.config.global_concurrency)?,
        };
        let bucket = Arc::new(Bucket::new(limits));
        if buckets
            .insert(key.to_owned(), Arc::clone(&bucket))
            .is_some()
        {
            return Err(ApiError::Permanent("duplicate bucket registration".into()));
        }
        increment(&self.counters.active_buckets);
        Ok(bucket)
    }

    fn ensure_running(&self) -> Result<(), ApiError> {
        if self.cancellation.is_cancelled() {
            Err(ApiError::Cancelled)
        } else {
            Ok(())
        }
    }

    #[tracing::instrument(skip_all)]
    async fn acquire<'a>(
        &self,
        semaphore: &'a Semaphore,
        bucket: &Bucket,
    ) -> Result<SemaphorePermit<'a>, ApiError> {
        tokio::select! {
            biased;
            () = self.cancellation.cancelled() => Err(ApiError::Cancelled),
            () = bucket.rejected.cancelled() => Err(bucket.rejection_error()),
            permit = semaphore.acquire() => permit.map_err(|_| ApiError::Cancelled),
        }
    }

    #[tracing::instrument(skip_all)]
    async fn wait_until(&self, deadline: Option<Instant>, bucket: &Bucket) -> Result<(), ApiError> {
        let Some(deadline) = deadline else {
            return self.ensure_running();
        };
        tokio::select! {
            biased;
            () = self.cancellation.cancelled() => Err(ApiError::Cancelled),
            () = bucket.rejected.cancelled() => Err(bucket.rejection_error()),
            () = sleep_until(deadline) => Ok(()),
        }
    }

    #[tracing::instrument(skip_all)]
    async fn admit<'a>(&'a self, bucket: &'a Bucket) -> Result<AttemptPermit<'a>, ApiError> {
        let local = self.acquire(&bucket.in_flight, bucket).await?;
        let sequence = self.acquire(&bucket.dispatch, bucket).await?;
        // While sequence is owned, no new same-bucket attempt can start. At
        // most bucket concurrency - 1 old attempts can change its rate.
        // Each recheck consumes one such change; the bound includes initial
        // admission and one spare round. No rate ticket can accumulate a burst.
        for _ in 0..MAX_ADMISSION_ROUNDS {
            self.wait_until(bucket.due_at()?, bucket).await?;
            let global = self.acquire(&self.global, bucket).await?;
            let now = Instant::now();
            self.ensure_running()?;
            if !bucket.try_start(now)? {
                drop(global);
                continue;
            }
            drop(sequence);
            return Ok(AttemptPermit::new(global, local, &self.counters));
        }
        Err(ApiError::Exhausted(
            "admission recheck ceiling reached".into(),
        ))
    }

    #[tracing::instrument(skip_all)]
    async fn attempt<T, F, Fut>(
        &self,
        bucket: &Bucket,
        attempt: u32,
        operation: &mut F,
    ) -> Result<T, ApiError>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<T, ApiError>>,
    {
        let _permit = self.admit(bucket).await?;
        increment(&self.counters.attempts);
        if attempt > 0 {
            increment(&self.counters.retries);
        }
        let result = tokio::select! {
            biased;
            () = self.cancellation.cancelled() => Err(ApiError::Cancelled),
            () = bucket.rejected.cancelled() => Err(bucket.rejection_error()),
            outcome = timeout(Duration::from_millis(self.config.attempt_timeout_ms), async {
                operation().await
            }) => match outcome {
                Ok(result) => result,
                Err(_) => Err(ApiError::Timeout),
            },
        };
        // Publish rate feedback before releasing local/global reservations.
        // Thus each currently active request can cause at most one rate change
        // while the next dispatch reservation is waiting.
        self.feedback(bucket, &result)?;
        result
    }

    fn feedback<T>(&self, bucket: &Bucket, result: &Result<T, ApiError>) -> Result<(), ApiError> {
        match result.as_ref().err().and_then(retry_kind) {
            Some(RetryKind::Throttled) => {
                increment(&self.counters.throttles);
            }
            Some(RetryKind::Timeout) => {
                increment(&self.counters.timeouts);
            }
            Some(RetryKind::Transient) | None => {}
        }
        let outcome = bucket.feedback(result);
        if matches!(outcome, Err(ApiError::RetryAfterRejected(_))) {
            increment(&self.counters.rejected_retry_hints);
        } else if matches!(result, Err(ApiError::RetryAfter { .. })) && outcome.is_ok() {
            increment(&self.counters.server_delays);
        }
        outcome
    }

    #[tracing::instrument(skip_all)]
    async fn backoff(&self, bucket: &Bucket, nonce: u64, attempt: u32) -> Result<(), ApiError> {
        let cap = BASE_BACKOFF_MS
            .saturating_mul(2_u64.saturating_pow(attempt))
            .min(self.config.max_backoff_ms);
        let random = mix(self.jitter_seed ^ nonce ^ u64::from(attempt).rotate_left(32));
        let milliseconds = random
            .checked_rem(cap.saturating_add(1))
            .ok_or_else(|| ApiError::Permanent("zero internal backoff range".into()))?;
        tokio::select! {
            biased;
            () = self.cancellation.cancelled() => Err(ApiError::Cancelled),
            () = bucket.rejected.cancelled() => Err(bucket.rejection_error()),
            () = sleep(Duration::from_millis(milliseconds)) => Ok(()),
        }
    }
}

fn retryable(error: &ApiError) -> bool {
    retry_kind(error).is_some()
}

fn retry_kind(error: &ApiError) -> Option<RetryKind> {
    match error {
        ApiError::Throttled(_) => Some(RetryKind::Throttled),
        ApiError::Transient(_) => Some(RetryKind::Transient),
        ApiError::Timeout => Some(RetryKind::Timeout),
        ApiError::RetryAfter { kind, .. } => Some(*kind),
        _ => None,
    }
}

fn exhausted(error: ApiError) -> ApiError {
    let detail = match error {
        ApiError::Throttled(message) => format!("throttled: {}", bounded_message(&message)),
        ApiError::Transient(message) => format!("transient: {}", bounded_message(&message)),
        ApiError::Timeout => "attempt timeout".to_owned(),
        ApiError::RetryAfter {
            kind,
            message,
            retry_after,
        } => format!(
            "{kind}: {}; server retry directive: {retry_after:?}",
            bounded_message(&message)
        ),
        other => other.to_string(),
    };
    ApiError::Exhausted(detail)
}

fn bounded_message(message: &str) -> String {
    message.chars().take(MAX_ERROR_CHARS).collect()
}

/// SplitMix64 finalizer. Wrapping arithmetic is part of this non-cryptographic
/// jitter permutation; the seed is randomized once outside the attempt path.
fn mix(value: u64) -> u64 {
    let first = (value ^ value.wrapping_shr(30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    let second = (first ^ first.wrapping_shr(27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    second ^ second.wrapping_shr(31)
}

struct RequestRecord<'a> {
    counters: &'a Counters,
    complete: bool,
}

impl<'a> RequestRecord<'a> {
    fn new(counters: &'a Counters) -> Self {
        Self {
            counters,
            complete: false,
        }
    }

    fn complete<T>(&mut self, result: &Result<T, ApiError>) {
        match result {
            Ok(_) => increment(&self.counters.successes),
            Err(ApiError::Cancelled) => increment(&self.counters.cancelled),
            Err(_) => increment(&self.counters.failures),
        };
        self.complete = true;
    }
}

impl Drop for RequestRecord<'_> {
    fn drop(&mut self) {
        if !self.complete {
            increment(&self.counters.cancelled);
        }
    }
}

struct AttemptPermit<'a> {
    _global: SemaphorePermit<'a>,
    _local: SemaphorePermit<'a>,
    counters: &'a Counters,
}

impl<'a> AttemptPermit<'a> {
    fn new(
        global: SemaphorePermit<'a>,
        local: SemaphorePermit<'a>,
        counters: &'a Counters,
    ) -> Self {
        let current = increment(&counters.in_flight);
        counters
            .peak_in_flight
            .fetch_max(current, Ordering::Relaxed);
        Self {
            _global: global,
            _local: local,
            counters,
        }
    }
}

impl Drop for AttemptPermit<'_> {
    fn drop(&mut self) {
        self.counters
            .in_flight
            .update(Ordering::Relaxed, Ordering::Relaxed, |value| {
                value.saturating_sub(1)
            });
    }
}

#[cfg(test)]
mod tests {
    use super::{AdaptiveRate, ApiError, MAX_REQUESTS_PER_SECOND};

    #[test]
    fn fractional_policy_recovery_keeps_sub_nanosecond_rate_credits() -> Result<(), ApiError> {
        let mut rate = AdaptiveRate::PerPeriod {
            interval_ns: 10_000,
            minimum_ns: 10_000,
            recovery_rps: 0,
        };
        rate.throttle()?;
        assert_eq!(rate.interval_ns()?, 20_000);
        for _ in 0..3 {
            rate.recover();
        }
        assert_eq!(rate.interval_ns()?, 19_999);
        for _ in 0..MAX_REQUESTS_PER_SECOND {
            rate.recover();
            assert!(rate.interval_ns()? >= 10_000);
        }
        assert_eq!(rate.interval_ns()?, 10_000);
        rate.throttle()?;
        assert_eq!(rate.interval_ns()?, 20_000);
        Ok(())
    }
}
