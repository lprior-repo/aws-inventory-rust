//! Behavior: fractional_and_sub_one_rps_policies_pace_every_concurrent_start.
use super::*;

#[tokio::test(start_paused = true)]
pub(super) async fn fractional_and_sub_one_rps_policies_pace_every_concurrent_start() -> TestResult
{
    for (requests, period_ms, interval_ns) in [
        (1, 2_000, 2_000_000_000),
        (2, 3_000, 1_500_000_000),
        (3, 10_000, 3_333_333_334),
        (1, MAX_RATE_PERIOD_MS, 300_000_000_000),
    ] {
        let executor = ApiExecutor::with_rate_policies(
            config(),
            BTreeMap::from([("fractional".into(), period_policy(requests, period_ms, 4))]),
        )?;
        let starts = Mutex::new(Vec::new());
        let outcomes = stream::iter(0..8)
            .map(|_| executor.execute("fractional", || async { record(&starts) }))
            .buffer_unordered(4)
            .collect::<Vec<_>>()
            .await;
        for outcome in outcomes {
            outcome?;
        }
        let observed = recorded(&starts)?;
        assert_eq!(observed.len(), 8);
        let minimum = Duration::from_nanos(interval_ns);
        spaced(&observed, minimum);
        for (first, second) in observed.iter().zip(observed.iter().skip(1)) {
            assert!(
                second.duration_since(*first) <= minimum.saturating_add(Duration::from_millis(2))
            );
        }
    }
    Ok(())
}
