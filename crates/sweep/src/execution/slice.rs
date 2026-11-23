/// Wall-clock budget for one sweep invocation.
///
/// A bounded invocation (for example a Lambda run) stops claiming new work
/// once the budget is spent and leaves the rest of the sweep for a later
/// `--resume` invocation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct SliceBudget {
    deadline_ms: Option<i64>,
}

impl SliceBudget {
    /// No limit: drain until the queue is empty.
    pub(super) const fn unbounded() -> Self {
        Self { deadline_ms: None }
    }

    /// Budget of `timeout_secs` starting at `started_ms`; a saturated
    /// deadline means "never expires" rather than wrapping.
    pub(super) fn starting_at(started_ms: i64, timeout_secs: u64) -> Self {
        let millis = i64::try_from(timeout_secs)
            .unwrap_or(i64::MAX)
            .saturating_mul(1_000);
        Self {
            deadline_ms: Some(started_ms.saturating_add(millis)),
        }
    }

    pub(super) fn exhausted(self, now_ms: i64) -> bool {
        self.deadline_ms.is_some_and(|deadline| now_ms >= deadline)
    }
}

#[cfg(test)]
mod tests {
    use super::SliceBudget;

    #[test]
    fn given_no_timeout_when_any_time_passes_then_budget_is_never_exhausted() {
        assert!(!SliceBudget::unbounded().exhausted(i64::MAX));
    }

    #[test]
    fn given_timeout_when_clock_reaches_deadline_then_budget_is_exhausted_exactly_there() {
        let budget = SliceBudget::starting_at(10_000, 5);
        assert!(!budget.exhausted(14_999));
        assert!(budget.exhausted(15_000));
        assert!(budget.exhausted(20_000));
    }

    #[test]
    fn given_clock_before_start_when_checked_then_budget_is_not_exhausted() {
        assert!(!SliceBudget::starting_at(10_000, 5).exhausted(0));
    }

    #[test]
    fn given_huge_timeout_when_built_then_deadline_saturates_instead_of_wrapping() {
        let budget = SliceBudget::starting_at(1_000, u64::MAX);
        assert!(!budget.exhausted(i64::MAX - 1));
    }

    #[test]
    fn given_zero_timeout_when_checked_then_budget_is_exhausted_at_once() {
        assert!(SliceBudget::starting_at(500, 0).exhausted(500));
    }
}
