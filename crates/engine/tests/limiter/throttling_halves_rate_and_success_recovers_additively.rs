//! Behavior: throttling_halves_rate_and_success_recovers_additively.
use super::*;

#[tokio::test(start_paused = true)]
pub(super) async fn throttling_halves_rate_and_success_recovers_additively() -> TestResult {
    let executor = ApiExecutor::with_seed(
        ExecutorConfig {
            requests_per_second: 4,
            ..config()
        },
        42,
    )?;
    let starts = Mutex::new(Vec::new());
    let attempts = AtomicU32::new(0);
    executor
        .execute("adaptive", || async {
            record(&starts)?;
            if attempts.fetch_add(1, Ordering::Relaxed) == 0 {
                Err(ApiError::Throttled("slow down".into()))
            } else {
                Ok(())
            }
        })
        .await?;
    executor
        .execute("adaptive", || async { record(&starts) })
        .await?;
    executor
        .execute("adaptive", || async { record(&starts) })
        .await?;
    let observed = recorded(&starts)?;
    let gaps = observed
        .iter()
        .zip(observed.iter().skip(1))
        .map(|(first, second)| second.duration_since(*first))
        .collect::<Vec<_>>();
    assert_eq!(gaps.len(), 3);
    for (actual, minimum) in gaps.iter().zip([
        Duration::from_millis(500),
        Duration::from_nanos(333_333_334),
        Duration::from_millis(250),
    ]) {
        assert!(*actual >= minimum);
        assert!(*actual <= minimum.saturating_add(Duration::from_millis(2)));
    }
    assert_eq!(executor.stats().throttles, 1);
    Ok(())
}
