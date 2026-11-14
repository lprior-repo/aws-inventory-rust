use super::*;

#[tokio::test(start_paused = true)]
pub(super) async fn retries_consume_the_same_rate_budget_as_initial_calls() -> TestResult {
    let executor = ApiExecutor::with_seed(
        ExecutorConfig {
            requests_per_second: 2,
            ..config()
        },
        42,
    )?;
    let starts = Mutex::new(Vec::new());
    let attempts = AtomicU32::new(0);
    executor
        .execute("account/region/dynamodb", || async {
            record(&starts)?;
            if attempts.fetch_add(1, Ordering::Relaxed) < 2 {
                Err(ApiError::Transient("temporary connection failure".into()))
            } else {
                Ok(())
            }
        })
        .await?;
    let observed = recorded(&starts)?;
    assert_eq!(observed.len(), 3);
    spaced(&observed, Duration::from_millis(500));
    let stats = executor.stats();
    assert_eq!(stats.attempts, 3);
    assert_eq!(stats.retries, 2);
    assert_eq!(stats.successes, 1);
    assert_eq!(stats.failures, 0);
    Ok(())
}
