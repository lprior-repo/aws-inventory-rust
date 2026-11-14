use std::future::Future;
use std::time::Duration;

use inventory_engine::jobs::{Completion, JobError, JobSpec, JobState, JobStore, TerminalJobState};
use proptest::prelude::*;
use proptest::test_runner::{Config, RngSeed, TestCaseError, TestCaseResult};
use serde_json::{Value, json};
use tokio::runtime::Builder;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn failure(error: impl std::fmt::Display) -> TestCaseError {
    TestCaseError::fail(error.to_string())
}

fn intent(id: &str) -> JobSpec {
    JobSpec {
        job_id: id.into(),
        kind: "scan".into(),
        payload: json!({}),
        max_attempts: 2,
        available_at: 0,
    }
}

fn rejected_input<T>(result: Result<T, JobError>) -> bool {
    matches!(result, Err(JobError::Invalid(_) | JobError::Validation(_)))
}

async fn bounded_case(case: impl Future<Output = TestCaseResult>) -> TestCaseResult {
    tokio::time::timeout(Duration::from_secs(10), case)
        .await
        .map_err(|_| failure("durable job property exceeded its 10-second case deadline"))?
}

fn run_case(case: impl Future<Output = TestCaseResult>) -> TestCaseResult {
    Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(failure)?
        .block_on(bounded_case(case))
}

fn non_object(kind: u8, value: u16) -> Value {
    match kind {
        0 => Value::Null,
        1 => json!(true),
        2 => json!(value),
        3 => json!([value]),
        _ => json!(format!("value-{value}")),
    }
}

async fn check_payload(store: &JobStore, kind: u8, value: u16) -> TestCaseResult {
    let mut invalid = intent("shape");
    invalid.payload = non_object(kind, value);
    prop_assert!(rejected_input(store.submit(invalid, 0).await));
    prop_assert!(store.list(1).await.map_err(failure)?.is_empty());
    let mut valid = intent("shape");
    valid.payload = json!({"value": value});
    let accepted = store.submit(valid, 0).await.map_err(failure)?;
    prop_assert_eq!(accepted.payload, Some(json!({"value": value})));
    prop_assert_eq!(accepted.state, JobState::Queued);
    prop_assert_eq!(accepted.attempt, 0);
    Ok(())
}

async fn payload_case(kind: u8, value: u16) -> TestCaseResult {
    let store = JobStore::open(":memory:").await.map_err(failure)?;
    let result = check_payload(&store, kind, value).await;
    store.close().await;
    result
}

fn text_with_bytes(bytes: usize, prefix: &str) -> String {
    format!("{prefix}{}", "x".repeat(bytes.saturating_sub(prefix.len())))
}

async fn check_byte_boundaries(store: &JobStore, prefix: &str, value: u16) -> TestCaseResult {
    let mut accepted = 0_usize;

    for bytes in [63_usize, 64, 65, 66] {
        let mut spec = intent(&format!("kind-{bytes}-{value}"));
        spec.kind = text_with_bytes(bytes, prefix);
        let result = store.submit(spec.clone(), 0).await;
        if bytes <= 64 {
            let record = result.map_err(failure)?;
            prop_assert_eq!(record.kind, spec.kind);
            accepted = accepted.saturating_add(1);
        } else {
            prop_assert!(rejected_input(result));
        }
        prop_assert_eq!(store.list(16).await.map_err(failure)?.len(), accepted);
    }
    for bytes in [255_usize, 256, 257, 258] {
        let id = text_with_bytes(bytes, prefix);
        let result = store.submit(intent(&id), 0).await;
        if bytes <= 256 {
            let record = result.map_err(failure)?;
            prop_assert_eq!(record.job_id, id);
            accepted = accepted.saturating_add(1);
        } else {
            prop_assert!(rejected_input(result));
        }
        prop_assert_eq!(store.list(16).await.map_err(failure)?.len(), accepted);
    }
    Ok(())
}

async fn byte_boundary_case(prefix: &str, value: u16) -> TestCaseResult {
    let store = JobStore::open(":memory:").await.map_err(failure)?;
    let result = check_byte_boundaries(&store, prefix, value).await;
    store.close().await;
    result
}

async fn snapshot(store: &JobStore, id: &str) -> Result<Value, JobError> {
    Ok(json!({
        "job": store.get(id).await?,
        "attempts": store.history(id).await?,
    }))
}

fn completion(state: TerminalJobState) -> Completion {
    Completion {
        state,
        result: None,
        error: None,
    }
}

