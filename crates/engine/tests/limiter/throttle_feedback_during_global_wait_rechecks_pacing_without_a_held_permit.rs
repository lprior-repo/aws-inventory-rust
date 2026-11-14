use super::*;

#[tokio::test(start_paused = true)]
pub(super) async fn throttle_feedback_during_global_wait_rechecks_pacing_without_a_held_permit()
-> TestResult {
    let executor = ApiExecutor::with_seed(
        ExecutorConfig {
            global_concurrency: 2,
            per_bucket_concurrency: 2,
            max_attempts: 1,
            ..config()
        },
        42,
    )?;
    let barrier = Barrier::new(3);
    let beginning = Instant::now();
    let starts = Mutex::new(Vec::new());
    let throttled = executor.execute("adaptive-wait", || async {
        barrier.wait().await;
        sleep(Duration::from_millis(150)).await;
        Err::<(), _>(ApiError::Throttled("slow down".into()))
    });
    let blocker = executor.execute("unrelated-blocker", || async {
        barrier.wait().await;
        sleep(Duration::from_secs(1)).await;
        Ok(())
    });
    let waiting = async {
        barrier.wait().await;
        executor
            .execute("adaptive-wait", || async { record(&starts) })
            .await
    };
    let expected_throttle = async {
        require_error(throttled.await, |error| {
            matches!(error, ApiError::Exhausted(_))
        })
    };
    bounded_barrier_case(async { tokio::try_join!(expected_throttle, blocker, waiting) }).await?;
    let observed = recorded(&starts)?;
    assert_eq!(observed.len(), 1);
    assert!(
        observed
            .first()
            .is_some_and(|at| at.duration_since(beginning) >= Duration::from_millis(200))
    );
    assert!(
        observed
            .first()
            .is_some_and(|at| at.duration_since(beginning) < Duration::from_millis(205))
    );
    assert_eq!(executor.stats().in_flight, 0);
    Ok(())
}
