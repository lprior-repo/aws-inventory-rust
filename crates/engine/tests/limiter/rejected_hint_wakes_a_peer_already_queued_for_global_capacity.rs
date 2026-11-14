use super::*;

#[tokio::test(start_paused = true)]
pub(super) async fn rejected_hint_wakes_a_peer_already_queued_for_global_capacity() -> TestResult {
    let executor = ApiExecutor::with_seed(
        ExecutorConfig {
            global_concurrency: 2,
            per_bucket_concurrency: 2,
            ..config()
        },
        42,
    )?;
    let barrier = Barrier::new(3);
    let beginning = Instant::now();
    let peer_finished = Mutex::new(Vec::new());
    let peer_calls = AtomicU32::new(0);
    let first = executor.execute("shared", || async {
        barrier.wait().await;
        sleep(Duration::from_millis(150)).await;
        Err::<(), _>(directed(RetryKind::Throttled, RetryAfter::Excessive))
    });
    let blocker = executor.execute("other", || async {
        barrier.wait().await;
        sleep(Duration::from_secs(1)).await;
        Ok(())
    });
    let peer = async {
        barrier.wait().await;
        let outcome = executor
            .execute("shared", || async {
                peer_calls.fetch_add(1, Ordering::Relaxed);
                Ok(())
            })
            .await;
        record(&peer_finished)?;
        outcome
    };
    let expected_first = async {
        require_error(first.await, |error| {
            matches!(error, ApiError::RetryAfterRejected(_))
        })
    };
    let expected_peer = async {
        require_error(peer.await, |error| {
            matches!(error, ApiError::RetryAfterRejected(_))
        })
    };
    bounded_barrier_case(async { tokio::try_join!(expected_first, blocker, expected_peer) })
        .await?;
    assert_eq!(peer_calls.load(Ordering::Relaxed), 0);
    assert!(
        recorded(&peer_finished)?
            .first()
            .is_some_and(|at| at.duration_since(beginning) < Duration::from_millis(200))
    );
    assert_eq!(executor.stats().in_flight, 0);
    Ok(())
}
