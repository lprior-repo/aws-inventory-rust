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
    /// The provider does not expose this discoverable catalog type. This is a
    /// capability boundary, not a transient or malformed response.
    #[error("provider does not support this type: {0}")]
    UnsupportedType(String),
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

impl ApiExecutor {}

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
mod policy;
pub use policy::*;
mod stats;
pub use stats::*;
mod pacing;
use pacing::*;
mod bucket;
use bucket::*;
mod admission;
mod construction;
mod execution;
mod registry;
mod retry;
use retry::*;
