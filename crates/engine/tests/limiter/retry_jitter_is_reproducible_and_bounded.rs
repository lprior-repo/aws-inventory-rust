//! Behavior: retry_jitter_is_reproducible_and_bounded.
use super::*;

#[tokio::test(start_paused = true)]
pub(super) async fn retry_jitter_is_reproducible_and_bounded() -> TestResult {
    async fn schedule(seed: u64) -> Result<Vec<Duration>, ApiError> {
        let executor = ApiExecutor::with_seed(
            ExecutorConfig {
                requests_per_second: 100_000,
                max_attempts: 4,
                max_backoff_ms: 250,
                ..config()
            },
            seed,
        )?;
        let starts = Mutex::new(Vec::new());
        let beginning = Instant::now();
        let result: Result<(), ApiError> = executor
            .execute("retry", || async {
                record(&starts)?;
                Err(ApiError::Transient("retry".into()))
            })
            .await;
        assert!(matches!(result, Err(ApiError::Exhausted(_))));
        Ok(recorded(&starts)?
            .into_iter()
            .map(|at| at.duration_since(beginning))
            .collect())
    }
    let first = schedule(42).await?;
    let repeated = schedule(42).await?;
    let different = schedule(43).await?;
    assert_eq!(first, repeated);
    assert_ne!(first, different);
    for (earlier, later) in first.iter().zip(first.iter().skip(1)) {
        assert!(later.saturating_sub(*earlier) <= Duration::from_millis(252));
    }
    Ok(())
}
