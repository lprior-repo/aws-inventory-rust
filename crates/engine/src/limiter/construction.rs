//! Executor construction and validated policy registration.
use super::*;

impl ApiExecutor {
    pub fn new(config: ExecutorConfig) -> Result<Self, ApiError> {
        Self::with_seed(config, RandomState::new().hash_one("inventory-api-jitter"))
    }

    /// Supply a seed for reproducible retry schedules in tests/replay.
    pub fn with_seed(config: ExecutorConfig, jitter_seed: u64) -> Result<Self, ApiError> {
        Self::build(config, BTreeMap::new(), jitter_seed)
    }

    /// Apply validated, immutable limits to exact bucket keys. This does not
    /// discover AWS quotas or share limits with other processes.
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
