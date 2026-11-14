use super::*;

pub(super) fn occupancy(assignments: &[bool], global: usize, local: usize) -> usize {
    let first = assignments
        .iter()
        .filter(|value| **value)
        .count()
        .min(local);
    let second = assignments
        .iter()
        .filter(|value| !**value)
        .count()
        .min(local);
    first.saturating_add(second).min(global)
}

pub(super) async fn occupy_then_release(
    executor: &ApiExecutor,
    assignments: &[bool],
    expected: usize,
    cancel: bool,
) -> TestCaseResult {
    let before = executor.stats();
    let starts = Cell::new(0_usize);
    let mut calls: Vec<_> = assignments
        .iter()
        .map(|first| {
            Box::pin(
                executor.execute(if *first { "first" } else { "second" }, || {
                    starts.set(starts.get().saturating_add(1));
                    pending::<Result<(), ApiError>>()
                }),
            )
        })
        .collect();

    for _ in 0..9 {
        advance(Duration::from_millis(1)).await;
        for call in &mut calls {
            prop_assert!(call.as_mut().now_or_never().is_none());
        }
    }
    prop_assert_eq!(starts.get(), expected);
    prop_assert_eq!(
        executor.stats().in_flight,
        u64::try_from(expected).map_err(failure)?
    );
    if cancel {
        executor.cancel();
        let outcomes = timeout(Duration::from_secs(1), future::join_all(calls))
            .await
            .map_err(failure)?;
        prop_assert!(
            outcomes
                .iter()
                .all(|result| *result == Err(ApiError::Cancelled))
        );
    } else {
        drop(calls);
    }
    let after = executor.stats();
    prop_assert_eq!(after.in_flight, 0);
    prop_assert_eq!(
        after.cancelled.saturating_sub(before.cancelled),
        u64::try_from(assignments.len()).map_err(failure)?
    );
    prop_assert_eq!(after.failures, before.failures);
    prop_assert_eq!(
        after.attempts.saturating_sub(before.attempts),
        u64::try_from(expected).map_err(failure)?
    );
    Ok(())
}

pub(super) async fn check_permits(
    global: usize,
    local: usize,
    assignments: &[bool],
    cancel: bool,
) -> TestCaseResult {
    let executor = ApiExecutor::with_seed(
        ExecutorConfig {
            global_concurrency: global,
            per_bucket_concurrency: local,
            attempt_timeout_ms: 5_000,
            requests_per_second: 100_000,
            ..config(1)
        },
        19,
    )
    .map_err(failure)?;
    let expected = occupancy(assignments, global, local);
    occupy_then_release(&executor, assignments, expected, cancel).await?;
    if cancel {
        let called = Cell::new(false);
        let result = executor
            .execute("new", || {
                called.set(true);
                future::ready(Ok(()))
            })
            .await;
        prop_assert_eq!(result, Err(ApiError::Cancelled));
        prop_assert!(!called.get());
    } else {
        occupy_then_release(&executor, assignments, expected, false).await?;
    }
    prop_assert_eq!(executor.stats().in_flight, 0);
    prop_assert!(executor.stats().peak_in_flight <= u64::try_from(global).map_err(failure)?);
    Ok(())
}
