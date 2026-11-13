//! Behavior: adaptive_recovery_stops_at_the_bucket_policy_ceiling.
use super::*;

#[tokio::test(start_paused = true)]
pub(super) async fn adaptive_recovery_stops_at_the_bucket_policy_ceiling() -> TestResult {
    let executor = ApiExecutor::with_bucket_policies(
        config(),
        BTreeMap::from([(
            "adaptive-policy".into(),
            BucketLimits {
                requests_per_second: 4,
                concurrency: 2,
            },
        )]),
    )?;
    let starts = Mutex::new(Vec::new());
    let attempts = AtomicU32::new(0);
    executor
        .execute("adaptive-policy", || async {
            record(&starts)?;
            if attempts.fetch_add(1, Ordering::Relaxed) == 0 {
                Err(ApiError::Throttled("slow down".into()))
            } else {
                Ok(())
            }
        })
        .await?;
    for _ in 0..5 {
        executor
            .execute("adaptive-policy", || async { record(&starts) })
            .await?;
    }
    let observed = recorded(&starts)?;
    assert_eq!(observed.len(), 7);
    let expected = [
        Duration::from_millis(500),
        Duration::from_nanos(333_333_334),
        Duration::from_millis(250),
        Duration::from_millis(250),
        Duration::from_millis(250),
        Duration::from_millis(250),
    ];
    for ((first, second), interval) in observed.iter().zip(observed.iter().skip(1)).zip(expected) {
        let gap = second.duration_since(*first);
        assert!(gap >= interval);
        assert!(gap <= interval.saturating_add(Duration::from_millis(2)));
    }
    assert_eq!(executor.stats().throttles, 1);
    assert_eq!(executor.stats().retries, 1);
    Ok(())
}
