use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Statistics {
    pub connections_opened: usize,

    pub connections_open: usize,

    pub requests_sent: usize,

    pub requests_reused: usize,

    pub timeouts: usize,

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
