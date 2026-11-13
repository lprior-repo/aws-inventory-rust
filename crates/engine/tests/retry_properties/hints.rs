//! Generated server delay and rejected-hint properties.
use super::*;

pub(super) fn kind(index: u8) -> RetryKind {
    match index {
        0 => RetryKind::Throttled,
        1 => RetryKind::Transient,
        _ => RetryKind::Timeout,
    }
}

pub(super) fn directed(kind: RetryKind, retry_after: RetryAfter) -> ApiError {
    ApiError::RetryAfter {
        kind,
        message: "synthetic server directive".into(),
        retry_after,
    }
}

pub(super) async fn check_delay(delay_ms: u64, retry_kind: RetryKind, seed: u64) -> TestCaseResult {
    let executor = ApiExecutor::with_seed(config(2), seed).map_err(failure)?;
    let starts = RefCell::new(Vec::new());
    let peer_start = Cell::new(None);
    let delayed = executor.execute("delayed", || {
        let index = starts.borrow().len();
        starts.borrow_mut().push(Instant::now());
        future::ready(if index == 0 {
            Err(directed(
                retry_kind,
                RetryAfter::Delay(Duration::from_millis(delay_ms)),
            ))
        } else {
            Ok(())
        })
    });
    let peer = executor.execute("unrelated", || {
        peer_start.set(Some(Instant::now()));
        future::ready(Ok(()))
    });
    let deadline = Duration::from_millis(delay_ms).saturating_add(CASE_DEADLINE);
    let (delayed, peer) = timeout(deadline, async { tokio::join!(delayed, peer) })
        .await
        .map_err(failure)?;
    delayed.map_err(failure)?;
    peer.map_err(failure)?;
    let starts = starts.into_inner();
    prop_assert_eq!(starts.len(), 2);
    let first = starts
        .first()
        .copied()
        .ok_or_else(|| failure("missing first attempt"))?;
    let second = starts
        .last()
        .copied()
        .ok_or_else(|| failure("missing retry"))?;
    prop_assert!(second.duration_since(first) >= Duration::from_millis(delay_ms));
    let peer = peer_start
        .get()
        .ok_or_else(|| failure("unrelated bucket never ran"))?;
    prop_assert!(peer.duration_since(first) <= Duration::from_millis(1));
    let stats = executor.stats();
    prop_assert_eq!(stats.attempts, 3);
    prop_assert_eq!(stats.retries, 1);
    prop_assert_eq!(stats.server_delays, 1);
    prop_assert_eq!(
        stats.throttles,
        u64::from(retry_kind == RetryKind::Throttled)
    );
    prop_assert_eq!(stats.timeouts, u64::from(retry_kind == RetryKind::Timeout));
    prop_assert_eq!(stats.successes, 2);
    prop_assert_eq!(stats.failures, 0);
    prop_assert_eq!(stats.in_flight, 0);
    Ok(())
}

pub(super) async fn check_rejected_hint(
    hint: RetryAfter,
    retry_kind: RetryKind,
    seed: u64,
) -> TestCaseResult {
    let executor = ApiExecutor::with_seed(config(MAX_ATTEMPTS), seed).map_err(failure)?;
    let calls = Cell::new(0_u32);
    for _ in 0..3 {
        let result = executor
            .execute("closed", || {
                calls.set(calls.get().saturating_add(1));
                future::ready(Err::<(), _>(directed(retry_kind, hint)))
            })
            .await;
        prop_assert!(matches!(result, Err(ApiError::RetryAfterRejected(_))));
    }
    prop_assert_eq!(calls.get(), 1);
    executor
        .execute("open", || future::ready(Ok(())))
        .await
        .map_err(failure)?;
    let stats = executor.stats();
    prop_assert_eq!(stats.attempts, 2);
    prop_assert_eq!(stats.retries, 0);
    prop_assert_eq!(stats.rejected_retry_hints, 1);
    prop_assert_eq!(stats.failures, 3);
    prop_assert_eq!(stats.successes, 1);
    prop_assert_eq!(stats.in_flight, 0);
    Ok(())
}
