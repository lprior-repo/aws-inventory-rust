use super::*;

impl ApiExecutor {
    #[tracing::instrument(skip_all)]
    pub(super) async fn attempt<T, F, Fut>(
        &self,
        bucket: &Bucket,
        attempt: u32,
        operation: &mut F,
    ) -> Result<T, ApiError>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<T, ApiError>>,
    {
        let _permit = self.admit(bucket).await?;
        increment(&self.counters.attempts);
        if attempt > 0 {
            increment(&self.counters.retries);
        }
        let result = tokio::select! {
            biased;
            () = self.cancellation.cancelled() => Err(ApiError::Cancelled),
            () = bucket.rejected.cancelled() => Err(bucket.rejection_error()),
            outcome = timeout(Duration::from_millis(self.config.attempt_timeout_ms), async {
                operation().await
            }) => match outcome {
                Ok(result) => result,
                Err(_) => Err(ApiError::Timeout),
            },
        };

        self.feedback(bucket, &result)?;
        result
    }

    pub(super) fn feedback<T>(
        &self,
        bucket: &Bucket,
        result: &Result<T, ApiError>,
    ) -> Result<(), ApiError> {
        match result.as_ref().err().and_then(retry_kind) {
            Some(RetryKind::Throttled) => {
                increment(&self.counters.throttles);
            }
            Some(RetryKind::Timeout) => {
                increment(&self.counters.timeouts);
            }
            Some(RetryKind::Transient) | None => {}
        }
        let outcome = bucket.feedback(result);
        if matches!(outcome, Err(ApiError::RetryAfterRejected(_))) {
            increment(&self.counters.rejected_retry_hints);
        } else if matches!(result, Err(ApiError::RetryAfter { .. })) && outcome.is_ok() {
            increment(&self.counters.server_delays);
        }
        outcome
    }

    #[tracing::instrument(skip_all)]
    pub(super) async fn backoff(
        &self,
        bucket: &Bucket,
        nonce: u64,
        attempt: u32,
    ) -> Result<(), ApiError> {
        let cap = BASE_BACKOFF_MS
            .saturating_mul(2_u64.saturating_pow(attempt))
            .min(self.config.max_backoff_ms);
        let random = mix(self.jitter_seed ^ nonce ^ u64::from(attempt).rotate_left(32));
        let milliseconds = random
            .checked_rem(cap.saturating_add(1))
            .ok_or_else(|| ApiError::Permanent("zero internal backoff range".into()))?;
        tokio::select! {
            biased;
            () = self.cancellation.cancelled() => Err(ApiError::Cancelled),
            () = bucket.rejected.cancelled() => Err(bucket.rejection_error()),
            () = sleep(Duration::from_millis(milliseconds)) => Ok(()),
        }
    }
}
pub(super) fn retryable(error: &ApiError) -> bool {
    retry_kind(error).is_some()
}

pub(super) fn retry_kind(error: &ApiError) -> Option<RetryKind> {
    match error {
        ApiError::Throttled(_) => Some(RetryKind::Throttled),
        ApiError::Transient(_) => Some(RetryKind::Transient),
        ApiError::Timeout => Some(RetryKind::Timeout),
        ApiError::RetryAfter { kind, .. } => Some(*kind),
        _ => None,
    }
}

pub(super) fn exhausted(error: ApiError) -> ApiError {
    let detail = match error {
        ApiError::Throttled(message) => format!("throttled: {}", bounded_message(&message)),
        ApiError::Transient(message) => format!("transient: {}", bounded_message(&message)),
        ApiError::Timeout => "attempt timeout".to_owned(),
        ApiError::RetryAfter {
            kind,
            message,
            retry_after,
        } => format!(
            "{kind}: {}; server retry directive: {retry_after:?}",
            bounded_message(&message)
        ),
        other => other.to_string(),
    };
    ApiError::Exhausted(detail)
}

pub(super) fn bounded_message(message: &str) -> String {
    message.chars().take(MAX_ERROR_CHARS).collect()
}

pub(super) fn mix(value: u64) -> u64 {
    let first = (value ^ value.wrapping_shr(30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    let second = (first ^ first.wrapping_shr(27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    second ^ second.wrapping_shr(31)
}
