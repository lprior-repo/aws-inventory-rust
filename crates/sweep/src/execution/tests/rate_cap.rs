use super::*;
use inventory_engine::limiter::ApiError;
use std::{collections::BTreeMap, time::Duration};
use tokio::time::Instant;

type TestResult = Result<(), Box<dyn std::error::Error>>;
const KEY: &str = "123456789012/us-east-1/ec2";

fn executor(max_rps: Option<u32>) -> Result<ApiExecutor, Box<dyn std::error::Error>> {
    let settings = executor_config(8)?;
    Ok(ApiExecutor::with_default_policy(
        settings,
        rate::policy(max_rps, settings.per_bucket_concurrency),
        BTreeMap::new(),
    )?)
}

async fn succeed(executor: &ApiExecutor, calls: u32) -> Result<Duration, ApiError> {
    let start = Instant::now();
    for _ in 0..calls {
        executor
            .execute(KEY, || async { Ok::<(), ApiError>(()) })
            .await?;
    }
    Ok(start.elapsed())
}

fn current_rate(executor: &ApiExecutor) -> Result<u32, &'static str> {
    let entries = executor.bucket_rates();
    for (key, rate, _count) in entries {
        if key == KEY {
            return Ok(rate);
        }
    }
    Err("bucket absent")
}

#[tokio::test(start_paused = true)]
async fn given_a_cap_of_four_when_forty_calls_succeed_then_they_take_at_least_nine_seconds()
-> TestResult {
    // Given a rate cap of 4 requests per second.
    let executor = executor(Some(4))?;

    // When 40 calls succeed.
    let elapsed = succeed(&executor, 40).await?;

    // Then they take at least 9 seconds: the first call is immediate and the
    // other 39 are spaced 250 ms apart, which is 9.75 s.
    assert!(elapsed >= Duration::from_secs(9), "{elapsed:?}");
    assert!(current_rate(&executor)? <= 4);
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn given_no_cap_when_two_hundred_calls_succeed_then_the_rate_rose_above_the_start()
-> TestResult {
    // Given no rate cap (default ceiling).
    let executor = executor(None)?;

    // When 200 calls succeed.
    let elapsed = succeed(&executor, 200).await?;

    // Then the rate rose above the start of 10/s, finishing faster than 10/s fixed.
    // At 10/s fixed, 200 calls would take ~19.9 seconds.
    assert!(elapsed < Duration::from_secs(19), "{elapsed:?}");
    assert!(current_rate(&executor)? > 10);
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn given_a_cap_of_twenty_when_four_hundred_calls_succeed_then_the_rate_stops_at_the_cap()
-> TestResult {
    // Given a rate cap of 20 requests per second.
    let executor = executor(Some(20))?;

    // When 400 calls succeed.
    let elapsed = succeed(&executor, 400).await?;

    // Then the rate stops at the cap.
    assert_eq!(current_rate(&executor)?, 20);
    // At 20/s, 400 calls take at least 20s (first is immediate, so 19s).
    assert!(elapsed >= Duration::from_secs(19), "{elapsed:?}");
    Ok(())
}

fn sweep_config(max_rps: Option<u32>) -> SweepConfig {
    SweepConfig {
        region: None,
        max_pages: 1,
        concurrency: 8,
        resume: false,
        endpoint: None,
        ca_bundle: None,
        idle_timeout: None,
        pool_idle_per_host: None,
        slice_timeout_secs: None,
        max_rps,
        max_page_size: None,
    }
}

#[tokio::test(start_paused = true)]
async fn given_a_sweep_with_a_cap_of_three_when_its_resources_are_built_then_its_bucket_runs_at_three()
-> TestResult {
    // Given a sweep configuration capped at 3 requests per second.
    let credentials = crate::credentials::TestCredentials::new("us-east-1");
    let resources = collection::Resources::new(&sweep_config(Some(3)), &credentials)?;

    // When one call goes through the sweep's own executor.
    succeed(&resources.executor, 1).await?;

    // Then its bucket follows the rising policy, held at the cap.
    assert_eq!(current_rate(&resources.executor)?, 3);
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn given_a_sweep_without_a_cap_when_its_resources_are_built_then_its_bucket_starts_at_ten()
-> TestResult {
    // Given a sweep configuration with no cap.
    let credentials = crate::credentials::TestCredentials::new("us-east-1");
    let resources = collection::Resources::new(&sweep_config(None), &credentials)?;

    // When one call goes through the sweep's own executor.
    succeed(&resources.executor, 1).await?;

    // Then its bucket follows the rising policy from the start rate.
    assert_eq!(current_rate(&resources.executor)?, 10);
    Ok(())
}
