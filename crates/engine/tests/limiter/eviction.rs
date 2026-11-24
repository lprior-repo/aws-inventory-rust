use std::sync::atomic::AtomicBool;

use futures_util::FutureExt;
use tokio::sync::Notify;
use tokio::time::advance;

use super::*;

/// One eviction pass removes this many idle buckets.
const BATCH: u64 = 1_024;
/// The registry's bound, `MAX_BUCKETS`.
const REGISTRY: u64 = 65_536;
/// The learned-rate map's bound, `MAX_LEARNED_RATES`.
const LEARNED: u64 = 262_144;

fn hundred_per_second() -> ExecutorConfig {
    ExecutorConfig {
        requests_per_second: 100,
        max_attempts: 1,
        attempt_timeout_ms: 300_000,
        ..config()
    }
}

/// Calls `count` fresh keys once each, a second apart per thousand keys, so
/// every bucket but the newest thousand has finished its pacing interval.
/// Any result other than the one `outcome` leads to fails the test.
async fn call_fresh_keys(
    executor: &ApiExecutor,
    prefix: &str,
    count: u64,
    outcome: fn() -> Result<(), ApiError>,
) -> Result<(), ApiError> {
    let expected = outcome().map_err(exhausted_throttle);
    for key in 0..count {
        if key % 1_000 == 0 {
            advance(Duration::from_secs(1)).await;
        }
        let result = executor
            .execute(&format!("{prefix}-{key}"), || async { outcome() })
            .await;
        if result != expected {
            return Err(ApiError::Permanent(format!(
                "{prefix}-{key}: expected {expected:?}, got {result:?}"
            )));
        }
    }
    Ok(())
}

fn succeed() -> Result<(), ApiError> {
    Ok(())
}

fn throttle() -> Result<(), ApiError> {
    Err(ApiError::Throttled("slow down".into()))
}

fn exhausted_throttle(_: ApiError) -> ApiError {
    ApiError::Exhausted("throttled: slow down".into())
}

