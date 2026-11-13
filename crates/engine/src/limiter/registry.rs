//! Exact bounded bucket registration.
use super::*;

impl ApiExecutor {
    pub(super) fn bucket(&self, key: &str) -> Result<Arc<Bucket>, ApiError> {
        validate_key(key)?;
        let mut buckets = self
            .buckets
            .lock()
            .map_err(|_| ApiError::Permanent("bucket registry poisoned".into()))?;
        if let Some(bucket) = buckets.get(key) {
            return Ok(Arc::clone(bucket));
        }
        if buckets.len() == MAX_BUCKETS {
            return Err(ApiError::Exhausted(
                "bucket registry capacity reached".into(),
            ));
        }
        buckets
            .try_reserve(1)
            .map_err(|_| ApiError::Exhausted("bucket registry allocation failed".into()))?;
        let limits = match self.policies.get(key) {
            Some(limits) => *limits,
            None => BucketPolicy::from(BucketLimits::defaults(self.config))
                .validate(self.config.global_concurrency)?,
        };
        let bucket = Arc::new(Bucket::new(limits));
        if buckets
            .insert(key.to_owned(), Arc::clone(&bucket))
            .is_some()
        {
            return Err(ApiError::Permanent("duplicate bucket registration".into()));
        }
        increment(&self.counters.active_buckets);
        Ok(bucket)
    }
}
