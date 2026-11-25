use super::*;

const EAST: &str = "123456789012/us-east-1/lambda";
const WEST: &str = "123456789012/us-west-2/lambda";

fn rising() -> Result<ApiExecutor, ApiError> {
    ApiExecutor::with_default_policy(
        config(),
        BucketPolicy::Adaptive(AdaptiveLimits {
            start_rps: 10,
            floor_rps: 1,
            ceiling_rps: 100,
            concurrency: 2,
        }),
        BTreeMap::new(),
    )
}

async fn succeed(
    executor: &ApiExecutor,
    key: &str,
    starts: &Mutex<Vec<Instant>>,
    calls: usize,
) -> Result<(), ApiError> {
    for _ in 0..calls {
        executor.execute(key, || async { record(starts) }).await?;
    }
    Ok(())
}

async fn throttled_then_succeed(
    executor: &ApiExecutor,
    key: &str,
    starts: &Mutex<Vec<Instant>>,
) -> Result<(), ApiError> {
    let attempts = AtomicU32::new(0);
    executor
        .execute(key, || async {
            record(starts)?;
            if attempts.fetch_add(1, Ordering::Relaxed) == 0 {
                Err(ApiError::Throttled("slow down".into()))
            } else {
                Ok(())
            }
        })
        .await
}

fn gaps(starts: &Mutex<Vec<Instant>>) -> Result<Vec<Duration>, ApiError> {
    let observed = recorded(starts)?;
    Ok(observed
        .iter()
        .zip(observed.iter().skip(1))
        .map(|(first, second)| second.duration_since(*first))
        .collect())
}

#[tokio::test(start_paused = true)]
pub(super) async fn given_two_regions_of_one_service_when_both_are_called_then_each_is_paced_at_its_own_rate()
-> TestResult {
    // Given: two buckets for the same service in different regions.
    let executor = rising()?;
    let east_starts = Mutex::new(Vec::new());
    let west_starts = Mutex::new(Vec::new());

    // When: each bucket is called three times.
    succeed(&executor, EAST, &east_starts, 3).await?;
    succeed(&executor, WEST, &west_starts, 3).await?;

    // Then: each bucket paces itself at the start rate of 10 rps.
    let east_gaps = gaps(&east_starts)?;
    let west_gaps = gaps(&west_starts)?;
    assert_eq!(east_gaps.len(), 2);
    assert_eq!(west_gaps.len(), 2);
    assert_eq!(east_gaps, west_gaps);
    for gap in east_gaps {
        assert!(gap >= Duration::from_millis(90), "{gap:?}");
        assert!(gap <= Duration::from_millis(100), "{gap:?}");
    }
    let mut rates = executor.bucket_rates();
    rates.sort();
    assert_eq!(
        rates,
        vec![(EAST.to_owned(), 10, 3), (WEST.to_owned(), 10, 3)]
    );
    Ok(())
}

#[tokio::test(start_paused = true)]
pub(super) async fn given_two_regions_of_one_service_when_one_is_throttled_then_the_other_keeps_its_rate()
-> TestResult {
    // Given: two buckets for the same service in different regions.
    let executor = rising()?;
    let east_starts = Mutex::new(Vec::new());
    let west_starts = Mutex::new(Vec::new());

    // When: the east bucket is throttled while the west bucket is called.
    throttled_then_succeed(&executor, EAST, &east_starts).await?;
    succeed(&executor, EAST, &east_starts, 1).await?;
    succeed(&executor, WEST, &west_starts, 3).await?;

    // Then: the east bucket is cut to 70% while the west bucket is untouched.
    let east_gaps = gaps(&east_starts)?;
    let west_gaps = gaps(&west_starts)?;
    assert_eq!(east_gaps.len(), 2);
    assert_eq!(west_gaps.len(), 2);
    for gap in east_gaps {
        assert!(gap >= Duration::from_millis(137), "{gap:?}");
    }
    for gap in west_gaps {
        assert!(gap <= Duration::from_millis(100), "{gap:?}");
    }
    assert_eq!(executor.stats().throttles, 1);
    let mut rates = executor.bucket_rates();
    rates.sort();
    assert_eq!(
        rates,
        vec![(EAST.to_owned(), 7, 3), (WEST.to_owned(), 10, 3)]
    );
    Ok(())
}
