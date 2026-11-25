mod process;
#[cfg(test)]
mod tests;

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SweepMetrics {
    pub bootstrap_ms: u64,
    pub seed_ms: u64,
    pub drain_ms: u64,
    pub api_attempts: u64,
    pub api_retries: u64,
    pub api_throttles: u64,
    pub connections_opened: u64,
    pub requests_sent: u64,
    pub requests_reused: u64,
    pub http2_requests: u64,
    pub bytes_read: u64,
    pub handshake_ns: u64,
    pub hosts: u64,
    pub pages: u64,
    pub decode_ns: u64,
    pub store_commit_ns: u64,
    pub store_commits: u64,
    pub claims: u64,
    pub cpu_user_ms: u64,
    pub cpu_system_ms: u64,
    pub peak_rss_kb: u64,
    pub process_metrics_available: bool,
    pub progress_dropped: u64,
    pub child_requests_skipped: u64,
}

#[derive(Default)]
pub(crate) struct Accumulators {
    decode_ns: AtomicU64,
    store_commit_ns: AtomicU64,
    store_commits: AtomicU64,
    claims: AtomicU64,
    child_requests_skipped: AtomicU64,
    pub(crate) attempts: Option<std::sync::Arc<crate::AttemptedOperations>>,
}

impl Accumulators {
    pub(crate) fn decoded(&self, started: Instant) {
        add(&self.decode_ns, bounded(started.elapsed().as_nanos()));
    }

    pub(crate) fn committed(&self, started: Instant, succeeded: bool) {
        add(&self.store_commit_ns, bounded(started.elapsed().as_nanos()));
        if succeeded {
            add(&self.store_commits, 1);
        }
    }

    pub(crate) fn claimed(&self, leases: usize) {
        add(&self.claims, u64::try_from(leases).unwrap_or(u64::MAX));
    }

    pub(crate) fn skipped_children(&self, count: u64) {
        add(&self.child_requests_skipped, count);
    }

    fn snapshot(&self, metrics: &mut SweepMetrics) {
        metrics.decode_ns = self.decode_ns.load(Ordering::Relaxed);
        metrics.store_commit_ns = self.store_commit_ns.load(Ordering::Relaxed);
        metrics.store_commits = self.store_commits.load(Ordering::Relaxed);
        metrics.pages = metrics.store_commits;
        metrics.claims = self.claims.load(Ordering::Relaxed);
        metrics.child_requests_skipped = self.child_requests_skipped.load(Ordering::Relaxed);
    }
}

pub(crate) fn complete(
    metrics: &mut SweepMetrics,
    counters: &Accumulators,
    executor: inventory_engine::limiter::ExecutorStats,
    transport: aws_wire::transport::Statistics,
) {
    counters.snapshot(metrics);
    metrics.api_attempts = executor.attempts;
    metrics.api_retries = executor.retries;
    metrics.api_throttles = executor.throttles;
    metrics.connections_opened = count(transport.connections_opened);
    metrics.requests_sent = count(transport.requests_sent);
    metrics.requests_reused = count(transport.requests_reused);
    metrics.http2_requests = count(transport.http2_requests);
    metrics.bytes_read = count(transport.bytes_read);
    metrics.handshake_ns = transport.handshake_ns;
    metrics.hosts = count(transport.hosts);
    if let Some((user, system, rss)) = process::snapshot() {
        metrics.cpu_user_ms = user;
        metrics.cpu_system_ms = system;
        metrics.peak_rss_kb = rss;
        metrics.process_metrics_available = true;
    }
}

pub(crate) fn elapsed_ms(started: Instant) -> u64 {
    bounded(started.elapsed().as_millis())
}

fn bounded(value: u128) -> u64 {
    let Ok(value) = u64::try_from(value) else {
        return u64::MAX;
    };
    value
}

fn count(value: usize) -> u64 {
    let Ok(value) = u64::try_from(value) else {
        return u64::MAX;
    };
    value
}

fn add(counter: &AtomicU64, amount: u64) {
    counter.update(Ordering::Relaxed, Ordering::Relaxed, |value| {
        value.saturating_add(amount)
    });
}
