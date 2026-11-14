use super::*;

#[tokio::test(start_paused = true)]
pub(super) async fn final_attempt_retry_hint_delays_queued_peers() -> TestResult {
    let executor = ApiExecutor::with_seed(
        ExecutorConfig {
            max_attempts: 1,
            ..config()
        },
        42,
    )?;
    let received = Mutex::new(Vec::new());
    let peers = Mutex::new(Vec::new());
    let first = executor.execute("shared", || async {
        sleep(Duration::from_millis(50)).await;
        record(&received)?;
        Err::<(), _>(directed(
            RetryKind::Throttled,
            RetryAfter::Delay(Duration::from_secs(3)),
        ))
    });
    let peer = executor.execute("shared", || async { record(&peers) });
    let (first, peer) = tokio::join!(first, peer);
    assert!(
        matches!(first, Err(ApiError::Exhausted(message)) if message.contains("fixture server delay"))
    );
    peer?;
    let received = recorded(&received)?;
    let peers = recorded(&peers)?;
    assert_eq!(received.len(), 1);
    assert_eq!(peers.len(), 1);
    assert!(
        received
            .first()
            .zip(peers.first())
            .is_some_and(|(hint, peer)| peer.duration_since(*hint) >= Duration::from_secs(3))
    );
    assert_eq!(executor.stats().server_delays, 1);
    assert_eq!(executor.stats().retries, 0);
    Ok(())
}
