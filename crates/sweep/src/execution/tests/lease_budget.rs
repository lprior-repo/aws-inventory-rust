use super::support::{Fixture, TestResult, page};
use super::*;
use inventory_engine::limiter::{ApiError, RetryAfter, RetryKind};
use std::{
    sync::atomic::{AtomicU32, Ordering},
    time::Duration,
};
use tokio::time::Instant;

async fn slow_call(executor: &ApiExecutor, server_delay: bool) -> Result<(), ApiError> {
    let attempts = AtomicU32::new(0);
    executor
        .execute("same-service", || {
            let attempt = attempts.fetch_add(1, Ordering::Relaxed);
            async move {
                tokio::time::sleep(Duration::from_secs(29)).await;
                if attempt == 4 {
                    return Ok(());
                }
                if server_delay {
                    Err(ApiError::RetryAfter {
                        kind: RetryKind::Transient,
                        message: "retry the read".to_owned(),
                        retry_after: RetryAfter::Delay(Duration::from_secs(40)),
                    })
                } else {
                    Err(ApiError::Transient("retry the read".to_owned()))
                }
            }
        })
        .await
}

#[tokio::test]
async fn given_slow_discovery_and_target_when_page_commits_then_cursor_advances_once_without_lease_replay()
-> TestResult {
    let fixture = Fixture::new("discovery-target-budget").await?;
    let lease = fixture.seed().await?;
    let generation = lease.generation();
    let executor = ApiExecutor::with_seed(executor_config(4)?, 42)?;
    tokio::time::pause();
    let start = Instant::now();
    let calls = async {
        slow_call(&executor, true).await?;
        slow_call(&executor, true).await
    }
    .await;
    let elapsed = i64::try_from(start.elapsed().as_millis())?;
    tokio::time::resume();
    calls?;
    // The old one-call lease was305s at concurrency4. Both real retrying
    // executor calls must finish before the same durable page is committed.
    assert!(elapsed > 305_000);
    assert!(elapsed < fixture.lease_ms);
    assert_eq!(executor.stats().attempts, 10);
    let at = fixture
        .now
        .checked_add(elapsed)
        .ok_or("test timestamp overflow")?;
    let committed = fixture
        .store
        .commit_page(page(lease, at, Some(b"page-2".to_vec())))
        .await?;
    assert!(!committed.repeated_token);
    let next = fixture
        .store
        .claim(
            &fixture.sweep,
            at.checked_add(1).ok_or("test timestamp overflow")?,
            fixture.lease_ms,
        )
        .await?
        .ok_or("continuation lost")?;
    assert_eq!(next.token(), Some(b"page-2".as_slice()));
    assert!(next.generation() > generation);
    fixture
        .store
        .commit_page(page(
            next,
            at.checked_add(2).ok_or("test timestamp overflow")?,
            None,
        ))
        .await?;
    fixture
        .outcome(inventory_core::inventory::ScopeOutcome::Complete)
        .await?;
    Ok(())
}

#[tokio::test]
async fn given_sixty_four_same_service_leases_when_eight_permits_are_saturated_then_every_queued_page_commits()
-> TestResult {
    let fixture = Fixture::new("queued-permit-budget").await?;
    let settings = executor_config(64)?;
    let lease_ms = lease_budget(settings)?;
    let executor = ApiExecutor::with_seed(settings, 42)?;
    for ordinal in 0..64 {
        fixture
            .store
            .seed_work(WorkSpec::new(
                fixture.sweep.clone(),
                fixture.scope_for(ordinal)?,
                None,
                Vec::new(),
            )?)
            .await?;
    }
    let mut leases = Vec::new();
    for _ in 0..64 {
        leases.push(
            fixture
                .store
                .claim(&fixture.sweep, fixture.now, lease_ms)
                .await?
                .ok_or("batch lease absent")?,
        );
    }
    tokio::time::pause();
    let start = Instant::now();
    let results = stream::iter(0..64)
        .map(|_| slow_call(&executor, false))
        .buffer_unordered(64)
        .collect::<Vec<_>>()
        .await;
    let elapsed = i64::try_from(start.elapsed().as_millis())?;
    tokio::time::resume();
    for result in results {
        result?;
    }
    assert!(
        elapsed > 365_000,
        "fixture did not exercise the original queued-permit expiry"
    );
    assert!(elapsed < lease_ms);
    assert_eq!(executor.stats().attempts, 320);
    let at = fixture
        .now
        .checked_add(elapsed)
        .ok_or("test timestamp overflow")?;
    for lease in leases {
        fixture.store.commit_page(page(lease, at, None)).await?;
    }
    let summary = fixture.store.sweep_summary(&fixture.sweep).await?;
    assert_eq!(summary.complete_scopes, 64);
    assert_eq!(summary.work_pending, 0);
    Ok(())
}
