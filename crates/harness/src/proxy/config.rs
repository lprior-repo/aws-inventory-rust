use super::{
    Fault, MAX_DELAY, MAX_LOG_ENTRIES, MAX_RATE, MAX_RULES, MAX_SCOPES, Probability, ProxyError,
    Scope, ScopeName,
};
use std::path::PathBuf;
use std::time::Duration;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum BodyStyle {
    #[default]
    Json,

    QueryXml,

    RestXml,

    Ec2Xml,

    Cbor,
}

#[derive(Clone, Debug, Default)]
pub struct ScopeFilter {
    pub account: Option<ScopeName>,

    pub region: Option<ScopeName>,

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

#[derive(Clone, Debug)]
pub struct Rule {
    pub filter: ScopeFilter,

    pub probability: Probability,

    pub fault: Fault,
}

#[derive(Clone, Debug)]
pub struct Schedule {
    pub scope: Scope,

    pub ordinal: u64,

    pub fault: Fault,
}

#[derive(Clone, Debug)]
pub struct RateLimit {
    pub filter: ScopeFilter,

    pub requests_per_second: u32,
}

#[derive(Clone, Debug)]
pub struct Config {
    pub upstream: String,

    pub ca_bundle: Option<PathBuf>,

    pub seed: u64,

    pub rules: Vec<Rule>,

    pub schedule: Vec<Schedule>,

    pub accounts: Vec<(ScopeName, ScopeName)>,

    pub styles: Vec<(ScopeName, BodyStyle)>,

    pub default_style: BodyStyle,

    pub latency: Vec<(ScopeFilter, Duration)>,

    pub rate_limits: Vec<RateLimit>,

    pub refusal_duration: Duration,

    pub log_capacity: usize,

    pub scope_capacity: usize,
}
impl Config {
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
