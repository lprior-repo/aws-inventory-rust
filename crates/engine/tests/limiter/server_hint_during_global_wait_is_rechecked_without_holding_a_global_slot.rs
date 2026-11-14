use super::*;

#[tokio::test(start_paused = true)]
pub(super) async fn server_hint_during_global_wait_is_rechecked_without_holding_a_global_slot()
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
    let received = Mutex::new(Vec::new());
    let peers = Mutex::new(Vec::new());
    let probes = Mutex::new(Vec::new());
    let beginning = Instant::now();
    let first = executor.execute("shared", || async {
        barrier.wait().await;
        sleep(Duration::from_millis(150)).await;
        record(&received)?;
        Err::<(), _>(directed(
            RetryKind::Transient,
            RetryAfter::Delay(Duration::from_secs(2)),
        ))
    });
    let blocker = executor.execute("blocker", || async {
        barrier.wait().await;
        sleep(Duration::from_secs(1)).await;
        Ok(())
    });
    let peer = async {
        barrier.wait().await;
        executor
            .execute("shared", || async { record(&peers) })
            .await
    };
    let probe = async {
        sleep(Duration::from_millis(200)).await;
        executor
            .execute("probe", || async { record(&probes) })
            .await
    };

    let expected_first =
        async { require_error(first.await, |error| matches!(error, ApiError::Exhausted(_))) };
    bounded_barrier_case(async { tokio::try_join!(expected_first, blocker, peer, probe) }).await?;
    assert!(
        recorded(&received)?
            .first()
            .zip(recorded(&peers)?.first())
            .is_some_and(|(hint, peer)| peer.duration_since(*hint) >= Duration::from_secs(2))
    );
    assert!(
        recorded(&probes)?
            .first()
            .is_some_and(|probe| probe.duration_since(beginning) < Duration::from_millis(300))
    );
    Ok(())
}
