use super::{
    BodyStyle, Config, ProxyError, RateLimit, Rule, Schedule, Scope, ScopeFilter, ScopeName,
    transport::Target,
};
use std::{num::NonZeroU32, time::Duration};

#[derive(Clone, Copy)]
pub(crate) enum RatePolicy {
    Unlimited,
    Limited(NonZeroU32),
}
struct ValidatedRate {
    filter: ScopeFilter,
    per_second: NonZeroU32,
}

pub(crate) struct ValidatedConfig {
    seed: u64,
    rules: Vec<Rule>,
    schedule: Vec<Schedule>,
    accounts: Vec<(ScopeName, ScopeName)>,
    styles: Vec<(ScopeName, BodyStyle)>,
    default_style: BodyStyle,
    latency: Vec<(ScopeFilter, Duration)>,
    rates: Vec<ValidatedRate>,
    refusal_duration: Duration,
    log_capacity: usize,
    scope_capacity: usize,
}
impl ValidatedConfig {
    pub(crate) async fn parse(config: Config) -> Result<(Self, Target), ProxyError> {
        config.validate()?;
        let target = Target::new(&config).await?;
        Ok((Self::freeze(config)?, target))
    }
    fn freeze(config: Config) -> Result<Self, ProxyError> {
        let rates = rates(config.rate_limits)?;
        Ok(Self {
            seed: config.seed,
            rules: config.rules,
            schedule: config.schedule,
            accounts: config.accounts,
            styles: config.styles,
            default_style: config.default_style,
            latency: config.latency,
            rates,
            refusal_duration: config.refusal_duration,
            log_capacity: config.log_capacity,
            scope_capacity: config.scope_capacity,
        })
    }
    pub(crate) fn seed(&self) -> u64 {
        self.seed
    }
    pub(crate) fn rules(&self) -> &[Rule] {
        &self.rules
    }
    pub(crate) fn schedule(&self) -> &[Schedule] {
        &self.schedule
    }
    pub(crate) fn accounts(&self) -> &[(ScopeName, ScopeName)] {
        &self.accounts
    }
    pub(crate) fn log_capacity(&self) -> usize {
        self.log_capacity
    }
    pub(crate) fn scope_capacity(&self) -> usize {
        self.scope_capacity
    }
    pub(crate) fn refusal_duration(&self) -> Duration {
        self.refusal_duration
    }
    pub(crate) fn delay(&self, scope: &Scope) -> Duration {
        self.latency
            .iter()
            .find(|(filter, _)| filter.matches(scope))
            .map_or(Duration::ZERO, |(_, delay)| *delay)
    }
    pub(crate) fn rate(&self, scope: &Scope) -> RatePolicy {
        self.rates
            .iter()
            .find(|limit| limit.filter.matches(scope))
            .map_or(RatePolicy::Unlimited, |limit| {
                RatePolicy::Limited(limit.per_second)
            })
    }
    pub(crate) fn style(&self, scope: &Scope) -> BodyStyle {
        match scope {
            Scope::Unsigned => self.default_style,
            Scope::Signed { service, .. } => self
                .styles
                .iter()
                .find(|(candidate, _)| candidate == service)
                .map_or(self.default_style, |(_, style)| *style),
        }
    }
}

fn rates(raw: Vec<RateLimit>) -> Result<Vec<ValidatedRate>, ProxyError> {
    let mut rates = Vec::new();
    rates
        .try_reserve(raw.len())
        .map_err(|_| ProxyError::Allocation)?;
    raw.into_iter().try_for_each(|limit| {
        let per_second = NonZeroU32::new(limit.requests_per_second)
            .ok_or(ProxyError::Configuration("duration or rate"))?;
        rates.push(ValidatedRate {
            filter: limit.filter,
            per_second,
        });
        Ok::<(), ProxyError>(())
    })?;
    Ok(rates)
}
