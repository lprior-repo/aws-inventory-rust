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
    let executor = executor(Some(4))?;

    let elapsed = succeed(&executor, 40).await?;

    assert!(elapsed >= Duration::from_secs(9), "{elapsed:?}");
    assert!(current_rate(&executor)? <= 4);
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn given_no_cap_when_two_hundred_calls_succeed_then_the_rate_rose_above_the_start()
-> TestResult {
    let executor = executor(None)?;

    let elapsed = succeed(&executor, 200).await?;

    assert!(elapsed < Duration::from_secs(19), "{elapsed:?}");
    assert!(current_rate(&executor)? > 10);
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn given_a_cap_of_twenty_when_four_hundred_calls_succeed_then_the_rate_stops_at_the_cap()
-> TestResult {
    let executor = executor(Some(20))?;

    let elapsed = succeed(&executor, 400).await?;

    assert_eq!(current_rate(&executor)?, 20);
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
    let credentials = crate::credentials::TestCredentials::new("us-east-1");
    let resources = collection::Resources::new(&sweep_config(Some(3)), &credentials)?;

    succeed(&resources.executor, 1).await?;

    assert_eq!(current_rate(&resources.executor)?, 3);
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn given_a_sweep_without_a_cap_when_its_resources_are_built_then_its_bucket_starts_at_ten()
-> TestResult {
    let credentials = crate::credentials::TestCredentials::new("us-east-1");
    let resources = collection::Resources::new(&sweep_config(None), &credentials)?;

    succeed(&resources.executor, 1).await?;

    assert_eq!(current_rate(&resources.executor)?, 10);
    Ok(())
}
