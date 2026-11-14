use std::sync::atomic::{AtomicUsize, Ordering};

/// Approximate concurrent snapshot. Lifetime totals wrap at `usize::MAX`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Statistics {
    /// TCP connections successfully established (TLS may subsequently fail).
    pub connections_opened: usize,
    /// Established sockets currently owned by the connector/pool.
    pub connections_open: usize,
    /// Validated send attempts, including failed/cancelled attempts.
    pub requests_sent: usize,
    /// Estimate: attempts minus opened connections, saturating at zero.
    /// Hyper does not expose a per-request reuse flag; failures affect this estimate.
    pub requests_reused: usize,
    /// Public `send` outcomes returning `Timeout`, counted once for either deadline bound.
    /// Caller cancellation and detached connector expiry after a successful send are not counted.
    pub timeouts: usize,
    /// Socket bytes read, including HTTP headers and TLS records.
    pub bytes_read: usize,
}

#[derive(Default)]
pub(super) struct Counters {
    opened: AtomicUsize,
    open: AtomicUsize,
    requests: AtomicUsize,
    read: AtomicUsize,
    timeouts: AtomicUsize,
}

impl Counters {
    pub(super) fn opened(&self) {
        self.opened.fetch_add(1, Ordering::Relaxed);
        self.open.fetch_add(1, Ordering::Relaxed);
    }

    pub(super) fn closed(&self) {
        self.open.fetch_sub(1, Ordering::Relaxed);
    }

    pub(super) fn sent(&self) {
        self.requests.fetch_add(1, Ordering::Relaxed);
    }

    pub(super) fn timed_out(&self) {
        self.timeouts.fetch_add(1, Ordering::Relaxed);
    }

    pub(super) fn read(&self, bytes: usize) {
        self.read.fetch_add(bytes, Ordering::Relaxed);
    }

    pub(super) fn snapshot(&self) -> Statistics {
        let requests_sent = self.requests.load(Ordering::Relaxed);
        let connections_opened = self.opened.load(Ordering::Relaxed);
        Statistics {
            connections_opened,
            connections_open: self.open.load(Ordering::Relaxed),
            requests_sent,
            requests_reused: requests_sent.saturating_sub(connections_opened),
            timeouts: self.timeouts.load(Ordering::Relaxed),
            bytes_read: self.read.load(Ordering::Relaxed),
        }
    }
}
