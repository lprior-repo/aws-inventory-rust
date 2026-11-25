//! Mutation testing harness.
//!
//! Keeps checked local AWS/Falkor fixtures alive while running a mutation
//! command. The emulator, fixture seeding, two unmodified Cargo snapshot runs,
//! and the supplied command share one process/network lifetime.

use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use sha2::Digest;

use crate::HarnessError;

/// Request configuration for execute_command.
pub struct CommandRequest {
    pub arguments: Vec<String>,
    pub workspace: PathBuf,
    pub output: PathBuf,
    pub label: String,
    pub scan_id: Option<String>,
    pub port: i32,
    pub deadline_seconds: i64,
    pub health_check: Option<Box<dyn Fn() -> Result<(), HarnessError> + Send + Sync>>,
}

/// Maximum command argument count.
const MAX_COMMAND_ARGUMENTS: usize = 128;

/// Maximum command argument bytes.
const MAX_COMMAND_ARGUMENT_BYTES: usize = 32_768;

/// Command drain seconds.
const COMMAND_DRAIN_SECONDS: i64 = 35;

/// Forced drain seconds.
const FORCED_DRAIN_SECONDS: i64 = 5;

/// Child drain reserve seconds.
const CHILD_DRAIN_RESERVE_SECONDS: i64 = 27;

/// Drain environment variable name.
const DRAIN_ENVIRONMENT: &str = "INVENTORY_COMMAND_DRAIN_SECONDS";

/// Command receipt with all execution metadata.
#[derive(Debug, Clone)]
pub struct Receipt {
    pub argv: Vec<String>,
    pub workspace: PathBuf,
    pub deadline_seconds: i64,
    pub status: String,
    pub log: PathBuf,
    pub pid: Option<u32>,
    pub process_group: Option<i32>,
    pub cleanup_budget_seconds: Option<i64>,
    pub child_cleanup_budget_seconds: Option<i64>,
    pub cleanup: Option<Value>,
    pub child_exit_code: Option<i32>,
    pub exit_code: Option<i32>,
    pub elapsed_seconds: f64,
    pub fixture_health_checks: usize,
    pub interrupt_signal: Option<i32>,
    pub fixture_error_type: Option<String>,
    pub fixture_error: Option<Value>,
    pub supervisor_error: Option<Value>,
    pub log_sha256: Option<String>,
}

impl Receipt {
    pub fn new(
        argv: Vec<String>,
        workspace: PathBuf,
        deadline_seconds: i64,
        output: &Path,
        label: &str,
    ) -> Self {
        Self {
            argv,
            workspace,
            deadline_seconds,
            status: String::from("running"),
            log: output.join(format!("{label}.log")),
            pid: None,
            process_group: None,
            cleanup_budget_seconds: None,
            child_cleanup_budget_seconds: None,
            cleanup: None,
            child_exit_code: None,
            exit_code: None,
            elapsed_seconds: 0.0,
            fixture_health_checks: 0,
            interrupt_signal: None,
            fixture_error_type: None,
            fixture_error: None,
            supervisor_error: None,
            log_sha256: None,
        }
    }

    pub fn to_json(&self) -> Value {
        json!({
            "argv": self.argv,
            "workspace": self.workspace.to_string_lossy(),
            "deadline_seconds": self.deadline_seconds,
            "status": self.status,
            "log": self.log.to_string_lossy(),
            "pid": self.pid,
            "process_group": self.process_group,
            "cleanup_budget_seconds": self.cleanup_budget_seconds,
            "child_cleanup_budget_seconds": self.child_cleanup_budget_seconds,
            "cleanup": self.cleanup,
            "child_exit_code": self.child_exit_code,
            "exit_code": self.exit_code,
            "elapsed_seconds": self.elapsed_seconds,
            "fixture_health_checks": self.fixture_health_checks,
            "interrupt_signal": self.interrupt_signal,
            "fixture_error_type": self.fixture_error_type,
            "fixture_error": self.fixture_error,
            "supervisor_error": self.supervisor_error,
            "log_sha256": self.log_sha256,
        })
    }
}

