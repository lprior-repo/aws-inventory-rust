use std::collections::BTreeMap;
use std::error::Error;
use std::future::{Future, pending};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use futures_util::{StreamExt, TryStreamExt, stream};
use inventory_engine::limiter::{
    ApiError, ApiExecutor, BucketLimits, BucketPolicy, ExecutorConfig, MAX_BUCKETS,
    MAX_RATE_PERIOD_MS, MAX_RETRY_AFTER_MS, PeriodLimits, RetryAfter, RetryKind,
};
use tokio::sync::Barrier;
use tokio::time::{Instant, sleep, timeout};

type TestResult = Result<(), Box<dyn Error>>;

async fn bounded_barrier_case<T>(
    future: impl Future<Output = Result<T, ApiError>>,
) -> Result<T, Box<dyn Error>> {
    timeout(Duration::from_secs(30), future)
        .await
        .map_err(|_| "engine limiter barrier test exceeded its 30-second virtual deadline")?
        .map_err(Into::into)
}

fn require_error(
    result: Result<(), ApiError>,
    expected: fn(&ApiError) -> bool,
) -> Result<(), ApiError> {
    match result {
        Err(error) if expected(&error) => Ok(()),
        Err(error) => Err(error),
        Ok(()) => Err(ApiError::Permanent(
            "expected the exercised API failure".into(),
        )),
    }
}

fn config() -> ExecutorConfig {
    ExecutorConfig {
        global_concurrency: 4,
        per_bucket_concurrency: 4,
        requests_per_second: 10,
        max_attempts: 3,
        attempt_timeout_ms: 5_000,
        max_backoff_ms: 1,
    }
}

fn record(starts: &Mutex<Vec<Instant>>) -> Result<(), ApiError> {
    starts
        .lock()
        .map_err(|_| ApiError::Permanent("test start recorder poisoned".into()))?
        .push(Instant::now());
    Ok(())
}

fn recorded(starts: &Mutex<Vec<Instant>>) -> Result<Vec<Instant>, ApiError> {
    Ok(starts
        .lock()
        .map_err(|_| ApiError::Permanent("test start recorder poisoned".into()))?
        .clone())
}

fn spaced(starts: &[Instant], minimum: Duration) {
    for (first, second) in starts.iter().zip(starts.iter().skip(1)) {
        assert!(second.duration_since(*first) >= minimum);
    }
}

fn period_policy(requests: u32, period_ms: u64, concurrency: usize) -> BucketPolicy {
    BucketPolicy::PerPeriod(PeriodLimits {
        requests,
        period_ms,
        concurrency,
    })
}

fn directed(kind: RetryKind, retry_after: RetryAfter) -> ApiError {
    ApiError::RetryAfter {
        kind,
        message: "fixture server delay".into(),
        retry_after,
    }
}

