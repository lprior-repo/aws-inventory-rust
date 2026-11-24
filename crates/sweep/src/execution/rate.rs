//! The pacing policy every limiter bucket of a sweep starts with.

use inventory_engine::limiter::{AdaptiveLimits, BucketPolicy, MAX_REQUESTS_PER_SECOND};

/// Requests per second a bucket starts at.
pub(super) const START_RPS: u32 = 10;

/// Requests per second a throttled bucket never goes below.
pub(super) const FLOOR_RPS: u32 = 1;

/// A rate that rises from the start until throttled and never passes `max_rps`.
pub(super) fn policy(max_rps: Option<u32>, concurrency: usize) -> BucketPolicy {
    let ceiling = max_rps.unwrap_or(MAX_REQUESTS_PER_SECOND);
    let limits = AdaptiveLimits {
        start_rps: START_RPS.min(ceiling),
        floor_rps: FLOOR_RPS,
        ceiling_rps: ceiling,
        concurrency,
    };
    BucketPolicy::Adaptive(limits)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn limits(policy: BucketPolicy) -> Result<AdaptiveLimits, &'static str> {
        match policy {
            BucketPolicy::Adaptive(limits) => Ok(limits),
            _ => Err("not a rising policy"),
        }
    }

    #[test]
    fn given_no_cap_when_built_then_the_rate_starts_at_ten_and_may_rise_to_the_limiter_maximum()
    -> Result<(), &'static str> {
        // Given no rate cap.
        // When the policy is built.
        let built = limits(policy(None, 8))?;
        // Then it starts at ten and may rise to the limiter maximum.
        assert_eq!(
            built,
            AdaptiveLimits {
                start_rps: 10,
                floor_rps: 1,
                ceiling_rps: 100_000,
                concurrency: 8
            }
        );
        Ok(())
    }

    #[test]
    fn given_a_cap_above_the_start_when_built_then_the_cap_is_the_ceiling()
    -> Result<(), &'static str> {
        // Given a rate cap above the start.
        // When the policy is built.
        let built = limits(policy(Some(50), 4))?;
        // Then the cap becomes the ceiling.
        assert_eq!(
            built,
            AdaptiveLimits {
                start_rps: 10,
                floor_rps: 1,
                ceiling_rps: 50,
                concurrency: 4
            }
        );
        Ok(())
    }

    #[test]
    fn given_a_cap_below_the_start_when_built_then_the_rate_starts_at_the_cap()
    -> Result<(), &'static str> {
        // Given a rate cap below the start.
        // When the policy is built.
        let built = limits(policy(Some(3), 2))?;
        // Then the rate starts at the cap.
        assert_eq!(
            built,
            AdaptiveLimits {
                start_rps: 3,
                floor_rps: 1,
                ceiling_rps: 3,
                concurrency: 2
            }
        );
        Ok(())
    }

    #[test]
    fn given_a_cap_of_one_when_built_then_start_floor_and_ceiling_are_one()
    -> Result<(), &'static str> {
        // Given a rate cap of one.
        // When the policy is built.
        let built = limits(policy(Some(1), 1))?;
        // Then start, floor and ceiling are one.
        assert_eq!(
            built,
            AdaptiveLimits {
                start_rps: 1,
                floor_rps: 1,
                ceiling_rps: 1,
                concurrency: 1
            }
        );
        Ok(())
    }
}