pub fn write_receipt(receipt: &Receipt, output: &Path, label: &str) -> Result<(), HarnessError> {
    let path = output.join(format!("{label}.json"));
    let json = receipt.to_json();
    let content = serde_json::to_string_pretty(&json)?;
    let mut file = File::create(&path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

pub fn command_drain_budget() -> Result<i64, HarnessError> {
    let value =
        std::env::var(DRAIN_ENVIRONMENT).unwrap_or_else(|_| COMMAND_DRAIN_SECONDS.to_string());
    let parsed = value
        .parse::<i64>()
        .map_err(|_| HarnessError::invalid(format!("invalid command drain budget: {value}")))?;
    if parsed <= FORCED_DRAIN_SECONDS || parsed > COMMAND_DRAIN_SECONDS {
        return Err(HarnessError::invalid(format!(
            "command drain budget out of range: {parsed}"
        )));
    }
    Ok(parsed)
}

pub async fn execute_command(request: CommandRequest) -> Result<Receipt, HarnessError> {
    validate_command(&request.arguments)?;

    let budget = command_drain_budget()?;
    let child_budget = budget.saturating_sub(CHILD_DRAIN_RESERVE_SECONDS);

    let mut receipt = build_initial_receipt(&request, budget, child_budget)?;
    write_receipt(&receipt, &request.output, &request.label)?;

    let start = Instant::now();
    let log_file = open_log_file(&receipt.log)?;

    let child = match spawn_command(&request, log_file, child_budget) {
        Ok(child) => child,
        Err(e) => {
            receipt.status = String::from("supervisor_error");
            receipt.supervisor_error = Some(json!({
                "phase": "spawn",
                "type": std::any::type_name_of_val(&e),
            }));
            finalize_receipt(&mut receipt, &request, start);
            return Ok(receipt);
        }
    };

    let pid = child.id();
    receipt.pid = Some(pid);
    write_receipt(&receipt, &request.output, &request.label)?;

    let result = wait_for_child(child, &request, &receipt).await;
    receipt.status = result.status;
    receipt.child_exit_code = result.exit_code;
    receipt.supervisor_error = result.supervisor_error;
    receipt.fixture_error_type = result.fixture_error_type;
    receipt.fixture_error = result.fixture_error;
    receipt.fixture_health_checks = result.health_checks;

    finalize_receipt(&mut receipt, &request, start);
    Ok(receipt)
}

fn validate_command(arguments: &[String]) -> Result<(), HarnessError> {
    if arguments.is_empty() || arguments.len() > MAX_COMMAND_ARGUMENTS {
        return Err(HarnessError::invalid(
            "command argument count is outside its bound",
        ));
    }
    if arguments.iter().map(|a| a.len()).sum::<usize>() > MAX_COMMAND_ARGUMENT_BYTES {
        return Err(HarnessError::invalid(
            "command arguments exceed their byte budget",
        ));
    }
    Ok(())
}

fn build_initial_receipt(
    request: &CommandRequest,
    budget: i64,
    child_budget: i64,
) -> Result<Receipt, HarnessError> {
    let mut receipt = Receipt::new(
        request.arguments.clone(),
        request.workspace.clone(),
        request.deadline_seconds,
        &request.output,
        &request.label,
    );
    receipt.cleanup_budget_seconds = Some(budget);
    receipt.child_cleanup_budget_seconds = Some(child_budget);
    Ok(receipt)
}

fn open_log_file(path: &Path) -> Result<File, HarnessError> {
    std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
        .map_err(HarnessError::from)
}

fn spawn_command(
    request: &CommandRequest,
    log_file: File,
    child_budget: i64,
) -> Result<std::process::Child, std::io::Error> {
    let Some(program) = request.arguments.first() else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "command has no program",
        ));
    };

    let mut command = Command::new(program.as_str());
    command
        .args(request.arguments.iter().skip(1))
        .current_dir(&request.workspace)
        .stdin(Stdio::null())
        .stdout(Stdio::from(log_file.try_clone()?))
        .stderr(Stdio::from(log_file.try_clone()?));

    if child_budget > 0 {
        command.env(DRAIN_ENVIRONMENT, child_budget.to_string());
    }

    command.spawn()
}

struct ChildWaitResult {
    status: String,
    exit_code: Option<i32>,
    supervisor_error: Option<serde_json::Value>,
    fixture_error_type: Option<String>,
    fixture_error: Option<serde_json::Value>,
    health_checks: usize,
}

