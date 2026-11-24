use super::*;

impl ApiExecutor {
    pub fn new(config: ExecutorConfig) -> Result<Self, ApiError> {
        Self::with_seed(config, RandomState::new().hash_one("inventory-api-jitter"))
    }

    pub fn with_seed(config: ExecutorConfig, jitter_seed: u64) -> Result<Self, ApiError> {
        Self::build(config, BTreeMap::new(), jitter_seed)
    }

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
            registry: Mutex::new(Registry::default()),
            policies,
            cancellation: CancellationToken::new(),
            counters: Counters::default(),
            jitter_seed,
            nonce: AtomicU64::new(0),
        })
    }
}
