mod process;

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use super::{
    ChildOwner, Emulator, HarnessError, READY_ATTEMPTS, READY_POLL, READY_TIMEOUT, await_endpoint,
    emulator_from, poll_endpoint,
};
use process::{assert_reaped, await_zombie, capture};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn sleep_child(duration: &str) -> std::io::Result<ChildOwner> {
    let child = Command::new("/bin/sleep").arg(duration).spawn()?;
    Ok(ChildOwner { child })
}

fn emulator(child: ChildOwner) -> Emulator {
    Emulator {
        child,
        endpoint: "https://127.0.0.1:12345".to_owned(),
        credentials: PathBuf::new(),
        config: PathBuf::new(),
        ca_bundle: PathBuf::new(),
    }
}

fn failed_readiness(mut child: ChildOwner, log: &Path) -> Result<String, HarnessError> {
    await_endpoint(log, &mut child, READY_TIMEOUT, READY_POLL, READY_ATTEMPTS)
}

#[test]
fn given_live_child_when_guard_drops_then_process_is_killed_and_reaped() -> TestResult {
    let owner = sleep_child("60")?;
    let identity = capture(&owner)?;
    drop(owner);
    assert_reaped(identity)
}

#[test]
fn given_live_child_when_endpoint_resolution_fails_then_process_is_reaped() -> TestResult {
    let owner = sleep_child("60")?;
    let identity = capture(&owner)?;
    let result = emulator_from(
        owner,
        "invalid endpoint".to_owned(),
        Path::new("/nonexistent"),
    );
    assert!(matches!(result, Err(HarnessError::Invalid(reason))
        if reason == "emulator endpoint is not HTTPS loopback with a port"));
    assert_reaped(identity)
}

#[test]
fn given_live_child_when_readiness_fails_then_same_guard_reaps_process() -> TestResult {
    let directory = tempfile::tempdir()?;
    let owner = sleep_child("60")?;
    let identity = capture(&owner)?;
    let missing = directory.path().join("missing.jsonl");
    let result = failed_readiness(owner, &missing);
    assert!(matches!(result, Err(HarnessError::Invalid(reason))
        if reason.starts_with(&missing.display().to_string())));
    assert_reaped(identity)
}

#[test]
fn given_ready_log_when_endpoint_is_found_then_guard_still_owns_child() -> TestResult {
    let directory = tempfile::tempdir()?;
    let log = directory.path().join("emulator.jsonl");
    std::fs::write(
        &log,
        b"{\"fields\":{\"endpoint_url\":\"https://127.0.0.1:12345\"}}\n",
    )?;
    let mut owner = sleep_child("60")?;
    let identity = capture(&owner)?;
    let endpoint = await_endpoint(&log, &mut owner, READY_TIMEOUT, READY_POLL, READY_ATTEMPTS)?;
    assert_eq!(endpoint, "https://127.0.0.1:12345");
    assert_eq!(owner.child.try_wait()?, None);
    drop(owner);
    assert_reaped(identity)
}

#[test]
fn given_running_child_when_emulator_stops_then_process_is_killed_and_reaped() -> TestResult {
    let owner = sleep_child("60")?;
    let identity = capture(&owner)?;
    emulator(owner).stop()?;
    assert_reaped(identity)
}

#[test]
fn given_unreaped_exited_child_when_emulator_stops_then_stop_succeeds() -> TestResult {
    let owner = ChildOwner {
        child: Command::new("/bin/true").spawn()?,
    };
    let identity = capture(&owner)?;
    await_zombie(identity)?;
    emulator(owner).stop()?;
    assert_reaped(identity)
}

#[test]
fn given_running_emulator_when_dropped_then_process_is_killed_and_reaped() -> TestResult {
    let owner = sleep_child("60")?;
    let identity = capture(&owner)?;
    drop(emulator(owner));
    assert_reaped(identity)
}

