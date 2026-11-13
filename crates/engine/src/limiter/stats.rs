//! Bounded executor counters and immutable statistics.
use super::*;

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
pub(super) struct Counters {
    pub(super) attempts: AtomicU64,
    pub(super) retries: AtomicU64,
    pub(super) throttles: AtomicU64,
    pub(super) server_delays: AtomicU64,
    pub(super) rejected_retry_hints: AtomicU64,
    pub(super) timeouts: AtomicU64,
    pub(super) successes: AtomicU64,
    pub(super) failures: AtomicU64,
    pub(super) cancelled: AtomicU64,
    pub(super) in_flight: AtomicU64,
    pub(super) peak_in_flight: AtomicU64,
    pub(super) active_buckets: AtomicU64,
}

impl Counters {
    pub(super) fn snapshot(&self) -> ExecutorStats {
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

pub(super) fn increment(counter: &AtomicU64) -> u64 {
    counter
        .update(Ordering::Relaxed, Ordering::Relaxed, |value| {
            value.saturating_add(1)
        })
        .saturating_add(1)
}

pub(super) fn ceil_div(numerator: u64, denominator: u64) -> Result<u64, ApiError> {
    numerator
        .checked_add(denominator.saturating_sub(1))
        .and_then(|value| value.checked_div(denominator))
        .ok_or_else(|| ApiError::Permanent("invalid internal rate arithmetic".into()))
}
