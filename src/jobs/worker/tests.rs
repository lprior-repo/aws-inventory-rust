// `serde_json::Value` string indexing is infallible; array positions use `first`.
#![allow(clippy::indexing_slicing)]

use super::*;
use crate::json_test::first;
use std::sync::atomic::{AtomicBool, Ordering};

fn executor() -> Result<Arc<ApiExecutor>> {
    Ok(Arc::new(ApiExecutor::new(ExecutorConfig::default())?))
}

#[tokio::test]
async fn stopped_worker_does_not_claim_queued_jobs_or_load_aws_configuration() -> Result<()> {
    let store = JobStore::open(":memory:").await?;
    for job_id in ["first", "second"] {
        store
            .submit(
                JobSpec {
                    job_id: job_id.into(),
                    kind: "aws_scan".into(),
                    payload: json!({"intentionally_invalid_scan":true}),
                    max_attempts: 1,
                    available_at: 0,
                },
                0,
            )
            .await?;
    }
    let shutdown = CancellationToken::new();
    shutdown.cancel();
    let report = run_queue(&store, 2, 10_000, "stopped", &shutdown).await?;
    assert_eq!(report["executed"], 0);
    assert_eq!(report["status"], "cancelled");
    assert_eq!(report["stop_reason"], "worker_shutdown");
    for job_id in ["first", "second"] {
        let job = store.get(job_id).await?;
        assert_eq!(job.state, JobState::Queued);
        assert_eq!(job.attempt, 0);
    }
    store.close().await;
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn worker_shutdown_bounds_an_unresponsive_scan_and_drops_it() -> Result<()> {
    struct DropMark(Arc<AtomicBool>);
    impl Drop for DropMark {
        fn drop(&mut self) {
            self.0.store(true, Ordering::SeqCst);
        }
    }
    let dropped = Arc::new(AtomicBool::new(false));
    let marker = DropMark(Arc::clone(&dropped));
    let scan = async move {
        let _marker = marker;
        std::future::pending::<Result<Value>>().await
    };
    let shutdown = CancellationToken::new();
    shutdown.cancel();
    let executor = executor()?;
    let start = tokio::time::Instant::now();
    let error = supervise_futures(
        Arc::clone(&executor),
        &shutdown,
        scan,
        std::future::pending(),
        Duration::from_secs(100),
        Duration::from_secs(2),
    )
    .await
    .err()
    .context("drain must time out")?;
    assert!(error.to_string().contains("drain deadline"));
    assert!(executor.is_cancelled());
    assert!(dropped.load(Ordering::SeqCst));
    assert_eq!(
        tokio::time::Instant::now().duration_since(start),
        Duration::from_secs(2)
    );
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn lease_loss_preserves_error_and_bounds_scan_drain() -> Result<()> {
    let executor = executor()?;
    let shutdown = CancellationToken::new();
    let error = supervise_futures(
        Arc::clone(&executor),
        &shutdown,
        std::future::pending(),
        std::future::ready(Err(anyhow::anyhow!("synthetic lost lease"))),
        Duration::from_secs(100),
        Duration::from_secs(2),
    )
    .await
    .err()
    .context("lost lease must fail")?;
    assert!(format!("{error:#}").contains("synthetic lost lease"));
    assert!(format!("{error:#}").contains("drain deadline"));
    assert!(executor.is_cancelled());
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn timeout_cancels_and_allows_a_cooperative_scan_to_finalize() -> Result<()> {
    let executor = executor()?;
    let scan_executor = Arc::clone(&executor);
    let scan = async move {
        scan_executor.cancelled().await;
        tokio::time::sleep(Duration::from_millis(20)).await;
        Ok(json!({"status":"cancelled","scans":[],"finalized":true}))
    };
    let result = supervise_futures(
        executor,
        &CancellationToken::new(),
        scan,
        std::future::pending(),
        Duration::from_secs(1),
        Duration::from_secs(2),
    )
    .await?;
    assert_eq!(result["worker_timeout"], true);
    assert_eq!(result["finalized"], true);
    assert_eq!(result["status"], "cancelled");
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn heartbeat_future_keeps_running_while_cancelled_work_drains() -> Result<()> {
    let executor = executor()?;
    let shutdown = CancellationToken::new();
    shutdown.cancel();
    let pulsed = Arc::new(AtomicBool::new(false));
    let pulsed_copy = Arc::clone(&pulsed);
    let pulse = async move {
        tokio::time::sleep(Duration::from_millis(10)).await;
        pulsed_copy.store(true, Ordering::SeqCst);
        std::future::pending::<Result<()>>().await
    };
    let scan = async {
        tokio::time::sleep(Duration::from_millis(20)).await;
        Ok(json!({"status":"cancelled"}))
    };
    let result = supervise_futures(
        executor,
        &shutdown,
        scan,
        pulse,
        Duration::from_secs(1),
        Duration::from_secs(2),
    )
    .await?;
    assert!(pulsed.load(Ordering::SeqCst));
    assert_eq!(result["status"], "cancelled");
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn completed_scan_does_not_wait_for_heartbeat_or_deadline() -> Result<()> {
    let executor = executor()?;
    let result = supervise_futures(
        Arc::clone(&executor),
        &CancellationToken::new(),
        std::future::ready(Ok(json!({"status":"complete"}))),
        std::future::pending(),
        Duration::from_secs(100),
        Duration::from_secs(2),
    )
    .await?;
    assert_eq!(result["status"], "complete");
    assert!(!executor.is_cancelled());
    Ok(())
}

#[test]
fn cancellation_wins_a_success_shaped_completion() {
    assert_eq!(
        completion_state(&json!({"status":"complete"}), Cancellation::Requested),
        TerminalJobState::Cancelled
    );
    assert_eq!(
        completion_state(&json!({"status":"complete"}), Cancellation::NotRequested),
        TerminalJobState::Succeeded
    );
}

#[test]
fn oversized_results_retain_bounded_scan_summaries_and_explicit_counts() -> Result<()> {
    let small = json!({"status":"complete","scans":[],"api":{"attempts":1}});
    assert_eq!(compact_result(small.clone()), small);
    let oversized = json!({"status":"partial","unneeded":"x".repeat(1_048_576),
    "scans":[{"scan_id":"one","status":"partial","summary":{
        "resources":12,"edges":3,"completed_scopes":2,"failed_scopes":1,
        "unexpected":"x".repeat(1_048_576)
    }}]});
    let compact = compact_result(oversized);
    inventory_core::validate_serialized_size(
        &compact,
        inventory_engine::jobs::MAX_RESULT_BYTES,
        "test result",
    )?;
    assert_eq!(compact["result_truncated"], true);
    assert_eq!(compact["scan_count"], 1);
    assert_eq!(first(&compact["scans"])["summary"]["resources"], 12);
    assert!(
        first(&compact["scans"])["summary"]
            .get("unexpected")
            .is_none()
    );
    assert!(compact.get("unneeded").is_none());

    let maximum = json!({"status":"partial","scans":
        (0..crate::targets::MAX_TARGET_ACCOUNTS).map(|_| json!({
            "scan_id":"\u{1}".repeat(256),"status":"partial","summary":{}
        })).collect::<Vec<_>>()});
    let fallback = compact_result(maximum);
    inventory_core::validate_serialized_size(
        &fallback,
        inventory_engine::jobs::MAX_RESULT_BYTES,
        "test fallback",
    )?;
    assert_eq!(fallback["summaries_omitted"], true);
    assert_eq!(fallback["scans"], json!([]));
    assert_eq!(fallback["scan_count"], crate::targets::MAX_TARGET_ACCOUNTS);
    Ok(())
}

#[test]
fn summaries_omit_full_results_bound_error_text_and_stop_on_cancellation() -> Result<()> {
    let job = JobRecord {
        job_id: "summary".into(),
        kind: "aws_scan".into(),
        state: JobState::Cancelled,
        attempt: 1,
        max_attempts: 2,
        available_at: 0,
        created_at: 0,
        updated_at: 1,
        lease_owner: None,
        lease_until: None,
        cancel_requested: true,
        last_error: Some("🦀".repeat(2048)),
        payload: None,
        result: Some(json!({"large":"x".repeat(1_000_000),"scans":[{"scan_id":"one"}]})),
    };
    let row = job_summary(&job);
    assert!(row.get("result").is_none());
    assert_eq!(row["scan_count"], 1);
    assert_eq!(row["error_truncated"], true);
    assert_eq!(row["error"].as_str().map(str::len), Some(1024));
    let mut outcomes = Outcomes::new(1)?;
    outcomes.record(row)?;
    assert_eq!(outcomes.stop_reason(), "cancelled_job");
    assert_eq!(outcomes.report("worker")?["status"], "cancelled");
    Ok(())
}

#[test]
fn worker_intake_transitions_preserve_each_explicit_stop_cause() -> Result<()> {
    let mut outcomes = Outcomes::new(1)?;
    assert!(outcomes.accepting());
    assert_eq!(outcomes.stop_reason(), "claim_limit");
    outcomes.stop(StopReason::QueueEmpty);
    assert!(!outcomes.accepting());
    assert_eq!(outcomes.stop_reason(), "queue_empty");
    outcomes.stop(StopReason::WorkerShutdown);
    assert!(!outcomes.accepting());
    assert_eq!(outcomes.report("stopped")?["status"], "cancelled");
    Ok(())
}
