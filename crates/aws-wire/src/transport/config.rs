use std::{path::PathBuf, time::Duration};

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
