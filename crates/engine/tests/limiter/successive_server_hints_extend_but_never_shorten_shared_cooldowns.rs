use super::*;

#[tokio::test(start_paused = true)]
pub(super) async fn successive_server_hints_extend_but_never_shorten_shared_cooldowns() -> TestResult
{
    for delays in [(2, 4), (4, 1)] {
        let executor = ApiExecutor::with_seed(
            ExecutorConfig {
                requests_per_second: 1_000,
                max_attempts: 1,
                ..config()
            },
            42,
        )?;
        let first_hint = Mutex::new(Vec::new());
        let second_hint = Mutex::new(Vec::new());
        let peers = Mutex::new(Vec::new());
        let first = executor.execute("shared", || async {
            sleep(Duration::from_millis(100)).await;
            record(&first_hint)?;
            Err::<(), _>(directed(
                RetryKind::Transient,
                RetryAfter::Delay(Duration::from_secs(delays.0)),
            ))
        });
        let second = executor.execute("shared", || async {
            sleep(Duration::from_millis(200)).await;
            record(&second_hint)?;
            Err::<(), _>(directed(
                RetryKind::Transient,
                RetryAfter::Delay(Duration::from_secs(delays.1)),
            ))
        });
        let peer = async {
            sleep(Duration::from_millis(150)).await;
            executor
                .execute("shared", || async { record(&peers) })
                .await
        };
        let (first, second, peer) = tokio::join!(first, second, peer);
        assert!(matches!(first, Err(ApiError::Exhausted(_))));
        assert!(matches!(second, Err(ApiError::Exhausted(_))));
        peer?;
        let peers = recorded(&peers)?;
        for (hints, delay) in [(&first_hint, delays.0), (&second_hint, delays.1)] {
            assert!(
                recorded(hints)?
                    .first()
                    .zip(peers.first())
                    .is_some_and(
                        |(hint, peer)| peer.duration_since(*hint) >= Duration::from_secs(delay)
                    )
            );
        }
        assert_eq!(executor.stats().server_delays, 2);
    }
    Ok(())
}