#[tokio::test(start_paused = true)]
async fn given_a_full_registry_when_seventy_four_thousand_keys_call_in_turn_then_idle_buckets_make_room()
-> TestResult {
    // Given an executor whose registry holds at most 65,536 buckets.
    let executor = ApiExecutor::with_seed(hundred_per_second(), 42)?;
    // When 74,000 distinct keys each make one call.
    call_fresh_keys(&executor, "key", 74_000, succeed).await?;
    // Then every call succeeded and nine passes of 1,024 kept the registry within its bound.
    let stats = executor.stats();
    assert_eq!(stats.successes, 74_000);
    assert_eq!(stats.evicted_buckets, 9 * BATCH);
    assert_eq!(stats.active_buckets, 74_000 - 9 * BATCH);
    assert_eq!(stats.learned_rates, 0);
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn given_buckets_still_pacing_when_the_registry_is_full_then_the_new_key_is_refused()
-> TestResult {
    // Given 65,536 buckets that all started a call in the same instant.
    let executor = ApiExecutor::with_seed(hundred_per_second(), 42)?;
    for key in 0..REGISTRY {
        executor
            .execute(&format!("key-{key}"), || async { Ok(()) })
            .await?;
    }
    // When one more key arrives before any pacing interval has passed.
    let invoked = AtomicBool::new(false);
    let refused = executor
        .execute("one-too-many", || async {
            invoked.store(true, Ordering::Relaxed);
            Ok(())
        })
        .await;
    // Then it is refused without running, and no bucket lost its pacing.
    assert_eq!(
        refused,
        Err(ApiError::Exhausted(
            "bucket registry capacity reached".into()
        ))
    );
    assert!(!invoked.load(Ordering::Relaxed));
    assert_eq!(executor.stats().evicted_buckets, 0);
    assert_eq!(executor.stats().active_buckets, REGISTRY);
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn given_a_call_in_flight_when_its_bucket_is_the_oldest_then_eviction_leaves_it_alone()
-> TestResult {
    // Given a one-call-at-a-time bucket whose only call is parked inside its operation.
    let executor = ApiExecutor::with_bucket_policies(
        hundred_per_second(),
        BTreeMap::from([(
            "held".to_owned(),
            BucketLimits {
                requests_per_second: 100,
                concurrency: 1,
            },
        )]),
    )?;
    let (entered, release) = (Notify::new(), Notify::new());
    let second_ran = AtomicBool::new(false);
    let held = executor.execute("held", || async {
        entered.notify_one();
        release.notified().await;
        Ok(())
    });
    let rest = async {
        entered.notified().await;
        // When 74,000 other keys push the registry through nine eviction passes.
        call_fresh_keys(&executor, "other", 74_000, succeed).await?;
        // Then a second call on the held key still queues behind the first.
        let second = executor.execute("held", || async {
            second_ran.store(true, Ordering::Relaxed);
            Ok(())
        });
        tokio::pin!(second);
        if second.as_mut().now_or_never().is_some() {
            release.notify_one();
            return Ok(false);
        }
        release.notify_one();
        second.await?;
        Ok::<bool, ApiError>(true)
    };
    let (held, queued) = tokio::join!(held, rest);
    held?;
    assert!(queued?, "the held bucket was replaced while its call ran");
    assert!(second_ran.load(Ordering::Relaxed));
    assert_eq!(executor.stats().evicted_buckets, 9 * BATCH);
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn given_a_throttled_bucket_when_it_is_evicted_and_returns_then_it_resumes_its_reduced_rate()
-> TestResult {
    // Given a key throttled once, from 100 down to 50 requests per second.
    let executor = ApiExecutor::with_seed(hundred_per_second(), 42)?;
    call_fresh_keys(&executor, "slow", 1, throttle).await?;
    // When 65,536 newer keys fill the registry and one pass evicts the oldest 1,024.
    call_fresh_keys(&executor, "other", REGISTRY, succeed).await?;
    assert_eq!(executor.stats().evicted_buckets, BATCH);
    assert_eq!(executor.stats().learned_rates, 1);
    // Then the key comes back at 50 a second and its first success lifts it to 51.
    let starts = Mutex::new(Vec::new());
    for _ in 0..2 {
        executor
            .execute("slow-0", || async { record(&starts) })
            .await?;
    }
    let observed = recorded(&starts)?;
    let gap = match observed.as_slice() {
        [first, second] => second.duration_since(*first),
        _ => return Err("expected two recorded starts".into()),
    };
    let at_51_per_second = Duration::from_nanos(19_607_844);
    assert!(gap >= at_51_per_second, "gap {gap:?}");
    assert!(gap <= at_51_per_second.saturating_add(Duration::from_millis(2)));
    assert_eq!(executor.stats().learned_rates, 0);
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn given_more_throttled_keys_than_the_learned_map_holds_when_evicted_then_the_map_stops_at_its_bound()
-> TestResult {
    // Given enough throttled keys to fill the learned map and force one more pass.
    let executor = ApiExecutor::with_seed(hundred_per_second(), 42)?;
    let keys = REGISTRY + LEARNED + 1;
    // When each is throttled once and evicted in turn.
    call_fresh_keys(&executor, "key", keys, throttle).await?;
    // Then 257 passes ran, the map holds exactly its bound and the last pass forgot its rates.
    let stats = executor.stats();
    assert_eq!(stats.evicted_buckets, LEARNED + BATCH);
    assert_eq!(stats.learned_rates, LEARNED);
    assert_eq!(stats.active_buckets, keys - LEARNED - BATCH);
    Ok(())
}

fn resident_kib() -> Result<u64, Box<dyn Error>> {
    let status = std::fs::read_to_string("/proc/self/status")?;
    let kib = status
        .lines()
        .find_map(|line| line.strip_prefix("VmRSS:"))
        .and_then(|rest| rest.split_whitespace().next())
        .ok_or("/proc/self/status has no VmRSS line")?;
    Ok(kib.parse()?)
}

#[tokio::test(start_paused = true)]
#[ignore = "measurement for budget P7: prints resident bytes per bucket"]
async fn measure_the_resident_memory_of_a_full_registry() -> TestResult {
    // Given an empty registry and the process's resident size.
    let executor = ApiExecutor::with_seed(hundred_per_second(), 42)?;
    let before = resident_kib()?;
    // When 65,536 buckets with production-shaped keys are created.
    for account in 0..REGISTRY {
        executor
            .execute(&format!("{account:012}/us-east-1/ec2/describe"), || async {
                Ok(())
            })
            .await?;
    }
    // Then the growth per bucket is reported.
    let after = resident_kib()?;
    let per_bucket = after
        .saturating_sub(before)
        .saturating_mul(1_024)
        .checked_div(REGISTRY)
        .ok_or("empty registry bound")?;
    println!("resident {before} KiB -> {after} KiB: {per_bucket} bytes per bucket");
    assert_eq!(executor.stats().active_buckets, REGISTRY);
    Ok(())
}
