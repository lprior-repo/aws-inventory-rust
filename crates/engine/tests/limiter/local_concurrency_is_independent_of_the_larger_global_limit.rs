use super::*;

#[tokio::test(start_paused = true)]
pub(super) async fn local_concurrency_is_independent_of_the_larger_global_limit() -> TestResult {
    let executor = ApiExecutor::with_seed(
        ExecutorConfig {
            per_bucket_concurrency: 2,
            requests_per_second: 1_000,
            ..config()
        },
        42,
    )?;
    let results = stream::iter(0..12)
        .map(|_| {
            executor.execute("one-bucket", || async {
                sleep(Duration::from_millis(100)).await;
                Ok(())
            })
        })
        .buffer_unordered(12)
        .collect::<Vec<_>>()
        .await;
    for result in results {
        result?;
    }
    assert_eq!(executor.stats().peak_in_flight, 2);
    assert_eq!(executor.stats().in_flight, 0);
    Ok(())
}
