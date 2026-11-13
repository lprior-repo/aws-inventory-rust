//! Lease/recovery behavior exercised through real SQLx transactions.
use inventory_engine::jobs::{Completion, JobError, JobSpec, JobState, JobStore};
use inventory_engine::store::Store;
use serde_json::json;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn spec(id: &str, attempts: u32) -> JobSpec {
    JobSpec {
        job_id: id.into(),
        kind: "aws_scan".into(),
        payload: json!({"profile":"synthetic"}),
        max_attempts: attempts,
        available_at: 100,
    }
}
fn success() -> Completion {
    Completion {
        state: JobState::Succeeded,
        result: Some(json!({"status":"complete","synthetic":true})),
        error: None,
    }
}

#[tokio::test]
async fn submission_is_idempotent_but_different_work_conflicts() -> TestResult {
    let store = JobStore::open(":memory:").await?;
    let first = store.submit(spec("job-one", 3), 100).await?;
    let mut repeated = spec("job-one", 3);
    repeated.available_at = 900;
    let again = store.submit(repeated, 200).await?;
    assert_eq!(again.created_at, first.created_at);
    assert_eq!(again.available_at, 100);
    let mut other = spec("job-one", 3);
    other.payload = json!({"profile":"different"});
    assert!(matches!(
        store.submit(other, 300).await,
        Err(JobError::Conflict)
    ));
    assert!(matches!(
        store.submit(spec("job-one", 4), 300).await,
        Err(JobError::Conflict)
    ));
    assert_eq!(store.list(10).await?.len(), 1);
    store.close().await;
    Ok(())
}

#[tokio::test]
async fn schedule_and_deterministic_claim_order() -> TestResult {
    let store = JobStore::open(":memory:").await?;
    store.submit(spec("z", 2), 100).await?;
    store.submit(spec("a", 2), 100).await?;
    assert!(store.claim("worker", "early", 99, 10000).await?.is_none());
    let first = store
        .claim("worker", "one", 100, 10000)
        .await?
        .ok_or("missing claim")?;
    assert_eq!(first.job.job_id, "a");
    assert_eq!(first.job.attempt, 1);
    let second = store
        .claim("worker", "two", 100, 10000)
        .await?
        .ok_or("missing second")?;
    assert_eq!(second.job.job_id, "z");
    assert!(store.claim("worker", "third", 100, 10000).await?.is_none());
    store.close().await;
    Ok(())
}

#[tokio::test]
async fn competing_database_connections_claim_only_once() -> TestResult {
    let folder = tempfile::tempdir()?;
    let path = folder.path().join("queue.sqlite");
    let a = JobStore::open(&path).await?;
    let b = JobStore::open(&path).await?;
    a.submit(spec("single", 3), 100).await?;
    let (one, two) = tokio::join!(
        a.claim("a", "token-a", 100, 10000),
        b.claim("b", "token-b", 100, 10000)
    );
    let claimed = [one?, two?].into_iter().flatten().count();
    assert_eq!(claimed, 1);
    assert_eq!(a.history("single").await?.len(), 1);
    a.close().await;
    b.close().await;
    Ok(())
}

#[tokio::test]
async fn expired_attempt_is_recovered_and_old_owner_is_fenced() -> TestResult {
    let store = JobStore::open(":memory:").await?;
    store.submit(spec("recover", 3), 100).await?;
    let mut old = store
        .claim("same-worker", "same-token", 100, 10000)
        .await?
        .ok_or("old lease")?;
    let new = store
        .claim("same-worker", "same-token", 10100, 10000)
        .await?
        .ok_or("new lease")?;
    assert_eq!(new.job.attempt, 2);
    // Presentation metadata cannot forge the immutable fencing capability.
    old.job.attempt = new.job.attempt;
    assert!(matches!(
        store.heartbeat(&old, 10101, 10000).await,
        Err(JobError::LostLease)
    ));
    assert!(matches!(
        store.complete(&old, success(), 10101).await,
        Err(JobError::LostLease)
    ));
    let done = store.complete(&new, success(), 10102).await?;
    assert_eq!(done.state, JobState::Succeeded);
    let history = store.history("recover").await?;
    assert_eq!(
        history.iter().map(|v| v.state.as_str()).collect::<Vec<_>>(),
        vec!["expired", "succeeded"]
    );
    store.close().await;
    Ok(())
}

#[tokio::test]
async fn lease_expiration_is_a_hard_commit_boundary() -> TestResult {
    let store = JobStore::open(":memory:").await?;
    store.submit(spec("boundary", 1), 100).await?;
    let lease = store.claim("w", "t", 100, 10000).await?.ok_or("claim")?;
    assert!(matches!(
        store.complete(&lease, success(), 10100).await,
        Err(JobError::LostLease)
    ));
    assert!(matches!(
        store.heartbeat(&lease, 10100, 10000).await,
        Err(JobError::LostLease)
    ));
    assert!(store.claim("new", "new", 10100, 10000).await?.is_none());
    assert_eq!(store.get("boundary").await?.state, JobState::Failed);
    store.close().await;
    Ok(())
}

