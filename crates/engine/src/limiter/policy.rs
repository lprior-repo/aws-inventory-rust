use super::*;

#[derive(Clone, Copy, Debug, Serialize)]
pub struct ExecutorConfig {
    pub global_concurrency: usize,

    pub per_bucket_concurrency: usize,

    pub requests_per_second: u32,

    pub max_attempts: u32,

    pub attempt_timeout_ms: u64,

    pub max_backoff_ms: u64,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            global_concurrency: 64,
            per_bucket_concurrency: 8,
            requests_per_second: 5,
            max_attempts: 5,
            attempt_timeout_ms: 30_000,
            max_backoff_ms: 30_000,
        }
    }
}

impl ExecutorConfig {
    pub(super) fn validate(self) -> Result<Self, ApiError> {
        if !(1..=MAX_CONCURRENCY).contains(&self.global_concurrency)
            || !(1..=self.global_concurrency).contains(&self.per_bucket_concurrency)
        {
            return Err(ApiError::InvalidConfig(
                "require 1 <= per_bucket_concurrency <= global_concurrency <= 4096".into(),
            ));
        }
        if !(1..=MAX_REQUESTS_PER_SECOND).contains(&self.requests_per_second)
            || !(1..=MAX_ATTEMPTS).contains(&self.max_attempts)
        {
            return Err(ApiError::InvalidConfig(
                "require RPS in 1..=100000 and attempts in 1..=10".into(),
            ));
        }
        if !(1..=MAX_DURATION_MS).contains(&self.attempt_timeout_ms)
            || !(1..=MAX_DURATION_MS).contains(&self.max_backoff_ms)
        {
            return Err(ApiError::InvalidConfig(
                "timeout and backoff must be in 1..=300000 milliseconds".into(),
            ));
        }
        Ok(self)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BucketLimits {
    pub requests_per_second: u32,

    pub concurrency: usize,
}

impl BucketLimits {
    pub(super) fn validate(self, global_concurrency: usize) -> Result<(), ApiError> {
        if !(1..=MAX_REQUESTS_PER_SECOND).contains(&self.requests_per_second)
            || !(1..=global_concurrency).contains(&self.concurrency)
        {
            return Err(ApiError::InvalidConfig(
                "bucket policy requires RPS in 1..=100000 and concurrency in 1..=global_concurrency".into(),
            ));
        }
        Ok(())
    }

    pub(super) fn defaults(config: ExecutorConfig) -> Self {
        Self {
            requests_per_second: config.requests_per_second,
            concurrency: config.per_bucket_concurrency,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PeriodLimits {
    pub requests: u32,

    pub period_ms: u64,

    pub concurrency: usize,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(untagged)]
pub enum BucketPolicy {
    PerSecond(BucketLimits),

    PerPeriod(PeriodLimits),
}

impl From<BucketLimits> for BucketPolicy {
    fn from(limits: BucketLimits) -> Self {
        Self::PerSecond(limits)
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct ValidatedLimits {
    pub(super) concurrency: usize,
    pub(super) rate: AdaptiveRate,
}

impl BucketPolicy {
    pub(super) fn validate(self, global_concurrency: usize) -> Result<ValidatedLimits, ApiError> {
        match self {
            Self::PerSecond(limits) => {
                limits.validate(global_concurrency)?;
                Ok(ValidatedLimits {
                    concurrency: limits.concurrency,
                    rate: AdaptiveRate::PerSecond {
                        current: limits.requests_per_second,
                        maximum: limits.requests_per_second,
                    },
                })
            }
            Self::PerPeriod(limits) => limits.validate(global_concurrency),
        }
    }
}

impl PeriodLimits {
    pub(super) fn validate(self, global_concurrency: usize) -> Result<ValidatedLimits, ApiError> {
        if !(1..=MAX_REQUESTS_PER_SECOND).contains(&self.requests)
            || !(1..=MAX_RATE_PERIOD_MS).contains(&self.period_ms)
            || !(1..=global_concurrency).contains(&self.concurrency)
        {
            return Err(ApiError::InvalidConfig(
                "period policy requires requests in 1..=100000, period_ms in 1..=300000, and concurrency in 1..=global_concurrency".into(),
            ));
        }
        let numerator = u64::from(self.requests).checked_mul(1_000);
        let maximum = u64::from(MAX_REQUESTS_PER_SECOND).checked_mul(self.period_ms);
        if numerator.is_none() || maximum.is_none() || numerator > maximum {
            return Err(ApiError::InvalidConfig(
                "period policy exceeds 100000 requests per second".into(),
            ));
        }
        let period_ns = self
            .period_ms
            .checked_mul(NANOS_PER_MILLISECOND)
            .ok_or_else(|| ApiError::InvalidConfig("rate period overflow".into()))?;
        let interval_ns = ceil_div(period_ns, u64::from(self.requests))?;
        Ok(ValidatedLimits {
            concurrency: self.concurrency,
            rate: AdaptiveRate::PerPeriod {
                interval_ns,
                minimum_ns: interval_ns,
                recovery_rps: 0,
            },
        })
    }
}

pub(super) fn validate_key(key: &str) -> Result<(), ApiError> {
    if key.is_empty() || key.len() > MAX_BUCKET_KEY_BYTES {
        return Err(ApiError::InvalidConfig(
            "bucket key must be 1..=512 bytes".into(),
        ));
    }
    Ok(())
}

pub(super) fn validate_policies(
    config: ExecutorConfig,
    policies: BTreeMap<String, BucketPolicy>,
) -> Result<BTreeMap<String, ValidatedLimits>, ApiError> {
    if policies.len() > MAX_BUCKETS {
        return Err(ApiError::InvalidConfig(
            "at most 65536 bucket policies are permitted".into(),
        ));
    }
    policies
        .into_iter()
        .take(MAX_BUCKETS)
        .map(|(key, limits)| {
            validate_key(&key)?;
            Ok((key, limits.validate(config.global_concurrency)?))
        })
        .collect()
}