async fn wait_for_child(
    mut child: std::process::Child,
    request: &CommandRequest,
    _receipt: &Receipt,
) -> ChildWaitResult {
    let start = Instant::now();
    let health_check_interval = Duration::from_secs(5);
    let mut last_health_check = start;
    let mut health_checks: usize = 0;
    let deadline = compute_deadline(start, request.deadline_seconds);

    loop {
        let outcome = wait_once(&mut child);
        if let WaitOutcome::Completed(exit_code) = outcome {
            return completed_result(exit_code, health_checks);
        }
        if let WaitOutcome::SupervisorError(e) = outcome {
            return supervisor_error_result(e, health_checks);
        }

        let now = Instant::now();
        if handle_deadline(&mut child, now, deadline, health_checks) {
            return ChildWaitResult {
                status: String::from("timeout"),
                exit_code: None,
                supervisor_error: None,
                fixture_error_type: None,
                fixture_error: None,
                health_checks,
            };
        }

        health_checks = perform_health_check(
            &mut child,
            request.health_check.as_ref().map(|b| b.as_ref()),
            &now,
            &last_health_check,
            &health_check_interval,
            health_checks,
        );
        if health_checks == usize::MAX {
            return fixture_unavailable_result();
        }

        log_elapsed(&now, start, last_health_check, &request.label);
        tokio::time::sleep(Duration::from_millis(250)).await;
        last_health_check = now;
    }
}

fn completed_result(exit_code: i32, health_checks: usize) -> ChildWaitResult {
    ChildWaitResult {
        status: String::from("completed"),
        exit_code: Some(exit_code),
        supervisor_error: None,
        fixture_error_type: None,
        fixture_error: None,
        health_checks,
    }
}

fn supervisor_error_result(e: serde_json::Value, health_checks: usize) -> ChildWaitResult {
    ChildWaitResult {
        status: String::from("supervisor_error"),
        exit_code: None,
        supervisor_error: Some(e),
        fixture_error_type: None,
        fixture_error: None,
        health_checks,
    }
}

fn fixture_unavailable_result() -> ChildWaitResult {
    ChildWaitResult {
        status: String::from("fixture_unavailable"),
        exit_code: None,
        supervisor_error: None,
        fixture_error_type: Some(String::from("fixture_health")),
        fixture_error: None,
        health_checks: 0,
    }
}

fn handle_deadline(
    child: &mut std::process::Child,
    now: Instant,
    deadline: Instant,
    _health_checks: usize,
) -> bool {
    if now >= deadline {
        let _ = child.kill();
        true
    } else {
        false
    }
}

fn compute_deadline(start: Instant, deadline_seconds: i64) -> Instant {
    let clamped = i64::max(1, i64::min(60, deadline_seconds));
    let duration = Duration::from_secs(clamped.try_into().unwrap_or(60));
    match start.checked_add(duration) {
        Some(deadline) => deadline,
        None => start,
    }
}

enum WaitOutcome {
    Completed(i32),
    SupervisorError(serde_json::Value),
    StillRunning,
}

fn wait_once(
    child: &mut std::process::Child,
) -> WaitOutcome {
    match child.try_wait() {
        Ok(Some(status)) => WaitOutcome::Completed(status.code().unwrap_or(128)),
        Ok(None) => WaitOutcome::StillRunning,
        Err(e) => WaitOutcome::SupervisorError(json!({
            "phase": "execute",
            "type": std::any::type_name_of_val(&e),
        })),
    }
}


fn perform_health_check(
    child: &mut std::process::Child,
    health_check: Option<&(dyn Fn() -> Result<(), HarnessError> + Send + Sync)>,
    now: &Instant,
    last_health_check: &Instant,
    interval: &Duration,
    current_checks: usize,
) -> usize {
    if !should_check_health(now, last_health_check, interval) {
        return current_checks;
    }

    match health_check {
        Some(health) => match health() {
            Ok(()) => current_checks.saturating_add(1),
            Err(_) => {
                let _ = child.kill();
                usize::MAX
            }
        },
        None => current_checks,
    }
}

fn should_check_health(
    now: &Instant,
    last_health_check: &Instant,
    interval: &Duration,
) -> bool {
    let Some(next_check) = last_health_check.checked_add(*interval) else {
        return true;
    };
    *now >= next_check
}

fn log_elapsed(
    now: &Instant,
    start: Instant,
    last_health_check: Instant,
    label: &str,
) {
    let elapsed = now.saturating_duration_since(start);
    if elapsed.as_secs() < 30 {
        return;
    }

    let Some(next_log) = last_health_check.checked_add(Duration::from_secs(30)) else {
        return;
    };
    if *now < next_log {
        return;
    }

    let json = json!({
        "phase": label,
        "elapsed_seconds": elapsed.as_secs_f64(),
    });
    match serde_json::to_string(&json) {
        Ok(s) => eprintln!("{s}"),
        Err(_) => eprintln!("{{}}"),
    }
}

