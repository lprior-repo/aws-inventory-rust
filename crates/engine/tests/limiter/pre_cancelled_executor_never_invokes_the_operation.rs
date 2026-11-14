use super::*;

#[tokio::test(start_paused = true)]
pub(super) async fn pre_cancelled_executor_never_invokes_the_operation() -> TestResult {
    let executor = ApiExecutor::with_seed(config(), 42)?;
    let calls = AtomicU32::new(0);
    executor.cancel();
    let result = executor
        .execute("cancelled", || async {
            calls.fetch_add(1, Ordering::Relaxed);
            Ok(())
        })
        .await;
    assert_eq!(result, Err(ApiError::Cancelled));
    assert_eq!(calls.load(Ordering::Relaxed), 0);
    assert_eq!(executor.stats().attempts, 0);
    assert_eq!(executor.stats().cancelled, 1);
    Ok(())
}
