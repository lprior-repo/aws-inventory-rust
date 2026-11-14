use super::*;

#[tokio::test(start_paused = true)]
pub(super) async fn cancellation_wakes_backoff_before_another_attempt() -> TestResult {
    let executor = ApiExecutor::with_seed(
        ExecutorConfig {
            requests_per_second: 100_000,
            max_backoff_ms: 1_000,
            ..config()
        },
        42,
    )?;
    let failing = executor.execute("backoff", || async {
        Err::<(), _>(ApiError::Transient("retry".into()))
    });
    let cancel = async {
        sleep(Duration::from_millis(1)).await;
        executor.cancel();
    };
    let (result, ()) = tokio::join!(failing, cancel);
    assert_eq!(result, Err(ApiError::Cancelled));
    assert_eq!(executor.stats().attempts, 1);
    assert_eq!(executor.stats().retries, 0);
    assert_eq!(executor.stats().cancelled, 1);
    Ok(())
}
