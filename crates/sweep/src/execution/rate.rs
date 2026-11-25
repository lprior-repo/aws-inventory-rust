use inventory_engine::limiter::{
    AdaptiveLimits, BucketPolicy, MAX_REQUESTS_PER_SECOND, PeriodLimits,
};

pub(super) const START_RPS: u32 = 10;

pub(super) const FLOOR_RPS: u32 = 1;

pub(super) fn policy(
    max_rps: Option<u32>,
    max_rps_milli: Option<u32>,
    concurrency: usize,
) -> BucketPolicy {
    match max_rps_milli {
        Some(rate) => fractional_policy(rate, concurrency),
        None => adaptive_policy(max_rps, concurrency),
    }
}

fn fractional_policy(rate: u32, concurrency: usize) -> BucketPolicy {
    let period_ms = u64::from(1_000_000_u32.div_ceil(rate));
    BucketPolicy::PerPeriod(PeriodLimits {
        requests: 1,
        period_ms,
        concurrency,
    })
}

fn adaptive_policy(max_rps: Option<u32>, concurrency: usize) -> BucketPolicy {
    let ceiling = max_rps.unwrap_or(MAX_REQUESTS_PER_SECOND);
    BucketPolicy::Adaptive(AdaptiveLimits {
        start_rps: START_RPS.min(ceiling),
        floor_rps: FLOOR_RPS,
        ceiling_rps: ceiling,
        concurrency,
    })
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
        let built = limits(policy(None, None, 8))?;
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
        let built = limits(policy(Some(50), None, 4))?;
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
        let built = limits(policy(Some(3), None, 2))?;
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
        let built = limits(policy(Some(1), None, 1))?;
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
