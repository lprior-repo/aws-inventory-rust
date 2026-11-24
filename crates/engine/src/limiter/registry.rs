use std::collections::BinaryHeap;

use super::*;

/// Idle buckets one eviction pass removes, so a pass frees room for many
/// later keys and the scan under the registry lock stays rare.
const EVICT_BATCH: usize = 1_024;

/// The live buckets and the reduced rates of buckets that were evicted.
#[derive(Debug, Default)]
pub(super) struct Registry {
    buckets: HashMap<String, Arc<Bucket>>,
    learned: HashMap<String, AdaptiveRate>,
}

/// What one eviction pass removed.
#[derive(Clone, Copy, Debug, Default)]
struct Evicted {
    buckets: usize,
    forgotten_rates: usize,
}

impl ApiExecutor {
    pub(super) fn bucket(&self, key: &str) -> Result<Arc<Bucket>, ApiError> {
        validate_key(key)?;
        let mut registry = self
            .registry
            .lock()
            .map_err(|_| ApiError::Permanent("bucket registry poisoned".into()))?;
        if let Some(bucket) = registry.buckets.get(key) {
            return Ok(Arc::clone(bucket));
        }
        if registry.buckets.len() == MAX_BUCKETS {
            self.make_room(&mut registry)?;
        }
        let limits = match self.policies.get(key) {
            Some(limits) => *limits,
            None => BucketPolicy::from(BucketLimits::defaults(self.config))
                .validate(self.config.global_concurrency)?,
        };
        let bucket = registry.create(key, limits)?;
        self.counters.record_registry(&registry);
        Ok(bucket)
    }

    fn make_room(&self, registry: &mut Registry) -> Result<(), ApiError> {
        let evicted = registry.evict(Instant::now())?;
        if evicted.buckets == 0 {
            return Err(ApiError::Exhausted(
                "bucket registry capacity reached".into(),
            ));
        }
        self.counters.record_evicted(evicted.buckets);
        if evicted.forgotten_rates != 0 {
            tracing::warn!(
                forgotten_rates = evicted.forgotten_rates,
                "learned-rate map is full; evicted buckets restart at their policy rate"
            );
        }
        Ok(())
    }
}

impl Registry {
    pub(super) fn active(&self) -> usize {
        self.buckets.len()
    }

    pub(super) fn learned(&self) -> usize {
        self.learned.len()
    }

    /// Adds a bucket for `key`, resuming the rate it was evicted with.
    fn create(&mut self, key: &str, limits: ValidatedLimits) -> Result<Arc<Bucket>, ApiError> {
        self.buckets
            .try_reserve(1)
            .map_err(|_| ApiError::Exhausted("bucket registry allocation failed".into()))?;
        let rate = self.learned.remove(key).unwrap_or(limits.rate);
        let bucket = Arc::new(Bucket::new(ValidatedLimits { rate, ..limits }));
        if self
            .buckets
            .insert(key.to_owned(), Arc::clone(&bucket))
            .is_some()
        {
            return Err(ApiError::Permanent("duplicate bucket registration".into()));
        }
        Ok(bucket)
    }

    /// Removes up to `EVICT_BATCH` idle buckets, least recently used first.
    ///
    /// A bucket is idle when no call holds it and its pacing has nothing left
    /// to enforce. Only the registry clones a bucket and it does so under the
    /// registry lock, so a count of one here means no call can be using it.
    fn evict(&mut self, now: Instant) -> Result<Evicted, ApiError> {
        let mut oldest = BinaryHeap::new();
        oldest
            .try_reserve_exact(EVICT_BATCH)
            .map_err(|_| ApiError::Exhausted("bucket eviction allocation failed".into()))?;
        for (key, bucket) in &self.buckets {
            if Arc::strong_count(bucket) != 1 {
                continue;
            }
            let Some(idle) = bucket.idle(now) else {
                continue;
            };
            let candidate = (idle.last_used, key.as_str());
            if oldest.len() < EVICT_BATCH {
                oldest.push(candidate);
            } else if let Some(mut newest) = oldest.peek_mut()
                && candidate < *newest
            {
                *newest = candidate;
            }
        }
        let keys = oldest
            .into_iter()
            .map(|(_, key)| key.to_owned())
            .collect::<Vec<_>>();
        Ok(keys
            .iter()
            .fold(Evicted::default(), |evicted, key| self.remove(evicted, key)))
    }

    fn remove(&mut self, evicted: Evicted, key: &str) -> Evicted {
        let Some((key, bucket)) = self.buckets.remove_entry(key) else {
            return evicted;
        };
        let buckets = evicted.buckets.saturating_add(1);
        let Some(rate) = bucket.reduced_rate() else {
            return Evicted { buckets, ..evicted };
        };
        if self.learned.len() < MAX_LEARNED_RATES && self.learned.try_reserve(1).is_ok() {
            self.learned.insert(key, rate);
            return Evicted { buckets, ..evicted };
        }
        Evicted {
            buckets,
            forgotten_rates: evicted.forgotten_rates.saturating_add(1),
        }
    }
}
