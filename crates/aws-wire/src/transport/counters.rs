use std::{
    collections::BTreeMap,
    sync::{
        Mutex, MutexGuard, PoisonError,
        atomic::{AtomicU64, AtomicUsize, Ordering},
    },
    time::Duration,
};

pub(super) const MAX_HOSTS: usize = 16_384;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Statistics {
    pub connections_opened: usize,

    pub connections_open: usize,

    pub requests_sent: usize,

    pub requests_reused: usize,

    pub timeouts: usize,

    pub bytes_read: usize,

    pub http2_requests: usize,

    pub handshake_ns: u64,

    pub hosts: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostConnections {
    pub authority: Box<str>,

    pub connections: usize,
}

#[derive(Default)]
pub(super) struct Counters {
    opened: AtomicUsize,
    open: AtomicUsize,
    requests: AtomicUsize,
    read: AtomicUsize,
    timeouts: AtomicUsize,
    http2: AtomicUsize,
    handshake_ns: AtomicU64,
    hosts: Mutex<BTreeMap<Box<str>, usize>>,
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

    pub(super) fn http2(&self) {
        self.http2
            .update(Ordering::Relaxed, Ordering::Relaxed, |value| {
                value.saturating_add(1)
            });
    }

    pub(super) fn read(&self, bytes: usize) {
        self.read.fetch_add(bytes, Ordering::Relaxed);
    }

    pub(super) fn attempt(&self, authority: &str) {
        let mut hosts = self.hosts();
        if hosts.len() < MAX_HOSTS && !hosts.contains_key(authority) {
            hosts.insert(authority.into(), 0);
        }
    }

    pub(super) fn handshake(&self, elapsed: Duration) {
        let nanos = u64::try_from(elapsed.as_nanos()).unwrap_or(u64::MAX);
        self.handshake_ns
            .update(Ordering::Relaxed, Ordering::Relaxed, |value| {
                value.saturating_add(nanos)
            });
    }

    pub(super) fn connected(&self, authority: &str) {
        if let Some(connections) = self.hosts().get_mut(authority) {
            *connections = connections.saturating_add(1);
        }
    }

    pub(super) fn host_connections(&self) -> Vec<HostConnections> {
        self.hosts()
            .iter()
            .map(|(authority, connections)| HostConnections {
                authority: authority.clone(),
                connections: *connections,
            })
            .collect()
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
            http2_requests: self.http2.load(Ordering::Relaxed),
            handshake_ns: self.handshake_ns.load(Ordering::Relaxed),
            hosts: self.hosts().len(),
        }
    }

    fn hosts(&self) -> MutexGuard<'_, BTreeMap<Box<str>, usize>> {
        self.hosts.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

#[cfg(test)]
mod tests {
    use super::{Counters, HostConnections, MAX_HOSTS};

    #[test]
    fn given_more_authorities_than_the_bound_when_recorded_then_later_ones_are_dropped() {
        let counters = Counters::default();
        (0..MAX_HOSTS).for_each(|index| counters.attempt(&format!("host-{index}")));
        counters.attempt("one-too-many");
        counters.connected("one-too-many");
        let hosts = counters.host_connections();
        assert_eq!(counters.snapshot().hosts, MAX_HOSTS);
        assert_eq!(hosts.len(), MAX_HOSTS);
        assert!(
            hosts
                .iter()
                .all(|host| host.authority.as_ref() != "one-too-many")
        );
    }

    #[test]
    fn given_repeated_attempts_when_some_connect_then_only_connections_are_counted() {
        let counters = Counters::default();
        ["b:1", "b:1", "b:1", "a:1"]
            .iter()
            .for_each(|authority| counters.attempt(authority));
        counters.connected("b:1");
        counters.connected("b:1");
        assert_eq!(
            counters.host_connections(),
            vec![
                HostConnections {
                    authority: "a:1".into(),
                    connections: 0
                },
                HostConnections {
                    authority: "b:1".into(),
                    connections: 2
                },
            ]
        );
        assert_eq!(counters.snapshot().hosts, 2);
    }
}
