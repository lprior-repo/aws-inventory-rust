use super::*;

#[derive(Debug)]
pub(super) struct Bucket {
    pub(super) in_flight: Semaphore,
    pub(super) dispatch: Semaphore,
    pub(super) state: Mutex<PacingState>,
    pub(super) rejected: CancellationToken,
}

/// A bucket that can be dropped without losing pacing.
#[derive(Clone, Copy, Debug)]
pub(super) struct Idle {
    pub(super) last_used: Option<Instant>,
}

impl Bucket {
    pub(super) fn new(limits: ValidatedLimits) -> Self {
        Self {
            in_flight: Semaphore::new(limits.concurrency),
            dispatch: Semaphore::new(1),
            state: Mutex::new(PacingState {
                last_start: None,
                cooldown_until: None,
                rejected: None,
                rate: limits.rate,
            }),
            rejected: CancellationToken::new(),
        }
    }

    pub(super) fn due_at(&self) -> Result<Option<Instant>, ApiError> {
        self.state.lock().map_err(|_| poisoned_pacing())?.due_at()
    }

    /// When this bucket last started a call, if its pacing has nothing left
    /// to enforce: no rejection, no cooldown and no interval still running.
    pub(super) fn idle(&self, now: Instant) -> Option<Idle> {
        let state = self.state.lock().ok()?;
        match state.due_at() {
            Ok(due) if due.is_none_or(|due| due <= now) => Some(Idle {
                last_used: state.last_start,
            }),
            Ok(_) | Err(_) => None,
        }
    }

    /// The current rate, unless the pacing state is poisoned.
    pub(super) fn rate(&self) -> Option<AdaptiveRate> {
        Some(self.state.lock().ok()?.rate)
    }

    /// The rate to resume with after eviction, when results have moved it
    /// from where its policy starts it.
    pub(super) fn learned_rate(&self) -> Option<AdaptiveRate> {
        self.rate().filter(|rate| rate.is_learned())
    }

    pub(super) fn try_start(&self, now: Instant) -> Result<bool, ApiError> {
        let mut state = self.state.lock().map_err(|_| poisoned_pacing())?;
        if state.due_at()?.is_some_and(|due| due > now) {
            return Ok(false);
        }
        state.last_start = Some(now);
        Ok(true)
    }

    pub(super) fn feedback<T>(&self, result: &Result<T, ApiError>) -> Result<(), ApiError> {
        let outcome = self
            .state
            .lock()
            .map_err(|_| poisoned_pacing())?
            .feedback(result, Instant::now());
        if matches!(outcome, Err(ApiError::RetryAfterRejected(_))) {
            self.rejected.cancel();
        }
        outcome
    }

    pub(super) fn rejection_error(&self) -> ApiError {
        match self.state.lock() {
            Ok(state) => state.rejected.map_or_else(
                || ApiError::Permanent("missing bucket rejection cause".into()),
                RejectedHint::error,
            ),
            Err(_) => poisoned_pacing(),
        }
    }
}

pub(super) fn poisoned_pacing() -> ApiError {
    ApiError::Permanent("bucket pacing state poisoned".into())
}