#[tokio::test]
async fn heartbeat_prevents_early_reclaim_and_never_shortens_lease() -> TestResult {
    let store = JobStore::open(":memory:").await?;
    store.submit(spec("heart", 2), 100).await?;
    let lease = store.claim("w", "t", 100, 20000).await?.ok_or("claim")?;
    assert!(!store.heartbeat(&lease, 1000, 10000).await?);
    assert_eq!(store.get("heart").await?.lease_until, Some(20100));
    assert!(!store.heartbeat(&lease, 20000, 10000).await?);
    assert!(store.claim("other", "other", 20100, 10000).await?.is_none());
    assert_eq!(store.get("heart").await?.lease_until, Some(30000));
    assert_eq!(store.history("heart").await?.len(), 1);
    store.close().await;
    Ok(())
}

#[tokio::test]
async fn queued_cancellation_never_runs_and_running_cancellation_wins_completion() -> TestResult {
    let store = JobStore::open(":memory:").await?;
    store.submit(spec("queued", 2), 100).await?;
    assert_eq!(
        store.cancel("queued", 101).await?.state,
        JobState::Cancelled
    );
    assert!(store.claim("w", "t", 102, 10000).await?.is_none());
    store.submit(spec("running", 2), 100).await?;
    let lease = store.claim("w", "run", 102, 10000).await?.ok_or("claim")?;
    assert_eq!(store.cancel("running", 103).await?.state, JobState::Running);
    assert!(store.heartbeat(&lease, 104, 10000).await?);
    let final_job = store.complete(&lease, success(), 105).await?;
    assert_eq!(final_job.state, JobState::Cancelled);
    assert_eq!(
        store
            .history("running")
            .await?
            .first()
            .map(|v| v.state.as_str()),
        Some("cancelled")
    );
    store.close().await;
    Ok(())
}

#[tokio::test]
async fn expired_cancelled_work_is_not_requeued() -> TestResult {
    let store = JobStore::open(":memory:").await?;
    store.submit(spec("stop", 3), 100).await?;
    let lease = store.claim("w", "t", 100, 10000).await?.ok_or("claim")?;
    store.cancel("stop", 101).await?;
    assert!(store.claim("next", "next", 10100, 10000).await?.is_none());
    assert_eq!(store.get("stop").await?.state, JobState::Cancelled);
    assert!(matches!(
        store.complete(&lease, success(), 10101).await,
        Err(JobError::LostLease)
    ));
    store.close().await;
    Ok(())
}

#[tokio::test]
async fn restart_preserves_claim_and_recovers_only_after_expiration() -> TestResult {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("restart.sqlite");
    let first = JobStore::open(&path).await?;
    first.submit(spec("persist", 2), 100).await?;
    let lease = first
        .claim("before", "token", 100, 10000)
        .await?
        .ok_or("first claim")?;
    first.close().await;
    let reopened = JobStore::open(&path).await?;
    assert_eq!(reopened.get("persist").await?.state, JobState::Running);
    assert!(
        reopened
            .claim("after", "new", 10099, 10000)
            .await?
            .is_none()
    );
    let next = reopened
        .claim("after", "new", 10100, 10000)
        .await?
        .ok_or("reclaim")?;
    assert_eq!(next.job.attempt, 2);
    assert!(matches!(
        reopened.complete(&lease, success(), 10101).await,
        Err(JobError::LostLease)
    ));
    reopened.complete(&next, success(), 10101).await?;
    reopened.close().await;
    Ok(())
}

#[tokio::test]
async fn explicit_retry_retains_attempt_history_and_has_a_total_cap() -> TestResult {
    let store = JobStore::open(":memory:").await?;
    store.submit(spec("retry", 1), 100).await?;
    let lease = store.claim("w", "t", 100, 10000).await?.ok_or("claim")?;
    store
        .complete(
            &lease,
            Completion {
                state: JobState::Partial,
                result: Some(json!({"status":"partial"})),
                error: Some("denied scope".into()),
            },
            101,
        )
        .await?;
    assert!(store.claim("w", "t2", 102, 10000).await?.is_none());
    let queued = store.retry("retry", 2, 102).await?;
    assert_eq!(queued.max_attempts, 3);
    assert_eq!(queued.attempt, 1);
    let next = store
        .claim("w", "t2", 102, 10000)
        .await?
        .ok_or("retry claim")?;
    assert_eq!(next.job.attempt, 2);
    store
        .complete(
            &next,
            Completion {
                state: JobState::Failed,
                result: None,
                error: Some("failed".into()),
            },
            103,
        )
        .await?;
    assert!(matches!(
        store.retry("retry", 31, 104).await,
        Err(JobError::Limit(_))
    ));
    assert_eq!(store.history("retry").await?.len(), 2);
    store.close().await;
    Ok(())
}

