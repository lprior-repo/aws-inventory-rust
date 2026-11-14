use inventory_engine::jobs::{Completion, JobError, JobSpec, JobState, JobStore, TerminalJobState};
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
        state: TerminalJobState::Succeeded,
        result: Some(json!({"status":"complete","synthetic":true})),
        error: None,
    }
}

#[path = "jobs/competing_database_connections_claim_only_once.rs"]
mod competing_database_connections_claim_only_once;
#[path = "jobs/errors_are_utf8_bounded_and_terminal_failures_are_explicit.rs"]
mod errors_are_utf8_bounded_and_terminal_failures_are_explicit;
#[path = "jobs/expired_attempt_is_recovered_and_old_owner_is_fenced.rs"]
mod expired_attempt_is_recovered_and_old_owner_is_fenced;
#[path = "jobs/expired_cancelled_work_is_not_requeued.rs"]
mod expired_cancelled_work_is_not_requeued;
#[path = "jobs/explicit_retry_retains_attempt_history_and_has_a_total_cap.rs"]
mod explicit_retry_retains_attempt_history_and_has_a_total_cap;
#[path = "jobs/heartbeat_prevents_early_reclaim_and_never_shortens_lease.rs"]
mod heartbeat_prevents_early_reclaim_and_never_shortens_lease;
#[path = "jobs/invalid_and_oversized_inputs_do_not_mutate_queue.rs"]
mod invalid_and_oversized_inputs_do_not_mutate_queue;
#[path = "jobs/lease_expiration_is_a_hard_commit_boundary.rs"]
mod lease_expiration_is_a_hard_commit_boundary;
#[path = "jobs/listing_excludes_large_payloads_and_results.rs"]
mod listing_excludes_large_payloads_and_results;
#[path = "jobs/queue_and_inventory_database_paths_are_not_interchangeable.rs"]
mod queue_and_inventory_database_paths_are_not_interchangeable;
#[path = "jobs/queued_cancellation_never_runs_and_running_cancellation_wins_completion.rs"]
mod queued_cancellation_never_runs_and_running_cancellation_wins_completion;
#[path = "jobs/rejected_completion_preserves_live_lease.rs"]
mod rejected_completion_preserves_live_lease;
#[path = "jobs/restart_preserves_claim_and_recovers_only_after_expiration.rs"]
mod restart_preserves_claim_and_recovers_only_after_expiration;
#[path = "jobs/schedule_and_deterministic_claim_order.rs"]
mod schedule_and_deterministic_claim_order;
#[path = "jobs/submission_is_idempotent_but_different_work_conflicts.rs"]
mod submission_is_idempotent_but_different_work_conflicts;
#[path = "jobs/successful_jobs_cannot_accidentally_be_reexecuted.rs"]
mod successful_jobs_cannot_accidentally_be_reexecuted;
