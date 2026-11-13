//! Small generated command sequences exercise SQLx state, ownership and restart.
//! The oracle is an append-only list of caller-visible attempts and outcomes.

use std::collections::BTreeSet;
use std::future::Future;
use std::time::Duration;

use futures_util::{StreamExt, stream};
use inventory_engine::jobs::{Completion, JobError, JobSpec, JobState, JobStore, Lease};
use proptest::prelude::*;
use proptest::test_runner::{Config, RngSeed, TestCaseError, TestCaseResult};
use serde_json::json;
use tokio::runtime::{Builder, Runtime};

const JOB: &str = "property-job";
const START: i64 = 100;

fn failure(error: impl std::fmt::Display) -> TestCaseError {
    TestCaseError::fail(error.to_string())
}

fn runtime() -> Result<Runtime, TestCaseError> {
    // Queue time is an explicit integer capability. SQLx worker I/O uses real
    // Tokio scheduling; paused time would race its connection-acquire timers.
    Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(failure)
}

async fn bounded_case(case: impl Future<Output = TestCaseResult>) -> TestCaseResult {
    tokio::time::timeout(Duration::from_secs(10), case)
        .await
        .map_err(|_| failure("durable job property exceeded its 10-second case deadline"))?
}

fn spec(id: &str, budget: u32, value: u16) -> JobSpec {
    JobSpec {
        job_id: id.into(),
        kind: "synthetic_scan".into(),
        payload: json!({"value": value}),
        max_attempts: budget,
        available_at: START,
    }
}

fn completion(state: JobState, value: u16) -> Result<Completion, JobError> {
    Ok(Completion {
        state: state.try_into()?,
        result: Some(json!({"value": value, "reported": state})),
        error: (state == JobState::Failed).then(|| "synthetic failure".into()),
    })
}

fn terminal_state(value: u8) -> JobState {
    match value {
        0 => JobState::Succeeded,
        1 => JobState::Partial,
        2 => JobState::Failed,
        _ => JobState::Cancelled,
    }
}

proptest! {
    #![proptest_config(Config {
        cases: 24,
        max_shrink_iters: 96,
        max_shrink_time: 2_000,
        rng_seed: RngSeed::Fixed(0x4157_535f_4a4f_4253),
        ..Config::default()
    })]

    #[test]
    fn generated_heartbeat_sequences_preserve_fencing_across_reopen(
        budget in 1_u32..=4,
        initial_ms in 10_000_i64..=60_000,
        heartbeats in prop::collection::vec((0_u16..=30_000, 0_u16..=50_000), 0..=8),
        value in any::<u16>(),
    ) {
        runtime()?.block_on(bounded_case(lease_reopen_case(budget, initial_ms, &heartbeats, value)))?;
    }

    #[test]
    fn generated_terminal_commands_preserve_retry_and_append_only_history(
        budget in 1_u32..=4,
        commands in prop::collection::vec(finish_command(), 1..=8),
        value in any::<u16>(),
    ) {
        runtime()?.block_on(bounded_case(command_case(budget, &commands, value)))?;
    }

    #[test]
    fn concurrent_claims_never_duplicate_or_run_future_work(
        ready in prop::collection::vec(any::<bool>(), 1..=6),
        value in any::<u16>(),
    ) {
        runtime()?.block_on(bounded_case(competing_case(&ready, value)))?;
    }
}
#[path = "job_properties/lease_recovery.rs"]
mod lease_recovery;
use lease_recovery::*;
#[path = "job_properties/terminal_workflow.rs"]
mod terminal_workflow;
use terminal_workflow::*;
#[path = "job_properties/competition.rs"]
mod competition;
use competition::*;
