//! Behavior: policies_can_raise_or_lower_local_concurrency_within_global_capacity.
use super::*;

#[tokio::test(start_paused = true)]
pub(super) async fn policies_can_raise_or_lower_local_concurrency_within_global_capacity()
-> TestResult {
    for (default_concurrency, policy_concurrency) in [(1, 3), (4, 2)] {
        let executor = ApiExecutor::with_bucket_policies(
            ExecutorConfig {
                per_bucket_concurrency: default_concurrency,
                ..config()
            },
            BTreeMap::from([(
                "bounded".into(),
                BucketLimits {
                    requests_per_second: 1_000,
                    concurrency: policy_concurrency,
                },
            )]),
        )?;
        let results = stream::iter(0..6)
            .map(|_| {
                executor.execute("bounded", || async {
                    sleep(Duration::from_millis(100)).await;
                    Ok(())
                })
            })
            .buffer_unordered(6)
            .collect::<Vec<_>>()
            .await;
        for result in results {
            result?;
        }
        assert_eq!(
            executor.stats().peak_in_flight,
            u64::try_from(policy_concurrency)?,
        );
        assert_eq!(executor.stats().in_flight, 0);
    }
    Ok(())
}
