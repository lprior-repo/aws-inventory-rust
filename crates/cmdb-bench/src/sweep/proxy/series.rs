//! Arithmetic on recorded cumulative proxy response/throttle samples.
//! Cutoffs interpolate counts downwards; admitted rates are per second and shares ppm.
pub(super) type Sample = (u64, u64, u64);
fn validate(samples: &[Sample]) -> Result<(), String> {
    if samples.first().is_some_and(|sample| sample.0 != 0) {
        return Err("proxy recording missing origin".into());
    }
    samples.windows(2).try_for_each(|pair| {
        let mut pair = pair.iter();
        let a = pair.next().ok_or("missing sample")?;
        let b = pair.next().ok_or("missing sample")?;
        if b.0 <= a.0 || b.1 < a.1 || b.2 < a.2 {
            return Err("nonmonotonic proxy recording".into());
        }
        Ok(())
    })
}
fn scale(value: u64, multiplier: u64, divisor: u64) -> Result<u64, String> {
    let value = u128::from(value)
        .checked_mul(u128::from(multiplier))
        .and_then(|value| value.checked_div(u128::from(divisor)))
        .ok_or("undefined proxy ratio")?;
    u64::try_from(value).map_err(|error| error.to_string())
}
fn at(samples: &[Sample], elapsed: u64) -> Result<Sample, String> {
    let index = samples.partition_point(|sample| sample.0 <= elapsed);
    let left = index
        .checked_sub(1)
        .and_then(|index| samples.get(index))
        .ok_or("missing proxy origin")?;
    if left.0 == elapsed {
        return Ok(*left);
    }
    let right = samples.get(index).ok_or("missing proxy endpoint")?;
    let span = right.0.saturating_sub(left.0);
    let offset = elapsed.saturating_sub(left.0);
    Ok((
        elapsed,
        left.1
            .saturating_add(scale(right.1.saturating_sub(left.1), offset, span)?),
        left.2
            .saturating_add(scale(right.2.saturating_sub(left.2), offset, span)?),
    ))
}
pub(super) fn measurements(
    samples: &[Sample],
    rps: u32,
) -> Result<Vec<(&'static str, Option<u64>)>, String> {
    validate(samples)?;
    if rps == 0 {
        return Err("proxy quota is zero".into());
    }
    let end = samples.last().ok_or("empty proxy recording")?;
    let steady = if end.0 >= 2 {
        let start = at(samples, end.0 / 2)?;
        Some(scale(
            admitted(*end, start),
            1_000,
            end.0.saturating_sub(start.0),
        )?)
    } else {
        None
    };
    let share = if end.0 > 30_000 {
        let start = at(samples, 30_000)?;
        let responses = end.1.saturating_sub(start.1);
        if responses == 0 {
            None
        } else {
            Some(scale(end.2.saturating_sub(start.2), 1_000_000, responses)?)
        }
    } else {
        None
    };
    let ramp = samples
        .iter()
        .filter(|sample| sample.0 >= 1000)
        .try_fold(None, |found, end| {
            if found.is_some() {
                return Ok::<_, String>(found);
            }
            let start = at(samples, end.0.saturating_sub(1000))?;
            Ok((u128::from(admitted(*end, start)).saturating_mul(10)
                >= u128::from(rps).saturating_mul(9))
            .then_some(end.0))
        })?;
    Ok(vec![
        ("proxy.steady_requests_per_second", steady),
        ("proxy.throttle_ppm_after_30s", share),
        ("proxy.ms_to_90_percent_of_limit", ramp),
    ])
}
fn admitted(end: Sample, start: Sample) -> u64 {
    end.1
        .saturating_sub(start.1)
        .saturating_sub(end.2.saturating_sub(start.2))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn recorded_log_excludes_early_traffic_and_uses_trailing_windows() -> Result<(), String> {
        let rows = measurements(&[(0, 0, 0), (30_000, 100, 90), (40_000, 103, 91)], 20)?;
        assert_eq!(
            rows.get(1),
            Some(&("proxy.throttle_ppm_after_30s", Some(333_333)))
        );
        let rows = measurements(&[(0, 0, 0), (100, 1, 0), (1000, 17, 0), (1100, 19, 0)], 20)?;
        assert_eq!(
            rows.last(),
            Some(&("proxy.ms_to_90_percent_of_limit", Some(1100)))
        );
        let rows = measurements(&[(0, 100, 4), (1000, 140, 4), (2000, 150, 4)], 20)?;
        assert_eq!(
            rows.first(),
            Some(&("proxy.steady_requests_per_second", Some(10)))
        );
        assert_eq!(rows.get(1), Some(&("proxy.throttle_ppm_after_30s", None)));
        assert!(measurements(&[(0, 0, 0), (100, 5, 0), (50, 6, 0)], 20).is_err());
        Ok(())
    }
    #[test]
    fn throttled_replies_are_not_admitted_throughput_or_ramp() -> Result<(), String> {
        let rows = measurements(&[(0, 0, 0), (1000, 100, 100), (2000, 200, 200)], 20)?;
        assert_eq!(
            rows.first(),
            Some(&("proxy.steady_requests_per_second", Some(0)))
        );
        assert_eq!(
            rows.last(),
            Some(&("proxy.ms_to_90_percent_of_limit", None))
        );
        Ok(())
    }
    #[test]
    fn boundaries_and_interpolation_are_explicit() -> Result<(), String> {
        assert_eq!(at(&[(0, 0, 0), (2000, 20, 4)], 500)?, (500, 5, 1));
        assert_eq!(
            measurements(&[(0, 0, 0), (30_000, 20, 1)], 20)?.get(1),
            Some(&("proxy.throttle_ppm_after_30s", None))
        );
        assert_eq!(
            measurements(&[(0, 0, 0), (31_000, 0, 0)], 20)?.get(1),
            Some(&("proxy.throttle_ppm_after_30s", None))
        );
        assert!(measurements(&[(0, 0, 0)], 0).is_err());
        assert!(measurements(&[], 20).is_err());
        Ok(())
    }
}
