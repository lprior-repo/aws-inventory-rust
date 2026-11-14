mod coding;
mod config;
mod errors;
mod http;
mod lifecycle;
mod message;
mod owner;
mod plan;
mod protocol;
mod reply;
mod response;
mod session;
mod state;
mod transport;
mod validated;

pub use config::{BodyStyle, Config, RateLimit, Rule, Schedule, ScopeFilter};
pub use lifecycle::{DrainReport, Proxy};
pub use plan::{Decision, Fault, Probability, Scope, ScopeName};
pub use state::{Counters, DecisionEntry, Snapshot};

use std::time::Duration;

pub const MAX_HEAD_BYTES: usize = 32_768;

pub const MAX_HEADERS: usize = 128;

pub const MAX_BODY_BYTES: usize = 8_388_608;

pub const MAX_CHUNK_BYTES: usize = 1_048_576;

pub const MAX_CHUNKS: usize = 65_536;

pub const MAX_CONNECTIONS: usize = 64;

pub const MAX_SCOPES: usize = 131_072;

pub const MAX_LOG_ENTRIES: usize = 65_536;

pub const MAX_RULES: usize = 1_024;

pub const MAX_SCOPE_NAME: usize = 128;

const MAX_HOST_BYTES: usize = 253;

const MAX_CA_BYTES: u64 = 1_048_576;

const MAX_CERTIFICATES: usize = 64;

pub const MAX_REQUESTS: usize = 65_536;

const MAX_OWNER_EVENTS: u64 = 1_000_000_000;

const CONTROL_CAPACITY: usize = 64;

pub const IDLE_TIMEOUT: Duration = Duration::from_secs(30);

pub const DRAIN_TIMEOUT: Duration = Duration::from_secs(2);

pub const MAX_DELAY: Duration = Duration::from_secs(3_600);

pub const MAX_RATE: u32 = 1_000_000;

#[derive(Debug, thiserror::Error)]
pub enum ProxyError {
    #[error("invalid proxy configuration: {0}")]
    Configuration(&'static str),

    #[error("invalid HTTP framing")]
    Framing,

    #[error("unsupported HTTP expectation")]
    UnsupportedExpectation,

    #[error("proxy resource limit: {0}")]
    Limit(&'static str),

    #[error("proxy allocation failed")]
    Allocation,

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("proxy I/O deadline exceeded")]
    Timeout,

    #[error("proxy owner stopped")]
    Stopped,

    #[error("proxy owner panicked")]
    OwnerPanicked,

    #[error("proxy state poisoned")]
    Poisoned,
}

pub(crate) fn reserve(bytes: &mut Vec<u8>, additional: usize) -> Result<(), ProxyError> {
    bytes
        .try_reserve(additional)
        .map_err(|_| ProxyError::Allocation)
}

pub(crate) async fn deadline<T>(
    future: impl Future<Output = Result<T, std::io::Error>>,
) -> Result<T, ProxyError> {
    tokio::time::timeout(IDLE_TIMEOUT, future)
        .await
        .map_err(|_| ProxyError::Timeout)?
        .map_err(ProxyError::Io)
}
