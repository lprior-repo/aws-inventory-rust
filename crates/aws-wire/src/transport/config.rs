use std::{path::PathBuf, time::Duration};

use super::TransportError;

/// Construction-time limits shared by every clone of a client.
#[derive(Clone, Debug)]
pub struct Config {
    /// Whole-request deadline, 2 ms through 300 s; default 30 s.
    pub request_deadline: Duration,
    /// Bound for the whole connector, including permit wait, DNS, TCP and TLS; default 10 s.
    /// Must be positive and no greater than `request_deadline`, including background connects.
    pub connect_timeout: Duration,
    /// Maximum buffered body, 1 byte through 64 MiB; default 8 MiB.
    pub response_body_limit: usize,
    /// Global socket cap, 1 through 512; provisional default 128.
    /// Choose near the executor's in-flight limit; reserve other process FDs separately.
    pub max_connections: usize,
    /// Idle timeout, at least 1 ms and at most half the request deadline; provisional default 1 s.
    /// Idle sockets retain permits for up to this timeout plus `max(idle_timeout, 90 ms)`.
    /// A sweep of M endpoints with cap N needs about ceil(M/N) capacity waves; new hosts
    /// can time out before eviction. Choose a short valid expiry and cap near the in-flight
    /// limit; defaults await sweep benchmarks. Active HTTP/2 streams can outlive pool expiry.
    pub idle_timeout: Duration,
    /// Maximum idle sockets per endpoint, 1 through the global cap; default 2.
    pub pool_idle_per_host: usize,
    /// Optional PEM roots appended to system roots; bundle size is capped at 1 MiB.
    pub extra_ca_bundle: Option<PathBuf>,
    /// Permit unencrypted HTTP for local emulators/proxies; default false.
    pub allow_plain_http: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            request_deadline: Duration::from_secs(30),
            connect_timeout: Duration::from_secs(10),
            response_body_limit: 8 * 1024 * 1024,
            max_connections: 128,
            idle_timeout: Duration::from_secs(1),
            pool_idle_per_host: 2,
            extra_ca_bundle: None,
            allow_plain_http: false,
        }
    }
}

impl Config {
    pub(super) fn validate(&self) -> Result<(), TransportError> {
        if !(Duration::from_millis(2)..=Duration::from_secs(300)).contains(&self.request_deadline)
            || self.connect_timeout.is_zero()
            || self.connect_timeout > self.request_deadline
            || self.idle_timeout < Duration::from_millis(1)
            || self.idle_timeout > self.request_deadline.saturating_sub(self.idle_timeout)
        {
            return Err(TransportError::InvalidRequest(
                "invalid request, connector or pool expiry bound",
            ));
        }
        if !(1..=64 * 1024 * 1024).contains(&self.response_body_limit)
            || !(1..=512).contains(&self.max_connections)
            || !(1..=self.max_connections).contains(&self.pool_idle_per_host)
        {
            return Err(TransportError::InvalidRequest(
                "invalid body or connection limit",
            ));
        }
        Ok(())
    }
}
