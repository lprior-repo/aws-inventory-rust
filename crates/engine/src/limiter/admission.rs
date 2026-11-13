//! Fair permit admission without holding capacity during pacing.
use super::*;

impl ApiExecutor {
    pub(super) fn ensure_running(&self) -> Result<(), ApiError> {
        if self.cancellation.is_cancelled() {
            Err(ApiError::Cancelled)
        } else {
            Ok(())
        }
    }

    #[tracing::instrument(skip_all)]
    pub(super) async fn acquire<'a>(
        &self,
        semaphore: &'a Semaphore,
        bucket: &Bucket,
    ) -> Result<SemaphorePermit<'a>, ApiError> {
        tokio::select! {
            biased;
            () = self.cancellation.cancelled() => Err(ApiError::Cancelled),
            () = bucket.rejected.cancelled() => Err(bucket.rejection_error()),
            permit = semaphore.acquire() => permit.map_err(|_| ApiError::Cancelled),
        }
    }

    #[tracing::instrument(skip_all)]
    pub(super) async fn wait_until(
        &self,
        deadline: Option<Instant>,
        bucket: &Bucket,
    ) -> Result<(), ApiError> {
        let Some(deadline) = deadline else {
            return self.ensure_running();
        };
        tokio::select! {
            biased;
            () = self.cancellation.cancelled() => Err(ApiError::Cancelled),
            () = bucket.rejected.cancelled() => Err(bucket.rejection_error()),
            () = sleep_until(deadline) => Ok(()),
        }
    }

    #[tracing::instrument(skip_all)]
    pub(super) async fn admit<'a>(
        &'a self,
        bucket: &'a Bucket,
    ) -> Result<AttemptPermit<'a>, ApiError> {
        let local = self.acquire(&bucket.in_flight, bucket).await?;
        let sequence = self.acquire(&bucket.dispatch, bucket).await?;
        // While sequence is owned, no new same-bucket attempt can start. At
        // most bucket concurrency - 1 old attempts can change its rate.
        // Each recheck consumes one such change; the bound includes initial
        // admission and one spare round. No rate ticket can accumulate a burst.
        for _ in 0..MAX_ADMISSION_ROUNDS {
            self.wait_until(bucket.due_at()?, bucket).await?;
            let global = self.acquire(&self.global, bucket).await?;
            let now = Instant::now();
            self.ensure_running()?;
            if !bucket.try_start(now)? {
                drop(global);
                continue;
            }
            drop(sequence);
            return Ok(AttemptPermit::new(global, local, &self.counters));
        }
        Err(ApiError::Exhausted(
            "admission recheck ceiling reached".into(),
        ))
    }
}
pub(super) struct AttemptPermit<'a> {
    pub(super) _global: SemaphorePermit<'a>,
    pub(super) _local: SemaphorePermit<'a>,
    pub(super) counters: &'a Counters,
}

impl<'a> AttemptPermit<'a> {
    pub(super) fn new(
        global: SemaphorePermit<'a>,
        local: SemaphorePermit<'a>,
        counters: &'a Counters,
    ) -> Self {
        let current = increment(&counters.in_flight);
        counters
            .peak_in_flight
            .fetch_max(current, Ordering::Relaxed);
        Self {
            _global: global,
            _local: local,
            counters,
        }
    }
}

impl Drop for AttemptPermit<'_> {
    fn drop(&mut self) {
        self.counters
            .in_flight
            .update(Ordering::Relaxed, Ordering::Relaxed, |value| {
                value.saturating_sub(1)
            });
    }
}