#[test]
fn rational_policy_json_preserves_integer_shape_and_rejects_mixed_or_unknown_fields() -> TestResult
{
    let integer =
        serde_json::from_str::<BucketPolicy>(r#"{"requests_per_second":5,"concurrency":2}"#)?;
    assert_eq!(
        integer,
        BucketPolicy::PerSecond(BucketLimits {
            requests_per_second: 5,
            concurrency: 2
        })
    );
    let fractional =
        serde_json::from_str::<BucketPolicy>(r#"{"requests":1,"period_ms":2000,"concurrency":2}"#)?;
    assert_eq!(fractional, period_policy(1, 2_000, 2));
    for text in [
        r#"{"requests_per_second":5,"requests":1,"period_ms":2000,"concurrency":2}"#,
        r#"{"requests":1,"period_ms":2000,"concurrency":2,"period_seconds":2}"#,
        r#"{"requests_per_second":5,"concurrency":2,"concurency":2}"#,
        r#"{"requests_per_second":0.5,"concurrency":2}"#,
        r#"{"requests":1.5,"period_ms":2000,"concurrency":2}"#,
    ] {
        assert!(serde_json::from_str::<BucketPolicy>(text).is_err());
    }
    Ok(())
}

#[test]
fn rational_policy_validates_fractional_floor_ceiling_and_payload_bounds() {
    for policy in [
        period_policy(0, 1_000, 1),
        period_policy(100_001, 1_000, 1),
        period_policy(1, 0, 1),
        period_policy(1, MAX_RATE_PERIOD_MS.saturating_add(1), 1),
        period_policy(101, 1, 1),
        period_policy(1, 1_000, 0),
        period_policy(1, 1_000, 5),
        period_policy(u32::MAX, u64::MAX, usize::MAX),
    ] {
        assert!(matches!(
            ApiExecutor::with_rate_policies(
                config(),
                BTreeMap::from([("bounded".into(), policy),])
            ),
            Err(ApiError::InvalidConfig(_))
        ));
    }
    for policy in [
        period_policy(1, MAX_RATE_PERIOD_MS, 1),
        period_policy(100, 1, 1),
    ] {
        assert!(
            ApiExecutor::with_rate_policies(
                config(),
                BTreeMap::from([("bounded".into(), policy),])
            )
            .is_ok()
        );
    }
}

#[tokio::test(start_paused = true)]
async fn fractional_and_sub_one_rps_policies_pace_every_concurrent_start() -> TestResult {
    for (requests, period_ms, interval_ns) in [
        (1, 2_000, 2_000_000_000),
        (2, 3_000, 1_500_000_000),
        (3, 10_000, 3_333_333_334),
        (1, MAX_RATE_PERIOD_MS, 300_000_000_000),
    ] {
        let executor = ApiExecutor::with_rate_policies(
            config(),
            BTreeMap::from([("fractional".into(), period_policy(requests, period_ms, 4))]),
        )?;
        let starts = Mutex::new(Vec::new());
        let outcomes = stream::iter(0..8)
            .map(|_| executor.execute("fractional", || async { record(&starts) }))
            .buffer_unordered(4)
            .collect::<Vec<_>>()
            .await;
        for outcome in outcomes {
            outcome?;
        }
        let observed = recorded(&starts)?;
        assert_eq!(observed.len(), 8);
        let minimum = Duration::from_nanos(interval_ns);
        spaced(&observed, minimum);
        for (first, second) in observed.iter().zip(observed.iter().skip(1)) {
            assert!(
                second.duration_since(*first) <= minimum.saturating_add(Duration::from_millis(2))
            );
        }
    }
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn fractional_throttle_recovery_never_exceeds_the_original_ceiling() -> TestResult {
    let executor = ApiExecutor::with_rate_policies(
        config(),
        BTreeMap::from([("fractional".into(), period_policy(1, 2_000, 2))]),
    )?;
    let starts = Mutex::new(Vec::new());
    let attempt = AtomicU32::new(0);
    executor
        .execute("fractional", || async {
            record(&starts)?;
            if attempt.fetch_add(1, Ordering::Relaxed) == 0 {
                Err(ApiError::Throttled("fixture slowdown".into()))
            } else {
                Ok(())
            }
        })
        .await?;
    for _ in 0..4 {
        executor
            .execute("fractional", || async { record(&starts) })
            .await?;
    }
    let observed = recorded(&starts)?;
    for ((first, second), minimum) in observed.iter().zip(observed.iter().skip(1)).zip([
        Duration::from_secs(4),
        Duration::from_secs(2),
        Duration::from_secs(2),
        Duration::from_secs(2),
        Duration::from_secs(2),
    ]) {
        let gap = second.duration_since(*first);
        assert!(gap >= minimum);
        assert!(gap <= minimum.saturating_add(Duration::from_millis(2)));
    }
    assert_eq!(executor.stats().attempts, 6);
    assert_eq!(executor.stats().throttles, 1);
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn final_attempt_retry_hint_delays_queued_peers() -> TestResult {
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

#[tokio::test(start_paused = true)]
async fn successive_server_hints_extend_but_never_shorten_shared_cooldowns() -> TestResult {
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

#[tokio::test(start_paused = true)]
async fn shared_cooldown_releases_global_capacity_and_retries_keep_their_budget() -> TestResult {
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

#[tokio::test(start_paused = true)]
async fn server_hint_during_global_wait_is_rechecked_without_holding_a_global_slot() -> TestResult {
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
    // If a request fails before entering its operation, its barrier peers must
    // be dropped promptly. An unexpected terminal result is a test failure,
    // not a reason to strand the remaining futures at the barrier.
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

#[tokio::test(start_paused = true)]
async fn cancelling_a_server_cooldown_wakes_all_bucket_waiters() -> TestResult {
    let executor = ApiExecutor::with_seed(
        ExecutorConfig {
            global_concurrency: 1,
            per_bucket_concurrency: 1,
            ..config()
        },
        42,
    )?;
    let beginning = Instant::now();
    let first = executor.execute("shared", || async {
        Err::<(), _>(directed(
            RetryKind::Transient,
            RetryAfter::Delay(Duration::from_millis(MAX_RETRY_AFTER_MS)),
        ))
    });
    let peer = executor.execute("shared", || async { Ok(()) });
    let cancel = async {
        sleep(Duration::from_millis(50)).await;
        executor.cancel();
    };
    let (first, peer, ()) = tokio::join!(first, peer, cancel);
    assert_eq!(first, Err(ApiError::Cancelled));
    assert_eq!(peer, Err(ApiError::Cancelled));
    assert!(Instant::now().duration_since(beginning) < Duration::from_millis(100));
    assert_eq!(executor.stats().attempts, 1);
    assert_eq!(executor.stats().in_flight, 0);
    assert_eq!(executor.stats().cancelled, 2);
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn invalid_and_excessive_hints_close_only_the_affected_bucket() -> TestResult {
    for hint in [
        RetryAfter::Invalid,
        RetryAfter::Excessive,
        RetryAfter::Delay(Duration::from_millis(MAX_RETRY_AFTER_MS.saturating_add(1))),
        RetryAfter::Delay(Duration::MAX),
    ] {
        let executor = ApiExecutor::with_seed(config(), 42)?;
        let calls = AtomicU32::new(0);
        let first = executor
            .execute("closed", || async {
                calls.fetch_add(1, Ordering::Relaxed);
                Err::<(), _>(directed(RetryKind::Throttled, hint))
            })
            .await;
        assert!(
            matches!(first, Err(ApiError::RetryAfterRejected(message)) if message.contains("fixture server delay"))
        );
        let peer = executor
            .execute("closed", || async {
                calls.fetch_add(1, Ordering::Relaxed);
                Ok(())
            })
            .await;
        assert!(matches!(peer, Err(ApiError::RetryAfterRejected(_))));
        executor.execute("other", || async { Ok(()) }).await?;
        assert_eq!(calls.load(Ordering::Relaxed), 1);
        assert_eq!(executor.stats().attempts, 2);
        assert_eq!(executor.stats().throttles, 1);
        assert_eq!(executor.stats().rejected_retry_hints, 1);
        assert_eq!(executor.stats().server_delays, 0);
    }
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn rejected_hint_wakes_existing_cooldown_and_local_capacity_waiters() -> TestResult {
    let executor = ApiExecutor::with_seed(
        ExecutorConfig {
            global_concurrency: 3,
            per_bucket_concurrency: 3,
            requests_per_second: 1_000,
            ..config()
        },
        42,
    )?;
    let beginning = Instant::now();
    let first = executor.execute("shared", || async {
        sleep(Duration::from_millis(100)).await;
        Err::<(), _>(directed(
            RetryKind::Transient,
            RetryAfter::Delay(Duration::from_millis(MAX_RETRY_AFTER_MS)),
        ))
    });
    let second = executor.execute("shared", || async {
        sleep(Duration::from_millis(200)).await;
        Err::<(), _>(directed(RetryKind::Throttled, RetryAfter::Invalid))
    });
    let peer = async {
        sleep(Duration::from_millis(150)).await;
        executor.execute("shared", || async { Ok(()) }).await
    };
    let local = async {
        sleep(Duration::from_millis(160)).await;
        executor.execute("shared", || async { Ok(()) }).await
    };
    let outcomes = timeout(Duration::from_millis(300), async {
        tokio::join!(first, second, peer, local)
    })
    .await?;
    for outcome in [outcomes.0, outcomes.1, outcomes.2, outcomes.3] {
        assert!(matches!(outcome, Err(ApiError::RetryAfterRejected(_))));
    }
    assert!(Instant::now().duration_since(beginning) < Duration::from_millis(300));
    assert_eq!(executor.stats().attempts, 2);
    assert_eq!(executor.stats().in_flight, 0);
    assert_eq!(executor.stats().server_delays, 1);
    assert_eq!(executor.stats().rejected_retry_hints, 1);
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn rejected_hint_wakes_a_peer_already_queued_for_global_capacity() -> TestResult {
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

#[tokio::test(start_paused = true)]
async fn rejected_hint_cancels_already_active_same_bucket_operations() -> TestResult {
    let executor = ApiExecutor::with_seed(config(), 42)?;
    let first = executor.execute("shared", || async {
        sleep(Duration::from_millis(200)).await;
        Err::<(), _>(directed(RetryKind::Transient, RetryAfter::Invalid))
    });
    let peer = executor.execute("shared", pending::<Result<(), ApiError>>);
    let (first, peer) = timeout(Duration::from_millis(300), async {
        tokio::join!(first, peer)
    })
    .await?;
    assert!(matches!(first, Err(ApiError::RetryAfterRejected(_))));
    assert!(matches!(peer, Err(ApiError::RetryAfterRejected(_))));
    assert_eq!(executor.stats().attempts, 2);
    assert_eq!(executor.stats().timeouts, 0);
    assert_eq!(executor.stats().in_flight, 0);
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn accepted_hint_boundary_and_retry_exhaustion_remain_bounded() -> TestResult {
    let executor = ApiExecutor::with_seed(config(), 42)?;
    let starts = Mutex::new(Vec::new());
    let result = executor
        .execute("bounded", || async {
            record(&starts)?;
            Err::<(), _>(directed(
                RetryKind::Timeout,
                RetryAfter::Delay(Duration::from_millis(MAX_RETRY_AFTER_MS)),
            ))
        })
        .await;
    assert!(
        matches!(result, Err(ApiError::Exhausted(message)) if message.contains("timeout") && message.contains("fixture server delay"))
    );
    assert_eq!(recorded(&starts)?.len(), 3);
    spaced(
        &recorded(&starts)?,
        Duration::from_millis(MAX_RETRY_AFTER_MS),
    );
    assert_eq!(executor.stats().attempts, 3);
    assert_eq!(executor.stats().retries, 2);
    assert_eq!(executor.stats().timeouts, 3);
    assert_eq!(executor.stats().server_delays, 3);
    Ok(())
}

#[test]
fn validates_every_configuration_bound() {
    let normal = config();
    let invalid = [
        ExecutorConfig {
            global_concurrency: 0,
            ..normal
        },
        ExecutorConfig {
            global_concurrency: 4_097,
            ..normal
        },
        ExecutorConfig {
            per_bucket_concurrency: 0,
            ..normal
        },
        ExecutorConfig {
            per_bucket_concurrency: 5,
            ..normal
        },
        ExecutorConfig {
            requests_per_second: 0,
            ..normal
        },
        ExecutorConfig {
            requests_per_second: 100_001,
            ..normal
        },
        ExecutorConfig {
            max_attempts: 0,
            ..normal
        },
        ExecutorConfig {
            max_attempts: 11,
            ..normal
        },
        ExecutorConfig {
            attempt_timeout_ms: 0,
            ..normal
        },
        ExecutorConfig {
            attempt_timeout_ms: 300_001,
            ..normal
        },
        ExecutorConfig {
            max_backoff_ms: 0,
            ..normal
        },
        ExecutorConfig {
            max_backoff_ms: 300_001,
            ..normal
        },
    ];
    for value in invalid {
        assert!(matches!(
            ApiExecutor::new(value),
            Err(ApiError::InvalidConfig(_))
        ));
    }
    assert!(ApiExecutor::new(ExecutorConfig::default()).is_ok());
}

#[test]
fn bucket_policies_validate_limits_and_exact_key_bounds() {
    for limits in [
        BucketLimits {
            requests_per_second: 0,
            concurrency: 1,
        },
        BucketLimits {
            requests_per_second: 100_001,
            concurrency: 1,
        },
        BucketLimits {
            requests_per_second: 1,
            concurrency: 0,
        },
        BucketLimits {
            requests_per_second: 1,
            concurrency: 5,
        },
    ] {
        assert!(matches!(
            ApiExecutor::with_bucket_policies(
                config(),
                BTreeMap::from([("account/region/service".into(), limits)]),
            ),
            Err(ApiError::InvalidConfig(_))
        ));
    }
    let limits = BucketLimits {
        requests_per_second: 1,
        concurrency: 1,
    };
    for key in [String::new(), "x".repeat(513)] {
        assert!(matches!(
            ApiExecutor::with_bucket_policies(config(), BTreeMap::from([(key, limits)])),
            Err(ApiError::InvalidConfig(_))
        ));
    }
    assert!(ApiExecutor::with_bucket_policies(config(), BTreeMap::new()).is_ok());
}

#[test]
fn bucket_policy_map_has_a_hard_entry_bound() {
    let policies = (0..=MAX_BUCKETS)
        .map(|index| {
            (
                format!("bucket-{index}"),
                BucketLimits {
                    requests_per_second: 1,
                    concurrency: 1,
                },
            )
        })
        .collect();
    assert!(matches!(
        ApiExecutor::with_bucket_policies(config(), policies),
        Err(ApiError::InvalidConfig(_))
    ));
}

#[test]
fn rational_policy_maps_accept_the_boundary_and_never_truncate_overflow() {
    let at_limit = (0..MAX_BUCKETS)
        .map(|index| (format!("period-{index}"), period_policy(1, 1_000, 1)))
        .collect();
    assert!(ApiExecutor::with_rate_policies(config(), at_limit).is_ok());

    let too_many = (0..=MAX_BUCKETS)
        .map(|index| (format!("period-{index}"), period_policy(1, 1_000, 1)))
        .collect();
    assert!(matches!(
        ApiExecutor::with_rate_policies(config(), too_many),
        Err(ApiError::InvalidConfig(_))
    ));
}

#[test]
fn bucket_policy_json_rejects_unknown_fields() {
    let invalid = r#"{"requests_per_second":10,"concurrency":2,"concurency":3}"#;
    assert!(serde_json::from_str::<BucketLimits>(invalid).is_err());
}

#[tokio::test(start_paused = true)]
async fn policies_match_exactly_and_preserve_default_rates() -> TestResult {
    const OVERRIDDEN: &str = "aws:123456789012:us-east-1:lambda:control_read";
    const DEFAULTED: &str = "aws:123456789012:us-east-2:lambda:control_read";
    let executor = ApiExecutor::with_bucket_policies(
        ExecutorConfig {
            requests_per_second: 2,
            ..config()
        },
        BTreeMap::from([(
            OVERRIDDEN.into(),
            BucketLimits {
                requests_per_second: 20,
                concurrency: 2,
            },
        )]),
    )?;
    for (key, interval) in [
        (OVERRIDDEN, Duration::from_millis(50)),
        (DEFAULTED, Duration::from_millis(500)),
    ] {
        let starts = Mutex::new(Vec::new());
        for _ in 0..4 {
            executor.execute(key, || async { record(&starts) }).await?;
        }
        let observed = recorded(&starts)?;
        assert_eq!(observed.len(), 4);
        for (first, second) in observed.iter().zip(observed.iter().skip(1)) {
            let gap = second.duration_since(*first);
            assert!(gap >= interval);
            assert!(gap <= interval.saturating_add(Duration::from_millis(2)));
        }
    }
    assert_eq!(executor.stats().active_buckets, 2);
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn policies_can_raise_or_lower_local_concurrency_within_global_capacity() -> TestResult {
    for (default_concurrency, policy_concurrency) in [(1, 3), (4, 2)] {
        let executor = ApiExecutor::with_bucket_policies(
            ExecutorConfig {
                per_bucket_concurrency: default_concurrency,
                ..config()
            },
            BTreeMap::from([(
                "bounded".into(),
                BucketLimits {
                    requests_per_second: 1_000,
                    concurrency: policy_concurrency,
                },
            )]),
        )?;
        let results = stream::iter(0..6)
            .map(|_| {
                executor.execute("bounded", || async {
                    sleep(Duration::from_millis(100)).await;
                    Ok(())
                })
            })
            .buffer_unordered(6)
            .collect::<Vec<_>>()
            .await;
        for result in results {
            result?;
        }
        assert_eq!(
            executor.stats().peak_in_flight,
            u64::try_from(policy_concurrency)?,
        );
        assert_eq!(executor.stats().in_flight, 0);
    }
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn adaptive_recovery_stops_at_the_bucket_policy_ceiling() -> TestResult {
    let executor = ApiExecutor::with_bucket_policies(
        config(),
        BTreeMap::from([(
            "adaptive-policy".into(),
            BucketLimits {
                requests_per_second: 4,
                concurrency: 2,
            },
        )]),
    )?;
    let starts = Mutex::new(Vec::new());
    let attempts = AtomicU32::new(0);
    executor
        .execute("adaptive-policy", || async {
            record(&starts)?;
            if attempts.fetch_add(1, Ordering::Relaxed) == 0 {
                Err(ApiError::Throttled("slow down".into()))
            } else {
                Ok(())
            }
        })
        .await?;
    for _ in 0..5 {
        executor
            .execute("adaptive-policy", || async { record(&starts) })
            .await?;
    }
    let observed = recorded(&starts)?;
    assert_eq!(observed.len(), 7);
    let expected = [
        Duration::from_millis(500),
        Duration::from_nanos(333_333_334),
        Duration::from_millis(250),
        Duration::from_millis(250),
        Duration::from_millis(250),
        Duration::from_millis(250),
    ];
    for ((first, second), interval) in observed.iter().zip(observed.iter().skip(1)).zip(expected) {
        let gap = second.duration_since(*first);
        assert!(gap >= interval);
        assert!(gap <= interval.saturating_add(Duration::from_millis(2)));
    }
    assert_eq!(executor.stats().throttles, 1);
    assert_eq!(executor.stats().retries, 1);
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn cancellation_is_observable_before_and_after_the_signal() -> TestResult {
    let executor = ApiExecutor::new(config())?;
    assert!(!executor.is_cancelled());
    let beginning = Instant::now();
    let observer = async {
        executor.cancelled().await;
        Instant::now()
    };
    let trigger = async {
        sleep(Duration::from_millis(10)).await;
        executor.cancel();
    };
    let (observed, ()) = tokio::join!(observer, trigger);
    assert!(observed.duration_since(beginning) >= Duration::from_millis(10));
    assert!(observed.duration_since(beginning) <= Duration::from_millis(12));
    assert!(executor.is_cancelled());
    let signalled = Instant::now();
    timeout(Duration::from_millis(1), executor.cancelled()).await?;
    assert_eq!(Instant::now(), signalled);
    assert_eq!(executor.stats().attempts, 0);
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn every_concurrent_start_obeys_bucket_rate() -> TestResult {
    let executor = ApiExecutor::with_seed(config(), 42)?;
    let starts = Mutex::new(Vec::new());
    let results = stream::iter(0..6)
        .map(|_| executor.execute("account/region/ec2", || async { record(&starts) }))
        .buffer_unordered(6)
        .collect::<Vec<_>>()
        .await;
    for result in results {
        result?;
    }
    let observed = recorded(&starts)?;
    assert_eq!(observed.len(), 6);
    spaced(&observed, Duration::from_millis(100));
    assert_eq!(executor.stats().attempts, 6);
    assert_eq!(executor.stats().successes, 6);
    assert_eq!(executor.stats().in_flight, 0);
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn fractional_rates_round_up_instead_of_exceeding_budget() -> TestResult {
    let executor = ApiExecutor::with_seed(
        ExecutorConfig {
            requests_per_second: 3,
            ..config()
        },
        42,
    )?;
    let starts = Mutex::new(Vec::new());
    for _ in 0..4 {
        executor
            .execute("account/region/lambda", || async { record(&starts) })
            .await?;
    }
    spaced(&recorded(&starts)?, Duration::from_nanos(333_333_334));
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn retries_consume_the_same_rate_budget_as_initial_calls() -> TestResult {
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

#[tokio::test(start_paused = true)]
async fn global_contention_cannot_release_accumulated_rate_tickets_as_a_burst() -> TestResult {
    let executor = ApiExecutor::with_seed(
        ExecutorConfig {
            global_concurrency: 2,
            per_bucket_concurrency: 2,
            ..config()
        },
        42,
    )?;
    let barrier = Barrier::new(3);
    let starts = Mutex::new(Vec::new());
    let blocker_a = executor.execute("a/region/s3", || async {
        barrier.wait().await;
        sleep(Duration::from_secs(1)).await;
        Ok(())
    });
    let blocker_b = executor.execute("b/region/s3", || async {
        barrier.wait().await;
        sleep(Duration::from_secs(1)).await;
        Ok(())
    });
    let waiting = async {
        barrier.wait().await;
        stream::iter(0..4)
            .map(|_| executor.execute("c/region/ec2", || async { record(&starts) }))
            .buffer_unordered(4)
            .try_collect::<Vec<_>>()
            .await
    };
    bounded_barrier_case(async { tokio::try_join!(blocker_a, blocker_b, waiting) }).await?;
    let observed = recorded(&starts)?;
    assert_eq!(observed.len(), 4);
    spaced(&observed, Duration::from_millis(100));
    assert_eq!(executor.stats().peak_in_flight, 2);
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn pacing_one_bucket_does_not_occupy_global_capacity() -> TestResult {
    let executor = ApiExecutor::with_seed(
        ExecutorConfig {
            global_concurrency: 1,
            per_bucket_concurrency: 1,
            requests_per_second: 1,
            ..config()
        },
        42,
    )?;
    executor.execute("slow", || async { Ok(()) }).await?;
    let beginning = Instant::now();
    let healthy = Mutex::new(Vec::new());
    let paced = executor.execute("slow", || async { Ok(()) });
    let unrelated = async {
        sleep(Duration::from_millis(10)).await;
        executor
            .execute("healthy", || async { record(&healthy) })
            .await
    };
    let (paced_result, healthy_result) = tokio::join!(paced, unrelated);
    paced_result?;
    healthy_result?;
    let observed = recorded(&healthy)?;
    assert_eq!(observed.len(), 1);
    assert!(
        observed
            .first()
            .is_some_and(|at| at.duration_since(beginning) < Duration::from_millis(20))
    );
    assert!(Instant::now().duration_since(beginning) >= Duration::from_secs(1));
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn independent_buckets_reach_but_never_exceed_global_concurrency() -> TestResult {
    let executor = ApiExecutor::with_seed(
        ExecutorConfig {
            global_concurrency: 3,
            per_bucket_concurrency: 3,
            ..config()
        },
        42,
    )?;
    let results = stream::iter(0..12)
        .map(|index| {
            let executor = &executor;
            async move {
                let bucket = format!("account-{index}/region/ec2");
                executor
                    .execute(&bucket, || async {
                        sleep(Duration::from_millis(100)).await;
                        Ok(())
                    })
                    .await
            }
        })
        .buffer_unordered(12)
        .collect::<Vec<_>>()
        .await;
    for result in results {
        result?;
    }
    let stats = executor.stats();
    assert_eq!(stats.peak_in_flight, 3);
    assert_eq!(stats.in_flight, 0);
    assert_eq!(stats.successes, 12);
    assert_eq!(stats.active_buckets, 12);
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn local_concurrency_is_independent_of_the_larger_global_limit() -> TestResult {
    let executor = ApiExecutor::with_seed(
        ExecutorConfig {
            per_bucket_concurrency: 2,
            requests_per_second: 1_000,
            ..config()
        },
        42,
    )?;
    let results = stream::iter(0..12)
        .map(|_| {
            executor.execute("one-bucket", || async {
                sleep(Duration::from_millis(100)).await;
                Ok(())
            })
        })
        .buffer_unordered(12)
        .collect::<Vec<_>>()
        .await;
    for result in results {
        result?;
    }
    assert_eq!(executor.stats().peak_in_flight, 2);
    assert_eq!(executor.stats().in_flight, 0);
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn throttling_halves_rate_and_success_recovers_additively() -> TestResult {
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

#[tokio::test(start_paused = true)]
async fn terminal_errors_are_not_retried() -> TestResult {
    let executor = ApiExecutor::with_seed(config(), 42)?;
    for error in [
        ApiError::Denied("read permission absent".into()),
        ApiError::Permanent("bad request".into()),
    ] {
        let expected = error.clone();
        let result: Result<(), ApiError> = executor
            .execute("terminal", || async { Err(error.clone()) })
            .await;
        assert_eq!(result, Err(expected));
    }
    let stats = executor.stats();
    assert_eq!(stats.attempts, 2);
    assert_eq!(stats.retries, 0);
    assert_eq!(stats.failures, 2);
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn throttle_feedback_during_global_wait_rechecks_pacing_without_a_held_permit() -> TestResult
{
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

#[tokio::test(start_paused = true)]
async fn retry_exhaustion_stops_at_the_configured_attempt_count() -> TestResult {
    let executor = ApiExecutor::with_seed(config(), 42)?;
    let result: Result<(), ApiError> = executor
        .execute("failing", || async {
            Err(ApiError::Transient("retry me".into()))
        })
        .await;
    assert!(matches!(result, Err(ApiError::Exhausted(message)) if message.contains("retry me")));
    let stats = executor.stats();
    assert_eq!(stats.attempts, 3);
    assert_eq!(stats.retries, 2);
    assert_eq!(stats.failures, 1);
    assert_eq!(stats.in_flight, 0);
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn timed_out_attempts_are_cancelled_retried_and_bounded() -> TestResult {
    let executor = ApiExecutor::with_seed(
        ExecutorConfig {
            max_attempts: 2,
            attempt_timeout_ms: 10,
            requests_per_second: 1_000,
            ..config()
        },
        42,
    )?;
    let result = executor
        .execute("timeout", pending::<Result<(), ApiError>>)
        .await;
    assert!(matches!(result, Err(ApiError::Exhausted(message)) if message.contains("timeout")));
    let stats = executor.stats();
    assert_eq!(stats.attempts, 2);
    assert_eq!(stats.timeouts, 2);
    assert_eq!(stats.retries, 1);
    assert_eq!(stats.failures, 1);
    assert_eq!(stats.in_flight, 0);
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn cancellation_wakes_active_calls_and_global_waiters() -> TestResult {
    let executor = ApiExecutor::with_seed(
        ExecutorConfig {
            global_concurrency: 1,
            per_bucket_concurrency: 1,
            ..config()
        },
        42,
    )?;
    let active = executor.execute("active", pending::<Result<(), ApiError>>);
    let queued = executor.execute("queued", pending::<Result<(), ApiError>>);
    let cancel = async {
        sleep(Duration::from_millis(10)).await;
        executor.cancel();
    };
    let (first, second, ()) = tokio::join!(active, queued, cancel);
    assert_eq!(first, Err(ApiError::Cancelled));
    assert_eq!(second, Err(ApiError::Cancelled));
    let stats = executor.stats();
    assert_eq!(stats.attempts, 1);
    assert_eq!(stats.cancelled, 2);
    assert_eq!(stats.in_flight, 0);
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn cancellation_wakes_rate_and_local_reservation_waiters() -> TestResult {
    let executor = ApiExecutor::with_seed(
        ExecutorConfig {
            global_concurrency: 1,
            per_bucket_concurrency: 1,
            requests_per_second: 1,
            ..config()
        },
        42,
    )?;
    executor.execute("paced", || async { Ok(()) }).await?;
    let rate_waiter = executor.execute("paced", || async { Ok(()) });
    let local_waiter = executor.execute("paced", || async { Ok(()) });
    let cancel = async {
        sleep(Duration::from_millis(10)).await;
        executor.cancel();
    };
    let (first, second, ()) = tokio::join!(rate_waiter, local_waiter, cancel);
    assert_eq!(first, Err(ApiError::Cancelled));
    assert_eq!(second, Err(ApiError::Cancelled));
    assert_eq!(executor.stats().attempts, 1);
    assert_eq!(executor.stats().cancelled, 2);
    assert_eq!(executor.stats().in_flight, 0);
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn cancellation_wakes_backoff_before_another_attempt() -> TestResult {
    // Seed 42 yields a positive first backoff; cancellation arrives before it.
    let executor = ApiExecutor::with_seed(
        ExecutorConfig {
            requests_per_second: 100_000,
            max_backoff_ms: 1_000,
            ..config()
        },
        42,
    )?;
    let failing = executor.execute("backoff", || async {
        Err::<(), _>(ApiError::Transient("retry".into()))
    });
    let cancel = async {
        sleep(Duration::from_millis(1)).await;
        executor.cancel();
    };
    let (result, ()) = tokio::join!(failing, cancel);
    assert_eq!(result, Err(ApiError::Cancelled));
    assert_eq!(executor.stats().attempts, 1);
    assert_eq!(executor.stats().retries, 0);
    assert_eq!(executor.stats().cancelled, 1);
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn backoff_holds_no_global_permit() -> TestResult {
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

#[tokio::test(start_paused = true)]
async fn dropping_a_call_releases_capacity_and_records_cancellation() -> TestResult {
    let executor = ApiExecutor::with_seed(
        ExecutorConfig {
            global_concurrency: 1,
            per_bucket_concurrency: 1,
            ..config()
        },
        42,
    )?;
    let result = timeout(
        Duration::from_millis(10),
        executor.execute("abandoned", pending::<Result<(), ApiError>>),
    )
    .await;
    assert!(result.is_err());
    assert_eq!(executor.stats().cancelled, 1);
    assert_eq!(executor.stats().in_flight, 0);
    executor.execute("healthy", || async { Ok(()) }).await?;
    assert_eq!(executor.stats().successes, 1);
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn pre_cancelled_executor_never_invokes_the_operation() -> TestResult {
    let executor = ApiExecutor::with_seed(config(), 42)?;
    let calls = AtomicU32::new(0);
    executor.cancel();
    let result = executor
        .execute("cancelled", || async {
            calls.fetch_add(1, Ordering::Relaxed);
            Ok(())
        })
        .await;
    assert_eq!(result, Err(ApiError::Cancelled));
    assert_eq!(calls.load(Ordering::Relaxed), 0);
    assert_eq!(executor.stats().attempts, 0);
    assert_eq!(executor.stats().cancelled, 1);
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn bucket_keys_have_a_hard_byte_bound() -> TestResult {
    let executor = ApiExecutor::with_seed(config(), 42)?;
    for key in [String::new(), "x".repeat(513)] {
        assert!(matches!(
            executor.execute(&key, || async { Ok(()) }).await,
            Err(ApiError::InvalidConfig(_))
        ));
    }
    executor
        .execute(&"x".repeat(512), || async { Ok(()) })
        .await?;
    assert_eq!(executor.stats().active_buckets, 1);
    assert_eq!(executor.stats().attempts, 1);
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn retry_jitter_is_reproducible_and_bounded() -> TestResult {
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
