use super::*;

const KEY: &str = "aws:123456789012:us-east-1:lambda:control_read";
const OTHER: &str = "aws:123456789012:us-east-2:lambda:control_read";

fn adaptive(start_rps: u32, floor_rps: u32, ceiling_rps: u32, concurrency: usize) -> BucketPolicy {
    BucketPolicy::Adaptive(AdaptiveLimits {
        start_rps,
        floor_rps,
        ceiling_rps,
        concurrency,
    })
}

fn rising(start_rps: u32, floor_rps: u32, ceiling_rps: u32) -> Result<ApiExecutor, ApiError> {
    ApiExecutor::with_default_policy(
        config(),
        adaptive(start_rps, floor_rps, ceiling_rps, 2),
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

fn paced(actual: &[Duration], expected_ns: &[u64]) {
    assert_eq!(actual.len(), expected_ns.len());
    for (actual, nanos) in actual.iter().zip(expected_ns) {
        let minimum = Duration::from_nanos(*nanos);
        assert!(*actual >= minimum, "{actual:?} is below {minimum:?}");
        assert!(
            *actual <= minimum.saturating_add(Duration::from_millis(2)),
            "{actual:?} is above {minimum:?}"
        );
    }
}

#[tokio::test(start_paused = true)]
pub(super) async fn given_a_rising_policy_when_calls_succeed_then_each_gap_is_shorter() -> TestResult
{
    // Given: 1 request per second to start with, up to 100.
    let executor = rising(1, 1, 100)?;
    let starts = Mutex::new(Vec::new());

    // When: four calls succeed.
    succeed(&executor, KEY, &starts, 4).await?;

    // Then: the rate was 2.0, 2.5 and 2.9 requests per second.
    paced(&gaps(&starts)?, &[500_000_000, 400_000_000, 344_827_587]);
    Ok(())
}

#[tokio::test(start_paused = true)]
pub(super) async fn given_a_rising_policy_when_throttled_then_the_rate_drops_to_seventy_percent()
-> TestResult {
    // Given: 10 requests per second to start with.
    let executor = rising(10, 1, 100)?;
    let starts = Mutex::new(Vec::new());

    // When: the first attempt is throttled and three attempts then succeed.
    throttled_then_succeed(&executor, KEY, &starts).await?;
    succeed(&executor, KEY, &starts, 2).await?;

    // Then: the rate was 7.000, 7.142 and 7.282 requests per second.
    paced(&gaps(&starts)?, &[142_857_143, 140_016_803, 137_324_911]);
    assert_eq!(executor.stats().throttles, 1);
    Ok(())
}

#[tokio::test(start_paused = true)]
pub(super) async fn given_a_throttle_when_calls_succeed_then_the_rate_holds_at_ninety_percent_for_a_minute()
-> TestResult {
    // Given: a throttle arrived at 10 requests per second.
    let executor = rising(10, 1, 100)?;
    let starts = Mutex::new(Vec::new());
    throttled_then_succeed(&executor, KEY, &starts).await?;

    // When: 19 more calls succeed inside the minute.
    succeed(&executor, KEY, &starts, 19).await?;

    // Then: the rate stopped at 9 requests per second.
    let held = gaps(&starts)?;
    assert_eq!(held.len(), 20);
    paced(held.get(17..).ok_or("three held gaps")?, &[111_111_112; 3]);

    // When: the minute has passed and two more calls succeed.
    sleep(Duration::from_secs(60)).await;
    let later = Mutex::new(Vec::new());
    succeed(&executor, KEY, &later, 2).await?;

    // Then: the rate rose to 9.111 requests per second.
    paced(&gaps(&later)?, &[109_757_437]);
    Ok(())
}

#[tokio::test(start_paused = true)]
pub(super) async fn given_equal_floor_start_and_ceiling_when_throttled_then_the_rate_is_unchanged()
-> TestResult {
    // Given: floor, start and ceiling are all 5 requests per second.
    let executor = rising(5, 5, 5)?;
    let starts = Mutex::new(Vec::new());

    // When: one attempt is throttled and three succeed.
    throttled_then_succeed(&executor, KEY, &starts).await?;
    succeed(&executor, KEY, &starts, 2).await?;

    // Then: every gap is one fifth of a second.
    paced(&gaps(&starts)?, &[200_000_000; 3]);
    Ok(())
}

#[test]
pub(super) fn given_out_of_range_adaptive_limits_when_building_then_the_configuration_is_rejected()
-> TestResult {
    // Given: limits just outside and just inside each bound (global concurrency is 4).
    let rejected = [
        (1, 0, 100, 2),
        (1, 2, 100, 2),
        (101, 1, 100, 2),
        (10, 1, 100_001, 2),
        (10, 1, 100, 0),
        (10, 1, 100, 5),
    ];
    let accepted = [(1, 1, 1, 1), (100_000, 1, 100_000, 4), (10, 10, 100, 4)];

    // When, Then: as the default policy and as a keyed policy.
    for (start, floor, ceiling, concurrency) in rejected {
        let policy = adaptive(start, floor, ceiling, concurrency);
        let invalid = |error: &ApiError| matches!(error, ApiError::InvalidConfig(_));
        require_error(
            ApiExecutor::with_default_policy(config(), policy, BTreeMap::new()).map(|_| ()),
            invalid,
        )?;
        require_error(
            ApiExecutor::with_rate_policies(config(), BTreeMap::from([(KEY.into(), policy)]))
                .map(|_| ()),
            invalid,
        )?;
    }
    for (start, floor, ceiling, concurrency) in accepted {
        let policy = adaptive(start, floor, ceiling, concurrency);
        ApiExecutor::with_default_policy(config(), policy, BTreeMap::new())?;
    }
    Ok(())
}

#[test]
pub(super) fn given_adaptive_policy_json_when_read_then_the_shape_is_exact() -> TestResult {
    // Given
    let text = r#"{"start_rps":10,"floor_rps":1,"ceiling_rps":100,"concurrency":2}"#;

    // When
    let policy = serde_json::from_str::<BucketPolicy>(text)?;

    // Then
    assert_eq!(policy, adaptive(10, 1, 100, 2));
    assert_eq!(serde_json::to_string(&policy)?, text);
    for invalid in [
        r#"{"start_rps":10,"floor_rps":1,"ceiling_rps":100,"concurrency":2,"celing_rps":5}"#,
        r#"{"start_rps":10,"floor_rps":1,"ceiling_rps":100,"concurrency":2,"requests_per_second":5}"#,
        r#"{"start_rps":10,"floor_rps":1,"concurrency":2}"#,
        r#"{"start_rps":1.5,"floor_rps":1,"ceiling_rps":100,"concurrency":2}"#,
    ] {
        assert!(serde_json::from_str::<BucketPolicy>(invalid).is_err());
    }
    Ok(())
}

#[tokio::test(start_paused = true)]
pub(super) async fn given_a_rising_default_when_a_key_has_its_own_policy_then_only_other_keys_rise()
-> TestResult {
    // Given: a rising default and a fixed 20 requests per second for one key.
    let executor = ApiExecutor::with_default_policy(
        config(),
        adaptive(1, 1, 100, 2),
        BTreeMap::from([(
            KEY.into(),
            BucketPolicy::PerSecond(BucketLimits {
                requests_per_second: 20,
                concurrency: 2,
            }),
        )]),
    )?;
    let fixed = Mutex::new(Vec::new());
    let risen = Mutex::new(Vec::new());

    // When
    succeed(&executor, KEY, &fixed, 3).await?;
    succeed(&executor, OTHER, &risen, 3).await?;

    // Then
    paced(&gaps(&fixed)?, &[50_000_000; 2]);
    paced(&gaps(&risen)?, &[500_000_000, 400_000_000]);
    Ok(())
}

#[tokio::test(start_paused = true)]
pub(super) async fn given_a_fixed_default_when_a_key_has_a_rising_policy_then_only_that_key_rises()
-> TestResult {
    // Given: the fixed default of 10 requests per second and one rising key.
    let executor = ApiExecutor::with_rate_policies(
        config(),
        BTreeMap::from([(KEY.into(), adaptive(1, 1, 100, 2))]),
    )?;
    let risen = Mutex::new(Vec::new());
    let fixed = Mutex::new(Vec::new());

    // When
    succeed(&executor, KEY, &risen, 3).await?;
    succeed(&executor, OTHER, &fixed, 3).await?;

    // Then
    paced(&gaps(&risen)?, &[500_000_000, 400_000_000]);
    paced(&gaps(&fixed)?, &[100_000_000; 2]);
    Ok(())
}
