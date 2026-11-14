//! Executor construction and validated policy registration.
use super::*;

impl ApiExecutor {
    /// Build default per-key policies with an independently randomized jitter seed.
    /// # Errors
    /// Returns `InvalidConfig` for zero/excessive concurrency, rate, retry or duration bounds.
    pub fn new(config: ExecutorConfig) -> Result<Self, ApiError> {
        Self::with_seed(config, RandomState::new().hash_one("inventory-api-jitter"))
    }

    /// Supply a seed for reproducible retry schedules in tests/replay.
    /// # Errors
    /// Returns `InvalidConfig` when any executor bound is zero or exceeds its hard limit.
    pub fn with_seed(config: ExecutorConfig, jitter_seed: u64) -> Result<Self, ApiError> {
        Self::build(config, BTreeMap::new(), jitter_seed)
    }

    /// Apply validated, immutable limits to exact bucket keys. This does not
    /// discover AWS quotas or share limits with other processes.
    /// # Errors
    /// Returns `InvalidConfig` for invalid defaults or more than 65,536 policies.
    /// Rejects empty or over-512-byte keys and invalid per-key rate/concurrency bounds.
    pub fn with_bucket_policies(
        config: ExecutorConfig,
        policies: BTreeMap<String, BucketLimits>,
    ) -> Result<Self, ApiError> {
        if policies.len() > MAX_BUCKETS {
            return Err(ApiError::InvalidConfig(
                "at most 65536 bucket policies are permitted".into(),
            ));
        }
        Self::with_rate_policies(
            config,
            policies
                .into_iter()
                .take(MAX_BUCKETS)
                .map(|(key, limits)| (key, BucketPolicy::from(limits)))
                .collect(),
        )
    }

    /// Add exact requests-per-period policies while preserving the integer
    /// policy shape. All policy validation precedes request admission.
    /// # Errors
    /// Returns `InvalidConfig` for invalid defaults, excessive policy-map/key bounds,
    /// zero/excessive rate windows/counts, over-100,000-RPS ratios or invalid concurrency.
    pub fn with_rate_policies(
        config: ExecutorConfig,
        policies: BTreeMap<String, BucketPolicy>,
    ) -> Result<Self, ApiError> {
        Self::build(
            config,
            policies,
            RandomState::new().hash_one("inventory-api-jitter"),
        )
    }

    pub(super) fn build(
        config: ExecutorConfig,
        policies: BTreeMap<String, BucketPolicy>,
        jitter_seed: u64,
    ) -> Result<Self, ApiError> {
        let config = config.validate()?;
        let policies = validate_policies(config, policies)?;
        Ok(Self {
            config,
            global: Semaphore::new(config.global_concurrency),
            pending: Semaphore::new(MAX_PENDING_OPERATIONS),
            buckets: Mutex::new(HashMap::new()),
            policies,
            cancellation: CancellationToken::new(),
            counters: Counters::default(),
            jitter_seed,
            nonce: AtomicU64::new(0),
        })
    }
}
