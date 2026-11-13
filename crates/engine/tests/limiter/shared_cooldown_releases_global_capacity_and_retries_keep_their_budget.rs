//! Behavior: shared_cooldown_releases_global_capacity_and_retries_keep_their_budget.
use super::*;

#[tokio::test(start_paused = true)]
pub(super) async fn shared_cooldown_releases_global_capacity_and_retries_keep_their_budget()
-> TestResult {
    let executor = ApiExecutor::with_seed(
        ExecutorConfig {
            global_concurrency: 1,
            per_bucket_concurrency: 1,
            ..config()
        },
        42,
    )?;
    let delayed = Mutex::new(Vec::new());
    let unrelated = Mutex::new(Vec::new());
    let attempts = AtomicU32::new(0);
    let beginning = Instant::now();
    let first = executor.execute("delayed", || async {
        record(&delayed)?;
        if attempts.fetch_add(1, Ordering::Relaxed) == 0 {
            Err(directed(
                RetryKind::Transient,
                RetryAfter::Delay(Duration::from_secs(2)),
            ))
        } else {
            Ok(())
        }
    });
    let other = async {
        sleep(Duration::from_millis(20)).await;
        executor
            .execute("other", || async { record(&unrelated) })
            .await
    };
    let (first, other) = tokio::join!(first, other);
    first?;
    other?;
    spaced(&recorded(&delayed)?, Duration::from_secs(2));
    assert!(
        recorded(&unrelated)?
            .first()
            .is_some_and(|at| at.duration_since(beginning) < Duration::from_secs(1))
    );
    assert_eq!(executor.stats().attempts, 3);
    assert_eq!(executor.stats().retries, 1);
    assert_eq!(executor.stats().peak_in_flight, 1);
    Ok(())
}
