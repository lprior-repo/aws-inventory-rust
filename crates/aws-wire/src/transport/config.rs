use std::{ops::RangeInclusive, path::PathBuf, time::Duration};

use super::TransportError;

#[derive(Clone, Debug)]
pub struct Config {
    pub request_deadline: Duration,

    pub connect_timeout: Duration,

    pub response_body_limit: usize,

    pub max_connections: usize,

    pub idle_timeout: Duration,

    pub pool_idle_per_host: usize,

    pub extra_ca_bundle: Option<PathBuf>,

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
    /// Idle sockets retain their connection permits until evicted. Bound their
    /// lifetime by both the connector timeout and half the request deadline.
    #[must_use]
    pub fn idle_timeout_range(&self) -> RangeInclusive<Duration> {
        let upper = self
            .request_deadline
            .checked_div(2)
            .map_or(Duration::ZERO, |half| self.connect_timeout.min(half));
        Duration::from_millis(1)..=upper
    }

    pub(super) fn validate(&self) -> Result<(), TransportError> {
        if !(Duration::from_millis(2)..=Duration::from_secs(300)).contains(&self.request_deadline)
            || self.connect_timeout.is_zero()
            || self.connect_timeout > self.request_deadline
        {
            return Err(TransportError::InvalidRequest(
                "request deadline or connector timeout is out of range",
            ));
        }
        if !self.idle_timeout_range().contains(&self.idle_timeout) {
            return Err(TransportError::InvalidRequest(
                "pool idle timeout is out of range",
            ));
        }
        if !(1..=64 * 1024 * 1024).contains(&self.response_body_limit)
            || !(1..=4_096).contains(&self.max_connections)
            || !(1..=self.max_connections).contains(&self.pool_idle_per_host)
        {
            return Err(TransportError::InvalidRequest(
                "invalid body or connection limit",
            ));
        }
        Ok(())
    }
}