#[tokio::test]
async fn successful_jobs_cannot_accidentally_be_reexecuted() -> TestResult {
    let store = JobStore::open(":memory:").await?;
    store.submit(spec("done", 3), 100).await?;
    let lease = store.claim("w", "t", 100, 10000).await?.ok_or("claim")?;
    store.complete(&lease, success(), 101).await?;
    assert!(store.retry("done", 1, 102).await.is_err());
    assert_eq!(store.cancel("done", 103).await?.state, JobState::Succeeded);
    assert!(store.claim("w", "next", 104, 10000).await?.is_none());
    store.close().await;
    Ok(())
}

#[tokio::test]
async fn listing_excludes_large_payloads_and_results() -> TestResult {
    let store = JobStore::open(":memory:").await?;
    store.submit(spec("visible", 1), 100).await?;
    let lease = store
        .claim("w", "secret-lease-token", 100, 10000)
        .await?
        .ok_or("claim")?;
    store.complete(&lease, success(), 101).await?;
    let full = store.get("visible").await?;
    assert!(full.payload.is_some() && full.result.is_some());
    let list = store.list(1).await?;
    let row = list.first().ok_or("list row")?;
    assert!(row.payload.is_none() && row.result.is_none());
    assert!(!serde_json::to_string(&full)?.contains("secret-lease-token"));
    store.close().await;
    Ok(())
}

#[tokio::test]
async fn invalid_and_oversized_inputs_do_not_mutate_queue() -> TestResult {
    let store = JobStore::open(":memory:").await?;
    assert!(store.submit(spec("", 3), 100).await.is_err());
    assert!(store.submit(spec("zero", 0), 100).await.is_err());
    assert!(store.submit(spec("many", 33), 100).await.is_err());
    assert!(store.submit(spec("negative", 3), -1).await.is_err());
    let mut big = spec("big", 3);
    big.payload = json!({"oversize":"x".repeat(65536)});
    assert!(store.submit(big, 100).await.is_err());
    assert!(store.claim("w", "t", i64::MAX, 10000).await.is_err());
    assert!(store.claim("w", "t", 100, 9999).await.is_err());
    assert!(store.list(0).await.is_err());
    assert!(store.list(1001).await.is_err());
    assert!(store.list(100).await?.is_empty());
    store.close().await;
    Ok(())
}

#[tokio::test]
async fn rejected_completion_preserves_live_lease() -> TestResult {
    let store = JobStore::open(":memory:").await?;
    store.submit(spec("rollback", 1), 100).await?;
    let lease = store.claim("w", "t", 100, 10000).await?.ok_or("claim")?;
    assert!(
        store
            .complete(
                &lease,
                Completion {
                    state: JobState::Queued,
                    result: None,
                    error: None
                },
                101
            )
            .await
            .is_err()
    );
    assert!(
        store
            .complete(
                &lease,
                Completion {
                    state: JobState::Succeeded,
                    result: Some(json!({"too_big":"x".repeat(1048576)})),
                    error: None
                },
                101
            )
            .await
            .is_err()
    );
    assert_eq!(store.get("rollback").await?.state, JobState::Running);
    assert_eq!(
        store
            .history("rollback")
            .await?
            .first()
            .map(|v| v.state.as_str()),
        Some("running")
    );
    store.complete(&lease, success(), 102).await?;
    store.close().await;
    Ok(())
}

#[tokio::test]
async fn errors_are_utf8_bounded_and_terminal_failures_are_explicit() -> TestResult {
    let store = JobStore::open(":memory:").await?;
    store.submit(spec("error", 1), 100).await?;
    let lease = store.claim("w", "t", 100, 10000).await?.ok_or("claim")?;
    let done = store
        .complete(
            &lease,
            Completion {
                state: JobState::Failed,
                result: None,
                error: Some("🦀".repeat(9000)),
            },
            101,
        )
        .await?;
    assert_eq!(done.last_error.as_ref().map(String::len), Some(8192));
    assert_eq!(done.state, JobState::Failed);
    store.close().await;
    Ok(())
}

#[tokio::test]
async fn queue_and_inventory_database_paths_are_not_interchangeable() -> TestResult {
    let folder = tempfile::tempdir()?;
    let inventory = folder.path().join("inventory.sqlite");
    let queue = folder.path().join("jobs.sqlite");
    Store::open(&inventory).await?.close().await;
    assert!(matches!(
        JobStore::open(&inventory).await,
        Err(JobError::ForeignDatabase)
    ));
    JobStore::open(&queue).await?.close().await;
    assert!(Store::open(&queue).await.is_err());
    let reopened = Store::open(&inventory).await?;
    reopened.close().await;
    let reopened = JobStore::open(&queue).await?;
    reopened.close().await;
    Ok(())
}
