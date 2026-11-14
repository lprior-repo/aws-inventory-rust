//! Seeded, bounded HTTP/1.1 fault proxy for development fixtures, not product traffic.
//!
//! Signed request bytes are preserved. Decisions depend only on seed, parsed scope,
//! ordinal and rules; Tokio time affects only latency, refusal and token refill.
//! Each client owns one upstream socket. Shutdown cancels socket work, drains its
//! owned task region and reports outcomes. Dropping the handle aborts that region.
//! HTTP buffers, scope state, rules, task admission and retained logs have hard caps.
//! Only CRLF-delimited heads are accepted. Expect headers are unsupported and
//! receive 417 with `x-proxy-error: expectation-failed`, without reading a body.
//! Matching latency applies to all responses after a scope is parsed; malformed
//! heads rejected before a scope is known are exempt. Snapshot entries are sorted
//! by scope and ordinal; eviction remains arrival-ordered, so replay determinism
//! of retained entries is guaranteed only while `Snapshot::dropped == 0`.
//! Signed tuples sort lexically by account, region and service; Unsigned sorts last.
//! Unsigned retains its legacy replay stream. Signed mixes its component bytes
//! from `seed ^ 0x7369_676e_6564_0001` rather than `seed`; previously
//! recorded signed replay vectors therefore change.

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

/// Maximum bytes in an HTTP head (including the terminating empty line).
pub const MAX_HEAD_BYTES: usize = 32_768;
/// Maximum headers in a parsed head or trailer block.
pub const MAX_HEADERS: usize = 128;
/// Maximum request or decoded response body bytes.
pub const MAX_BODY_BYTES: usize = 8_388_608;
/// Maximum individual upstream chunk payload.
pub const MAX_CHUNK_BYTES: usize = 1_048_576;
/// Maximum chunks per response; wire overhead is separately bounded.
pub const MAX_CHUNKS: usize = 65_536;
/// Maximum simultaneously admitted client tasks; excess clients stay in backlog.
pub const MAX_CONNECTIONS: usize = 64;
/// Maximum scope states; reaching this bound never evicts another scope.
pub const MAX_SCOPES: usize = 131_072;
/// Default and maximum retained decision entries.
pub const MAX_LOG_ENTRIES: usize = 65_536;
/// Maximum entries in each configuration table.
pub const MAX_RULES: usize = 1_024;
/// Maximum ASCII bytes in each account, access key, region or service name.
pub const MAX_SCOPE_NAME: usize = 128;
/// Maximum host length in the upstream authority.
const MAX_HOST_BYTES: usize = 253;
/// Maximum CA bundle size accepted during startup.
const MAX_CA_BYTES: u64 = 1_048_576;
/// Maximum certificates in a CA bundle.
const MAX_CERTIFICATES: usize = 64;
/// Per-connection request budget (also bounds keep-alive lifetime).
pub const MAX_REQUESTS: usize = 65_536;
/// Owner event budget, including accept, control, timer and join events.
const MAX_OWNER_EVENTS: u64 = 1_000_000_000;
/// Maximum waiting control commands.
const CONTROL_CAPACITY: usize = 64;
/// Idle read, write, connect and TLS-handshake deadline.
pub const IDLE_TIMEOUT: Duration = Duration::from_secs(30);
/// Graceful drain deadline before aborting remaining tasks.
pub const DRAIN_TIMEOUT: Duration = Duration::from_secs(2);
/// Largest configured latency or plan-driven refusal window.
pub const MAX_DELAY: Duration = Duration::from_secs(3_600);
/// Largest per-scope rate in requests per second.
pub const MAX_RATE: u32 = 1_000_000;

/// Typed configuration, framing, resource, supervision and transport failures.
#[derive(Debug, thiserror::Error)]
pub enum ProxyError {
    /// Configuration failed its documented bound or syntax.
    #[error("invalid proxy configuration: {0}")]
    Configuration(&'static str),
    /// Peer sent invalid or ambiguous HTTP framing.
    #[error("invalid HTTP framing")]
    Framing,
    /// Expect headers are unsupported; the request body is not consumed.
    #[error("unsupported HTTP expectation")]
    UnsupportedExpectation,
    /// A bounded input or collection was exhausted.
    #[error("proxy resource limit: {0}")]
    Limit(&'static str),
    /// Fallible buffer reservation failed.
    #[error("proxy allocation failed")]
    Allocation,
    /// Socket or file I/O failed, including a failed refusal rebind.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// A bounded I/O operation timed out.
    #[error("proxy I/O deadline exceeded")]
    Timeout,
    /// Control task is no longer running.
    #[error("proxy owner stopped")]
    Stopped,
    /// The owner task panicked; never confused with cancellation.
    #[error("proxy owner panicked")]
    OwnerPanicked,
    /// State mutex was poisoned.
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
