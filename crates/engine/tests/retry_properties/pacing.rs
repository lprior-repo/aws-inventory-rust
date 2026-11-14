use super::*;

pub(super) async fn check_rational_pacing(
    requests: u32,
    period_ms: u64,
    count: usize,
) -> TestCaseResult {
    let executor = ApiExecutor::with_rate_policies(
        ExecutorConfig {
            global_concurrency: 4,
            per_bucket_concurrency: 4,
            ..config(1)
        },
        BTreeMap::from([(
            "paced".to_owned(),
            BucketPolicy::PerPeriod(PeriodLimits {
                requests,
                period_ms,
                concurrency: 4,
            }),
        )]),
    )
    .map_err(failure)?;
    let starts = RefCell::new(Vec::new());
    let outcomes = timeout(
        CASE_DEADLINE,
        stream::iter(0..count)
            .map(|_| {
                executor.execute("paced", || {
                    starts.borrow_mut().push(Instant::now());
                    future::ready(Ok(()))
                })
            })
            .buffer_unordered(4)
            .collect::<Vec<_>>(),
    )
    .await
    .map_err(failure)?;
    prop_assert!(outcomes.iter().all(Result::is_ok));
    let starts = starts.into_inner();
    prop_assert_eq!(starts.len(), count);
    for (first, second) in starts.iter().zip(starts.iter().skip(1)) {
        let elapsed = second.duration_since(*first).as_nanos();
        prop_assert!(
            elapsed.saturating_mul(u128::from(requests))
                >= u128::from(period_ms).saturating_mul(1_000_000)
        );
    }
    prop_assert_eq!(
        executor.stats().attempts,
        u64::try_from(count).map_err(failure)?
    );
    prop_assert_eq!(executor.stats().in_flight, 0);
    Ok(())
}