#[test]
fn given_unready_log_when_attempts_exhaust_then_child_is_reaped() -> TestResult {
    let directory = tempfile::tempdir()?;
    let log = directory.path().join("unready.jsonl");
    std::fs::write(&log, b"{}\n")?;
    let owner = sleep_child("60")?;
    let identity = capture(&owner)?;
    let result = bounded_readiness(owner, &log, Duration::from_secs(60), Duration::ZERO, 2);
    assert!(matches!(result, Err(HarnessError::Invalid(reason))
        if reason == "emulator readiness attempt budget exhausted"));
    assert_reaped(identity)
}

#[test]
fn given_unready_log_when_time_exhausts_then_child_is_reaped() -> TestResult {
    let directory = tempfile::tempdir()?;
    let log = directory.path().join("unready.jsonl");
    std::fs::write(&log, b"{}\n")?;
    let owner = sleep_child("60")?;
    let identity = capture(&owner)?;
    let result = bounded_readiness(
        owner,
        &log,
        Duration::from_millis(1),
        Duration::from_millis(1),
        8,
    );
    assert!(matches!(result, Err(HarnessError::Invalid(reason))
        if reason == "emulator readiness time budget exhausted"));
    assert_reaped(identity)
}

fn bounded_readiness(
    mut child: ChildOwner,
    log: &Path,
    timeout: Duration,
    poll: Duration,
    attempts: usize,
) -> Result<String, HarnessError> {
    await_endpoint(log, &mut child, timeout, poll, attempts)
}

#[test]
fn given_oversized_log_when_readiness_is_polled_then_child_is_reaped() -> TestResult {
    let directory = tempfile::tempdir()?;
    let log = directory.path().join("large.jsonl");
    let length = u64::try_from(crate::snapshot::MAX_DOCUMENT_BYTES)?
        .checked_add(1)
        .ok_or("log length overflow")?;
    std::fs::File::create(&log)?.set_len(length)?;
    let owner = sleep_child("60")?;
    let identity = capture(&owner)?;
    let result = failed_readiness(owner, &log);
    assert!(matches!(result, Err(HarnessError::Invalid(reason))
        if reason == format!("input byte budget exceeded: {}", log.display())));
    assert_reaped(identity)
}

#[test]
fn given_ready_line_and_partial_utf8_tail_when_polled_then_endpoint_is_ready() -> TestResult {
    let directory = tempfile::tempdir()?;
    let log = directory.path().join("partial.jsonl");
    std::fs::write(
        &log,
        b"{\"fields\":{\"endpoint_url\":\"https://127.0.0.1:12345\"}}\n\xc3",
    )?;
    let mut owner = sleep_child("60")?;
    let identity = capture(&owner)?;
    assert_eq!(
        poll_endpoint(&log, &mut owner)?,
        Some("https://127.0.0.1:12345".to_owned())
    );
    drop(owner);
    assert_reaped(identity)
}

#[test]
fn given_unterminated_endpoint_when_newline_arrives_then_endpoint_becomes_ready() -> TestResult {
    use std::io::Write;
    let directory = tempfile::tempdir()?;
    let log = directory.path().join("append.jsonl");
    std::fs::write(
        &log,
        b"{\"fields\":{\"endpoint_url\":\"https://127.0.0.1:12345\"}}",
    )?;
    let mut owner = sleep_child("60")?;
    let identity = capture(&owner)?;
    assert_eq!(poll_endpoint(&log, &mut owner)?, None);
    std::fs::OpenOptions::new()
        .append(true)
        .open(&log)?
        .write_all(b"\n")?;
    assert_eq!(
        poll_endpoint(&log, &mut owner)?,
        Some("https://127.0.0.1:12345".to_owned())
    );
    drop(owner);
    assert_reaped(identity)
}

#[test]
fn given_ambient_environment_when_child_runs_then_only_allowed_variables_reach_it() -> TestResult {
    let output = super::command(Path::new("/usr/bin/env")).output()?;
    assert!(output.status.success());
    let environment = std::str::from_utf8(&output.stdout)?;
    let variables: std::collections::BTreeSet<_> = environment.lines().collect();
    assert_eq!(
        variables,
        std::collections::BTreeSet::from(["EMULATOR_TELEMETRY=off", "PATH=/usr/bin:/bin",]),
    );
    Ok(())
}
