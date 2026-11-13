//! Behavior: policies_match_exactly_and_preserve_default_rates.
use super::*;

#[tokio::test(start_paused = true)]
pub(super) async fn policies_match_exactly_and_preserve_default_rates() -> TestResult {
    const OVERRIDDEN: &str = "aws:123456789012:us-east-1:lambda:control_read";
    const DEFAULTED: &str = "aws:123456789012:us-east-2:lambda:control_read";
    let executor = ApiExecutor::with_bucket_policies(
        ExecutorConfig {
            requests_per_second: 2,
            ..config()
        },
        BTreeMap::from([(
            OVERRIDDEN.into(),
            BucketLimits {
                requests_per_second: 20,
                concurrency: 2,
            },
        )]),
    )?;
    for (key, interval) in [
        (OVERRIDDEN, Duration::from_millis(50)),
        (DEFAULTED, Duration::from_millis(500)),
    ] {
        let starts = Mutex::new(Vec::new());
        for _ in 0..4 {
            executor.execute(key, || async { record(&starts) }).await?;
        }
        let observed = recorded(&starts)?;
        assert_eq!(observed.len(), 4);
        for (first, second) in observed.iter().zip(observed.iter().skip(1)) {
            let gap = second.duration_since(*first);
            assert!(gap >= interval);
            assert!(gap <= interval.saturating_add(Duration::from_millis(2)));
        }
    }
    assert_eq!(executor.stats().active_buckets, 2);
    Ok(())
}
