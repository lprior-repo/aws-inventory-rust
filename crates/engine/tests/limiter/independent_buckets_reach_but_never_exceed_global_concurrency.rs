use super::*;

#[tokio::test(start_paused = true)]
pub(super) async fn independent_buckets_reach_but_never_exceed_global_concurrency() -> TestResult {
    let executor = ApiExecutor::with_seed(
        ExecutorConfig {
            global_concurrency: 3,
            per_bucket_concurrency: 3,
            ..config()
        },
        42,
    )?;
    let results = stream::iter(0..12)
        .map(|index| {
            let executor = &executor;
            async move {
                let bucket = format!("account-{index}/region/ec2");
                executor
                    .execute(&bucket, || async {
                        sleep(Duration::from_millis(100)).await;
                        Ok(())
                    })
                    .await
            }
        })
        .buffer_unordered(12)
        .collect::<Vec<_>>()
        .await;
    for result in results {
        result?;
    }
    let stats = executor.stats();
    assert_eq!(stats.peak_in_flight, 3);
    assert_eq!(stats.in_flight, 0);
    assert_eq!(stats.successes, 12);
    assert_eq!(stats.active_buckets, 12);
    Ok(())
}
