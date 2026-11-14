use super::*;

#[tokio::test(start_paused = true)]
pub(super) async fn fractional_throttle_recovery_never_exceeds_the_original_ceiling() -> TestResult
{
    let executor = ApiExecutor::with_rate_policies(
        config(),
        BTreeMap::from([("fractional".into(), period_policy(1, 2_000, 2))]),
    )?;
    let starts = Mutex::new(Vec::new());
    let attempt = AtomicU32::new(0);
    executor
        .execute("fractional", || async {
            record(&starts)?;
            if attempt.fetch_add(1, Ordering::Relaxed) == 0 {
                Err(ApiError::Throttled("fixture slowdown".into()))
            } else {
                Ok(())
            }
        })
        .await?;
    for _ in 0..4 {
        executor
            .execute("fractional", || async { record(&starts) })
            .await?;
    }
    let observed = recorded(&starts)?;
    for ((first, second), minimum) in observed.iter().zip(observed.iter().skip(1)).zip([
        Duration::from_secs(4),
        Duration::from_secs(2),
        Duration::from_secs(2),
        Duration::from_secs(2),
        Duration::from_secs(2),
    ]) {
        let gap = second.duration_since(*first);
        assert!(gap >= minimum);
        assert!(gap <= minimum.saturating_add(Duration::from_millis(2)));
    }
    assert_eq!(executor.stats().attempts, 6);
    assert_eq!(executor.stats().throttles, 1);
    Ok(())
}
