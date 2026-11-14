use super::{
    Fault, MAX_DELAY, MAX_LOG_ENTRIES, MAX_RATE, MAX_RULES, MAX_SCOPES, Probability, ProxyError,
    Scope, ScopeName,
};
use std::path::PathBuf;
use std::time::Duration;

/// AWS protocol-specific canned error envelope.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum BodyStyle {
    /// AWS JSON error object.
    #[default]
    Json,
    /// AWS Query ErrorResponse envelope.
    QueryXml,
    /// REST XML Error envelope (including S3 codes).
    RestXml,
    /// EC2 Response/Errors envelope.
    Ec2Xml,
    /// Definite CBOR map with __type and message fields.
    Cbor,
}

/// Optional signed-component matches; an empty filter also matches Unsigned.
#[derive(Clone, Debug, Default)]
pub struct ScopeFilter {
    /// Account or unmapped access key.
    pub account: Option<ScopeName>,
    /// Signing region.
    pub region: Option<ScopeName>,
    /// Signing service.
    pub service: Option<ScopeName>,
}
impl ScopeFilter {
    pub(crate) fn matches(&self, scope: &Scope) -> bool {
        match scope {
            Scope::Signed {
                account,
                region,
                service,
            } => {
                self.account.as_ref().is_none_or(|v| v == account)
                    && self.region.as_ref().is_none_or(|v| v == region)
                    && self.service.as_ref().is_none_or(|v| v == service)
            }
            Scope::Unsigned => {
                self.account.is_none() && self.region.is_none() && self.service.is_none()
            }
        }
    }
}

/// First successful matching rule wins after scheduled entries.
#[derive(Clone, Debug)]
pub struct Rule {
    /// Scope selection.
    pub filter: ScopeFilter,
    /// Probability in parts per million.
    pub probability: Probability,
    /// Fault to inject.
    pub fault: Fault,
}

/// Exact per-scope ordinal override, starting at zero.
#[derive(Clone, Debug)]
pub struct Schedule {
    /// Full scope tuple to match.
    pub scope: Scope,
    /// Zero-based request number in that scope.
    pub ordinal: u64,
    /// Fault to inject.
    pub fault: Fault,
}

/// Integer token bucket with one second of burst capacity.
#[derive(Clone, Debug)]
pub struct RateLimit {
    /// Matching scope (first matching limit wins).
    pub filter: ScopeFilter,
    /// 1..=MAX_RATE tokens per second and burst size.
    pub requests_per_second: u32,
}

/// Raw public builder options, consumed and frozen by `Proxy::start`.
/// Every table/capacity/timing bound is established before a listener is bound.
#[derive(Clone, Debug)]
pub struct Config {
    /// http://host:port or https://host:port, without a path or userinfo.
    pub upstream: String,
    /// Required for HTTPS, rejected for HTTP; only these authorities are trusted.
    pub ca_bundle: Option<PathBuf>,
    /// Deterministic SplitMix64 seed.
    pub seed: u64,
    /// Ordered probabilistic fault rules.
    pub rules: Vec<Rule>,
    /// Exact overrides; duplicate scope/ordinal pairs are rejected.
    pub schedule: Vec<Schedule>,
    /// Access-key to account mappings; duplicate keys are rejected.
    pub accounts: Vec<(ScopeName, ScopeName)>,
    /// Per-service envelope styles; duplicate services are rejected.
    pub styles: Vec<(ScopeName, BodyStyle)>,
    /// Fallback envelope style.
    pub default_style: BodyStyle,
    /// First matching fixed latency, bounded by MAX_DELAY.
    pub latency: Vec<(ScopeFilter, Duration)>,
    /// First matching per-scope token bucket.
    pub rate_limits: Vec<RateLimit>,
    /// Plan-driven connection refusal duration, 1ns..=MAX_DELAY.
    pub refusal_duration: Duration,
    /// Retained entries; 1..=MAX_LOG_ENTRIES, oldest discarded on overflow.
    pub log_capacity: usize,
    /// Maximum registered scopes, 1..=MAX_SCOPES, no eviction.
    pub scope_capacity: usize,
}
impl Config {
    /// Construct a fault-free plain HTTP configuration. HTTPS callers must set
    /// `ca_bundle` before `Proxy::start`; every bound is checked there.
    pub fn new(upstream: impl Into<String>, seed: u64) -> Self {
        Self {
            upstream: upstream.into(),
            ca_bundle: None,
            seed,
            rules: Vec::new(),
            schedule: Vec::new(),
            accounts: Vec::new(),
            styles: Vec::new(),
            default_style: BodyStyle::Json,
            latency: Vec::new(),
            rate_limits: Vec::new(),
            refusal_duration: Duration::from_secs(1),
            log_capacity: MAX_LOG_ENTRIES,
            scope_capacity: MAX_SCOPES,
        }
    }
    pub(crate) fn validate(&self) -> Result<(), ProxyError> {
        self.validate_capacities()?;
        self.validate_timing()?;
        self.validate_unique()
    }
    fn validate_capacities(&self) -> Result<(), ProxyError> {
        let lengths = [
            self.rules.len(),
            self.schedule.len(),
            self.accounts.len(),
            self.styles.len(),
            self.latency.len(),
            self.rate_limits.len(),
        ];
        if lengths.into_iter().any(|n| n > MAX_RULES)
            || self.log_capacity == 0
            || self.log_capacity > MAX_LOG_ENTRIES
            || self.scope_capacity == 0
            || self.scope_capacity > MAX_SCOPES
        {
            return Err(ProxyError::Configuration("configuration table capacity"));
        }
        Ok(())
    }
    fn validate_timing(&self) -> Result<(), ProxyError> {
        if self.refusal_duration.is_zero()
            || self.refusal_duration > MAX_DELAY
            || self.latency.iter().any(|(_, d)| *d > MAX_DELAY)
            || self
                .rate_limits
                .iter()
                .any(|r| r.requests_per_second == 0 || r.requests_per_second > MAX_RATE)
        {
            return Err(ProxyError::Configuration("duration or rate"));
        }
        Ok(())
    }
    fn validate_unique(&self) -> Result<(), ProxyError> {
        let schedules = self.schedule.iter().enumerate().any(|(i, a)| {
            self.schedule
                .iter()
                .take(i)
                .any(|b| a.scope == b.scope && a.ordinal == b.ordinal)
        });
        let accounts = self
            .accounts
            .iter()
            .enumerate()
            .any(|(i, a)| self.accounts.iter().take(i).any(|b| a.0 == b.0));
        let styles = self
            .styles
            .iter()
            .enumerate()
            .any(|(i, a)| self.styles.iter().take(i).any(|b| a.0 == b.0));
        if schedules || accounts || styles {
            return Err(ProxyError::Configuration("duplicate configuration key"));
        }
        Ok(())
    }
}