async fn check_epoch_boundary(store: &JobStore, negative: i64) -> TestCaseResult {
    store.submit(intent("epoch"), 0).await.map_err(failure)?;
    let queued = snapshot(store, "epoch").await.map_err(failure)?;
    prop_assert!(rejected_input(
        store.claim("worker", "token", negative, 10_000).await
    ));
    prop_assert!(rejected_input(store.cancel("epoch", negative).await));
    prop_assert_eq!(snapshot(store, "epoch").await.map_err(failure)?, queued);

    let lease = store
        .claim("worker", "token", 0, 10_000)
        .await
        .map_err(failure)?
        .ok_or_else(|| failure("epoch-zero work was not claimable"))?;
    prop_assert!(!store.heartbeat(&lease, 0, 10_000).await.map_err(failure)?);
    let running = snapshot(store, "epoch").await.map_err(failure)?;
    prop_assert!(rejected_input(
        store.heartbeat(&lease, negative, 10_000).await
    ));
    prop_assert!(rejected_input(
        store
            .complete(&lease, completion(TerminalJobState::Succeeded), negative)
            .await
    ));
    prop_assert_eq!(snapshot(store, "epoch").await.map_err(failure)?, running);

    store
        .complete(&lease, completion(TerminalJobState::Failed), 0)
        .await
        .map_err(failure)?;
    let terminal = snapshot(store, "epoch").await.map_err(failure)?;
    prop_assert!(rejected_input(store.retry("epoch", 1, negative).await));
    prop_assert_eq!(snapshot(store, "epoch").await.map_err(failure)?, terminal);
    prop_assert_eq!(
        store.retry("epoch", 1, 0).await.map_err(failure)?.state,
        JobState::Queued
    );
    prop_assert_eq!(
        store.cancel("epoch", 0).await.map_err(failure)?.state,
        JobState::Cancelled
    );
    let history = store.history("epoch").await.map_err(failure)?;
    prop_assert_eq!(history.len(), 1);
    prop_assert_eq!(
        history.first().map(|attempt| attempt.finished_at),
        Some(Some(0))
    );
    Ok(())
}

async fn epoch_case(negative: i64) -> TestCaseResult {
    let store = JobStore::open(":memory:").await.map_err(failure)?;
    let result = check_epoch_boundary(&store, negative).await;
    store.close().await;
    result
}

proptest! {
    #![proptest_config(Config {
        cases: 24,
        max_shrink_iters: 96,
        max_shrink_time: 2_000,
        rng_seed: RngSeed::Fixed(0x4157_535f_4a56_414c),
        ..Config::default()
    })]

    #[test]
    fn valid_json_that_is_not_object_intent_never_enters_the_queue(
        kind in 0_u8..5,
        value in any::<u16>(),
    ) {
        run_case(payload_case(kind, value))?;
    }

    #[test]
    fn job_identifiers_and_kinds_obey_utf8_byte_boundaries(
        prefix in prop::sample::select(vec!["é", "界", "🦀"]),
        value in any::<u16>(),
    ) {
        run_case(byte_boundary_case(prefix, value))?;
    }

    #[test]
    fn epoch_zero_is_valid_and_negative_lifecycle_times_never_change_state(
        negative in -1_000_i64..0,
    ) {
        run_case(epoch_case(negative))?;
    }
}

#[tokio::test]
async fn invalid_worker_capabilities_do_not_claim_or_consume_attempts() -> TestResult {
    let store = JobStore::open(":memory:").await?;
    store.submit(intent("capability"), 0).await?;
    let before = snapshot(&store, "capability").await?;
    let invalid = [String::new(), "contains\0nul".into(), "🦀".repeat(65)];
    for value in invalid {
        assert!(rejected_input(
            store.claim(&value, "token", 0, 10_000).await
        ));
        assert_eq!(snapshot(&store, "capability").await?, before);
        assert!(rejected_input(
            store.claim("worker", &value, 0, 10_000).await
        ));
        assert_eq!(snapshot(&store, "capability").await?, before);
    }
    let boundary = "🦀".repeat(64);
    let lease = store
        .claim(&boundary, &boundary, 0, 10_000)
        .await?
        .ok_or("valid boundary capability was not accepted")?;
    assert_eq!(lease.job.attempt, 1);
    store
        .complete(&lease, completion(TerminalJobState::Succeeded), 0)
        .await?;
    store.close().await;
    Ok(())
}
