use super::*;

#[tokio::test(start_paused = true)]
pub(super) async fn backoff_holds_no_global_permit() -> TestResult {
    let executor = ApiExecutor::with_seed(
        ExecutorConfig {
            global_concurrency: 1,
            per_bucket_concurrency: 1,
            requests_per_second: 100_000,
            max_backoff_ms: 1_000,
            ..config()
        },
        42,
    )?;
    let attempts = AtomicU32::new(0);
    let beginning = Instant::now();
    let healthy = Mutex::new(Vec::new());
    let failing = executor.execute("backoff", || async {
        if attempts.fetch_add(1, Ordering::Relaxed) == 0 {
            Err(ApiError::Transient("retry".into()))
        } else {
            Ok(())
        }
    });
    let unrelated = async {
        sleep(Duration::from_millis(1)).await;
        executor
            .execute("healthy", || async { record(&healthy) })
            .await
    };
    let (first, second) = tokio::join!(failing, unrelated);
    first?;
    second?;
    let observed = recorded(&healthy)?;
    assert!(
        observed
            .first()
            .is_some_and(|at| at.duration_since(beginning) < Duration::from_millis(5))
    );
    assert_eq!(executor.stats().retries, 1);
    Ok(())
}
