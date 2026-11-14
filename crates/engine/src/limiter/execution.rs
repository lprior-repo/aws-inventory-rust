//! Cancellation-aware bounded execution and request accounting.
use super::*;

impl ApiExecutor {
    /// Cancel all active operations and admission/backoff waits. Cancellation is
    /// permanent for this executor. The caller owns and must drain its futures.
    pub fn cancel(&self) {
        self.cancellation.cancel();
    }

    /// Check cancellation before admitting orchestration work outside API calls.
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.cancellation.is_cancelled()
    }

    /// Observe cancellation during configuration or other caller-owned waits.
    /// The returned future owns no task and is safe to drop.
    #[tracing::instrument(skip_all)]
    pub async fn cancelled(&self) {
        self.cancellation.cancelled().await;
    }

    /// Snapshot cumulative counters and current admission occupancy; reads are not transactional.
    #[must_use]
    pub fn stats(&self) -> ExecutorStats {
        self.counters.snapshot()
    }

    /// Execute a repeatable, drop-safe, nonblocking operation; SDK retries must
    /// be disabled. Tokio timeouts cannot preempt blocking future polling.
    /// This function does not buffer `T`; payload limits belong to the adapter.
    /// Dropping this future releases every reservation and counts cancellation.
    /// # Errors
    /// Returns terminal adapter errors, cancellation, invalid/bounded quota-key
    /// admission, rejected retry hints or exhausted retry/deadline budgets.
    #[tracing::instrument(skip(self, operation), fields(quota_bucket = bucket))]
    pub async fn execute<T, F, Fut>(&self, bucket: &str, operation: F) -> Result<T, ApiError>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<T, ApiError>>,
    {
        let mut record = RequestRecord::new(&self.counters);
        let result = self.execute_inner(bucket, operation).await;
        record.complete(&result);
        result
    }

    #[tracing::instrument(skip_all)]
    pub(super) async fn execute_inner<T, F, Fut>(
        &self,
        key: &str,
        mut operation: F,
    ) -> Result<T, ApiError>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<T, ApiError>>,
    {
        self.ensure_running()?;
        let _pending = self.pending.try_acquire().map_err(|_| {
            ApiError::Exhausted("executor pending-operation capacity reached".into())
        })?;
        let bucket = self.bucket(key)?;
        // Wrapping is intentional for a jitter nonce, not a resource counter.
        let nonce = self.nonce.fetch_add(1, Ordering::Relaxed);
        for attempt in 0..MAX_ATTEMPTS {
            if attempt >= self.config.max_attempts {
                break;
            }
            let result = self.attempt(&bucket, attempt, &mut operation).await;
            match result {
                Ok(value) => return Ok(value),
                Err(error) if retryable(&error) => {
                    if attempt.saturating_add(1) == self.config.max_attempts {
                        return Err(exhausted(error));
                    }
                    self.backoff(&bucket, nonce, attempt).await?;
                }
                Err(error) => return Err(error),
            }
        }
        Err(ApiError::Exhausted("static attempt ceiling reached".into()))
    }
}
pub(super) struct RequestRecord<'a> {
    pub(super) counters: &'a Counters,
    pub(super) complete: bool,
}

impl<'a> RequestRecord<'a> {
    pub(super) fn new(counters: &'a Counters) -> Self {
        Self {
            counters,
            complete: false,
        }
    }

    pub(super) fn complete<T>(&mut self, result: &Result<T, ApiError>) {
        match result {
            Ok(_) => increment(&self.counters.successes),
            Err(ApiError::Cancelled) => increment(&self.counters.cancelled),
            Err(_) => increment(&self.counters.failures),
        };
        self.complete = true;
    }
}

impl Drop for RequestRecord<'_> {
    fn drop(&mut self) {
        if !self.complete {
            increment(&self.counters.cancelled);
        }
    }
}
