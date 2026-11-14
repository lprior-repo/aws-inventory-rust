use super::*;

#[tokio::test(start_paused = true)]
pub(super) async fn dropping_a_call_releases_capacity_and_records_cancellation() -> TestResult {
    let executor = ApiExecutor::with_seed(
        ExecutorConfig {
            global_concurrency: 1,
            per_bucket_concurrency: 1,
            ..config()
        },
        42,
    )?;
    let result = timeout(
        Duration::from_millis(10),
        executor.execute("abandoned", pending::<Result<(), ApiError>>),
    )
    .await;
    assert!(result.is_err());
    assert_eq!(executor.stats().cancelled, 1);
    assert_eq!(executor.stats().in_flight, 0);
    executor.execute("healthy", || async { Ok(()) }).await?;
    assert_eq!(executor.stats().successes, 1);
    Ok(())
}
