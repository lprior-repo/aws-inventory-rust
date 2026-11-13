//! Pure adaptive pacing and server retry-directive state.
use super::*;

#[derive(Clone, Copy, Debug)]
pub(super) enum AdaptiveRate {
    PerSecond {
        current: u32,
        maximum: u32,
    },
    PerPeriod {
        interval_ns: u64,
        minimum_ns: u64,
        recovery_rps: u32,
    },
}

impl AdaptiveRate {
    pub(super) fn interval_ns(self) -> Result<u64, ApiError> {
        match self {
            Self::PerSecond { current, .. } => ceil_div(NANOS_PER_SECOND, u64::from(current)),
            Self::PerPeriod {
                interval_ns,
                minimum_ns,
                recovery_rps,
            } => period_interval(interval_ns, minimum_ns, recovery_rps),
        }
    }

    pub(super) fn throttle(&mut self) -> Result<(), ApiError> {
        let previous_interval = self.interval_ns()?;
        match self {
            Self::PerSecond { current, .. } => *current = current.saturating_div(2).max(1),
            Self::PerPeriod {
                interval_ns,
                recovery_rps,
                ..
            } => {
                *interval_ns = previous_interval
                    .saturating_mul(2)
                    .min(MAX_RATE_PERIOD_MS.saturating_mul(NANOS_PER_MILLISECOND));
                *recovery_rps = 0;
            }
        }
        Ok(())
    }

    pub(super) fn recover(&mut self) {
        match self {
            Self::PerSecond { current, maximum } => {
                *current = current.saturating_add(1).min(*maximum)
            }
            Self::PerPeriod { recovery_rps, .. } => {
                // Retaining +1 RPS credits prevents nanosecond rounding from
                // discarding every increment at high configured rates.
                *recovery_rps = recovery_rps.saturating_add(1).min(MAX_REQUESTS_PER_SECOND);
            }
        }
    }
}

pub(super) fn period_interval(
    base_ns: u64,
    minimum_ns: u64,
    recovery_rps: u32,
) -> Result<u64, ApiError> {
    let numerator = u128::from(NANOS_PER_SECOND).checked_mul(u128::from(base_ns));
    let denominator = u128::from(base_ns)
        .checked_mul(u128::from(recovery_rps))
        .and_then(|increment| u128::from(NANOS_PER_SECOND).checked_add(increment));
    let next = numerator
        .zip(denominator)
        .and_then(|(numerator, denominator)| {
            numerator
                .checked_add(denominator.saturating_sub(1))
                .and_then(|value| value.checked_div(denominator))
        })
        .and_then(|value| u64::try_from(value).ok())
        .ok_or_else(|| ApiError::Permanent("adaptive rate arithmetic overflow".into()))?;
    Ok(next.max(minimum_ns))
}

#[derive(Clone, Copy, Debug)]
pub(super) enum RejectedHint {
    Invalid,
    Excessive,
}

impl RejectedHint {
    pub(super) fn detail(self) -> &'static str {
        match self {
            Self::Invalid => "invalid server retry hint",
            Self::Excessive => {
                "server retry delay exceeds 300000 milliseconds or the monotonic clock range"
            }
        }
    }

    pub(super) fn error(self) -> ApiError {
        ApiError::RetryAfterRejected(self.detail().into())
    }
}

#[derive(Debug)]
pub(super) struct PacingState {
    pub(super) last_start: Option<Instant>,
    pub(super) cooldown_until: Option<Instant>,
    pub(super) rejected: Option<RejectedHint>,
    pub(super) rate: AdaptiveRate,
}

impl PacingState {
    pub(super) fn due_at(&self) -> Result<Option<Instant>, ApiError> {
        if let Some(rejected) = self.rejected {
            return Err(rejected.error());
        }
        let interval = Duration::from_nanos(self.rate.interval_ns()?);
        let paced = self
            .last_start
            .map(|last| {
                last.checked_add(interval)
                    .ok_or_else(|| ApiError::Permanent("pacing deadline overflow".into()))
            })
            .transpose()?;
        Ok(paced.max(self.cooldown_until))
    }

    pub(super) fn apply_hint(
        &mut self,
        hint: RetryAfter,
        now: Instant,
    ) -> Result<(), RejectedHint> {
        let delay = match hint {
            RetryAfter::Delay(delay) if delay <= Duration::from_millis(MAX_RETRY_AFTER_MS) => delay,
            RetryAfter::Delay(_) | RetryAfter::Excessive => return Err(RejectedHint::Excessive),
            RetryAfter::Invalid => return Err(RejectedHint::Invalid),
        };
        let deadline = now.checked_add(delay).ok_or(RejectedHint::Excessive)?;
        self.cooldown_until = self.cooldown_until.max(Some(deadline));
        Ok(())
    }

    pub(super) fn feedback<T>(
        &mut self,
        result: &Result<T, ApiError>,
        now: Instant,
    ) -> Result<(), ApiError> {
        if result.is_ok() {
            self.rate.recover();
        } else if result.as_ref().err().and_then(retry_kind) == Some(RetryKind::Throttled) {
            self.rate.throttle()?;
        }
        if let Err(ApiError::RetryAfter {
            kind,
            message,
            retry_after,
        }) = result
            && let Err(reason) = self.apply_hint(*retry_after, now)
        {
            self.rejected = Some(reason);
            return Err(ApiError::RetryAfterRejected(format!(
                "{}; {kind}: {}",
                reason.detail(),
                bounded_message(message)
            )));
        }
        Ok(())
    }
}
