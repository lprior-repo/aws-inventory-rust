use super::*;

/// How long after a throttle the rate stays at or below 90% of the throttled rate.
pub(super) const HOLD: Duration = Duration::from_secs(60);

/// A request rate in milli-requests per second that rises with successes
/// and is cut by a throttle.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct Seeking {
    pub(super) current_mrps: u64,
    pub(super) floor_mrps: u64,
    pub(super) ceiling_mrps: u64,
    pub(super) hold: Option<Hold>,
    /// Results this rate has learned from: successes and throttles.
    pub(super) samples: u64,
}

/// The rate a throttle arrived at, and when.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct Hold {
    pub(super) throttled_mrps: u64,
    pub(super) since: Instant,
}

impl Seeking {
    /// Create a rate that starts at `start_rps` and may range from
    /// `floor_rps` to `ceiling_rps`.
    pub(super) fn new(start_rps: u32, floor_rps: u32, ceiling_rps: u32) -> Self {
        Self {
            current_mrps: u64::from(start_rps).saturating_mul(1_000),
            floor_mrps: u64::from(floor_rps).saturating_mul(1_000),
            ceiling_mrps: u64::from(ceiling_rps).saturating_mul(1_000),
            hold: None,
            samples: 0,
        }
    }

    /// Whole requests per second at the current rate, at least 1.
    pub(super) fn whole_rps(self) -> u32 {
        u32::try_from(self.current_mrps.saturating_div(1_000))
            .unwrap_or(u32::MAX)
            .max(1)
    }

    /// Nanoseconds per request at the current rate.
    pub(super) fn interval_ns(self) -> Result<u64, ApiError> {
        let numerator = NANOS_PER_SECOND.saturating_mul(1_000);
        ceil_div(numerator, self.current_mrps)
    }

    /// Cut the rate to 70%, never below the floor, and start the hold.
    pub(super) fn throttle(&mut self, now: Instant) {
        let before = self.current_mrps;
        self.hold = Some(Hold {
            throttled_mrps: before,
            since: now,
        });
        let cut = before.saturating_mul(7).saturating_div(10);
        self.current_mrps = cut.max(self.floor_mrps);
        self.samples = self.samples.saturating_add(1);
    }

    /// Raise the rate by one step, never above the ceiling nor, while the
    /// hold lasts, above 90% of the rate the last throttle arrived at.
    pub(super) fn recover(&mut self, now: Instant) {
        self.samples = self.samples.saturating_add(1);
        if let Some(hold) = self.hold
            && now.saturating_duration_since(hold.since) >= HOLD
        {
            self.hold = None;
        }
        let step = 1_000_000_u64
            .checked_div(self.current_mrps)
            .unwrap_or(1)
            .max(1);
        let limit = self.hold.map_or(self.ceiling_mrps, |hold| {
            let bound = hold.throttled_mrps.saturating_mul(9).saturating_div(10);
            self.ceiling_mrps.min(bound.max(self.floor_mrps))
        });
        let raised = self.current_mrps.saturating_add(step).min(limit);
        self.current_mrps = self.current_mrps.max(raised);
    }
}

#[cfg(test)]
mod tests;
