use super::*;

pub(super) async fn check_tape(tape: &[Step], budget: u32, seed: u64) -> TestCaseResult {
    let executor = ApiExecutor::with_seed(config(budget), seed).map_err(failure)?;
    let calls = Cell::new(0_usize);
    let (expected_calls, expected_result) = oracle(tape, usize::try_from(budget).map_err(failure)?);
    let result = timeout(
        CASE_DEADLINE,
        executor.execute("script", || {
            let index = calls.get();
            calls.set(index.saturating_add(1));
            respond(tape.get(index).copied(), index)
        }),
    )
    .await
    .map_err(failure)?;
    prop_assert_eq!(terminal(&result), expected_result);
    prop_assert_eq!(calls.get(), expected_calls);
    let stats = executor.stats();
    let attempts = u64::try_from(expected_calls).map_err(failure)?;
    prop_assert_eq!(stats.attempts, attempts);
    prop_assert_eq!(stats.retries, attempts.saturating_sub(1));
    prop_assert_eq!(stats.in_flight, 0);
    prop_assert_eq!(stats.peak_in_flight, 1);
    prop_assert_eq!(stats.active_buckets, 1);
    prop_assert_eq!(
        stats.throttles,
        u64::try_from(
            tape.iter()
                .take(expected_calls)
                .filter(|step| **step == Step::Throttled)
                .count()
        )
        .map_err(failure)?
    );
    prop_assert_eq!(
        stats.timeouts,
        u64::try_from(
            tape.iter()
                .take(expected_calls)
                .filter(|step| **step == Step::AttemptTimeout)
                .count()
        )
        .map_err(failure)?
    );
    prop_assert_eq!(stats.successes, u64::from(result.is_ok()));
    prop_assert_eq!(
        stats.cancelled,
        u64::from(expected_result == Terminal::Cancelled)
    );
    prop_assert_eq!(
        stats.failures,
        u64::from(result.is_err() && expected_result != Terminal::Cancelled)
    );
    prop_assert_eq!(stats.server_delays, 0);
    prop_assert_eq!(stats.rejected_retry_hints, 0);

    for bucket in ["script", "independent"] {
        let followup = timeout(
            CASE_DEADLINE,
            executor.execute(bucket, || future::ready(Ok(71_u8))),
        )
        .await
        .map_err(failure)?
        .map_err(failure)?;
        prop_assert_eq!(followup, 71);
        prop_assert_eq!(executor.stats().in_flight, 0);
    }
    prop_assert_eq!(executor.stats().attempts, attempts.saturating_add(2));
    Ok(())
}