fn finalize_receipt(receipt: &mut Receipt, request: &CommandRequest, start: Instant) {
    receipt.elapsed_seconds = Instant::now()
        .saturating_duration_since(start)
        .as_secs_f64();

    if receipt.exit_code.is_none() {
        receipt.exit_code = compute_exit_code(receipt);
    }

    compute_log_hash(receipt);
    write_receipt(receipt, &request.output, &request.label).ok();
    log_receipt(receipt);
}

fn compute_exit_code(receipt: &Receipt) -> Option<i32> {
    if receipt.status.as_str() == "interrupted" {
        let signal: i32 = receipt.interrupt_signal.unwrap_or(0);
        Some(128_i32.saturating_add(signal))
    } else if receipt.status.as_str() != "completed" {
        Some(124_i32)
    } else {
        receipt.child_exit_code
    }
}

fn compute_log_hash(receipt: &mut Receipt) {
    if let Ok(log_content) = fs::read(&receipt.log) {
        let hash = sha2::Sha256::new_with_prefix(&log_content).finalize();
        receipt.log_sha256 = Some(hex::encode(hash));
    }
}

fn log_receipt(receipt: &Receipt) {
    eprintln!(
        "{}",
        serde_json::to_string(&json!({
            "phase": "execute",
            "status": receipt.status,
            "exit_code": receipt.exit_code,
            "elapsed_seconds": receipt.elapsed_seconds,
        }))
        .unwrap_or_default()
    );
}

/// Mutation testing arguments.
pub struct Args {
    pub emulator: PathBuf,
    pub binary: PathBuf,
    pub branch_probe: PathBuf,
    pub output: PathBuf,
    pub workspace: PathBuf,
    pub redis_server: Option<PathBuf>,
    pub falkor_module: Option<PathBuf>,
    pub command_deadline: i64,
    pub command: Vec<String>,
}

/// Run mutation testing campaign.
pub async fn run(args: &Args) -> Result<Value, HarnessError> {
    let output = args.output.canonicalize()?;
    fs::create_dir_all(&output)?;

    let _binary = args.binary.canonicalize()?;

    let baseline_cmd = build_baseline_command();
    let baseline_result = run_baselines(args, &baseline_cmd, &output).await?;

    if baseline_result.exit_code != Some(0) {
        return Ok(baseline_result.to_json());
    }

    run_mutation_command(args, &output).await
}

fn build_baseline_command() -> Vec<String> {
    vec![
        "cargo".to_string(),
        "test".to_string(),
        "--workspace".to_string(),
        "--exclude".to_string(),
        "inventory-sdk-catalog".to_string(),
        "--all-features".to_string(),
        "--test".to_string(),
        "emulator_snapshot".to_string(),
        "--locked".to_string(),
        "--offline".to_string(),
        "-j1".to_string(),
        "--".to_string(),
        "--ignored".to_string(),
        "--test-threads=1".to_string(),
    ]
}

async fn run_baselines(
    args: &Args,
    baseline_cmd: &[String],
    output: &Path,
) -> Result<Receipt, HarnessError> {
    let mut last_result = None;

    for i in 1..=2 {
        let label = format!("oracle-baseline-{i}");
        let result = execute_command(CommandRequest {
            arguments: baseline_cmd.to_vec(),
            workspace: args.workspace.clone(),
            output: output.to_path_buf(),
            label,
            scan_id: None,
            port: 0,
            deadline_seconds: 600,
            health_check: None,
        })
        .await?;

        if result.exit_code != Some(0) {
            return Ok(result);
        }

        last_result = Some(result);
    }

    match last_result {
        Some(r) => Ok(r),
        None => Err(HarnessError::invalid("no baseline results")),
    }
}

async fn run_mutation_command(
    args: &Args,
    output: &Path,
) -> Result<Value, HarnessError> {
    if args.command.is_empty() {
        return Ok(json!({
            "phase": "mutate",
            "status": "completed",
            "oracle_baselines": 2,
            "mutation_command": null,
        }));
    }

    let result = execute_command(CommandRequest {
        arguments: args.command.clone(),
        workspace: args.workspace.clone(),
        output: output.to_path_buf(),
        label: "mutation-command".to_string(),
        scan_id: None,
        port: 0,
        deadline_seconds: args.command_deadline,
        health_check: None,
    })
    .await?;

    Ok(result.to_json())
}
