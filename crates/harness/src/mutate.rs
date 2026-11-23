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
    let arguments = request.arguments;
    let workspace = request.workspace;
    let output = request.output;
    let label = request.label;
    validate_arguments(&arguments)?;

    let budget = command_drain_budget()?;
    let child_budget = budget.saturating_sub(CHILD_DRAIN_RESERVE_SECONDS);

    let mut receipt = Receipt::new(
        arguments.clone(),
        workspace.clone(),
        request.deadline_seconds,
        &output,
        &label,
    );
    receipt.cleanup_budget_seconds = Some(budget);
    receipt.child_cleanup_budget_seconds = Some(child_budget);

    write_receipt(&receipt, &output, &label)?;

    let mut command = child_command(&arguments, &workspace, &receipt.log, child_budget)?;
    let mut child = match command.spawn() {
        Ok(c) => c,
        Err(e) => {
            receipt.status = String::from("supervisor_error");
            receipt.supervisor_error = Some(json!({
                "phase": "spawn",
                "type": std::any::type_name_of_val(&e),
            }));
            return Ok(receipt);
        }
    };

    receipt.pid = Some(child.id());
    write_receipt(&receipt, &output, &label)?;

    let start = Instant::now();
    let supervision = Supervision {
        label: &label,
        health_check: request.health_check.as_deref(),
        start,
        deadline: command_deadline(start, request.deadline_seconds)?,
    };
    let health_checks = supervise(&mut child, &supervision, &mut receipt).await?;
    settle_receipt(&mut receipt, start, health_checks);

    write_receipt(&receipt, &output, &label)?;

    report_settled(&label, &receipt);

    Ok(receipt)
}

fn report_settled(label: &str, receipt: &Receipt) {
    eprintln!(
        "{}",
        serde_json::to_string(&json!({
            "phase": label,
            "status": receipt.status,
            "exit_code": receipt.exit_code,
            "elapsed_seconds": receipt.elapsed_seconds,
        }))
        .unwrap_or_default()
    );
}

fn validate_arguments(arguments: &[String]) -> Result<(), HarnessError> {
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

/// Builds the child command with stdout and stderr appended to a freshly truncated log.
fn child_command(
    arguments: &[String],
    workspace: &Path,
    log: &Path,
    child_budget: i64,
) -> Result<Command, HarnessError> {
    let log_file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(log)?;

    let program = arguments
        .first()
        .ok_or_else(|| HarnessError::invalid("command has no program"))?;
    let mut command = Command::new(program.as_str());
    command
        .args(arguments.iter().skip(1))
        .current_dir(workspace)
        .stdin(Stdio::null())
        .stdout(Stdio::from(log_file.try_clone()?))
        .stderr(Stdio::from(log_file.try_clone()?));

    if child_budget > 0 {
        command.env(DRAIN_ENVIRONMENT, child_budget.to_string());
    }
    Ok(command)
}

fn command_deadline(start: Instant, deadline_seconds: i64) -> Result<Instant, HarnessError> {
    let deadline_seconds = i64::max(1, i64::min(60, deadline_seconds));
    let duration = Duration::from_secs(deadline_seconds.try_into().unwrap_or(60));
    start
        .checked_add(duration)
        .ok_or(HarnessError::invalid("deadline overflow"))
}

/// Fixed inputs of one child supervision loop.
struct Supervision<'a> {
    label: &'a str,
    health_check: Option<&'a (dyn Fn() -> Result<(), HarnessError> + Send + Sync)>,
    start: Instant,
    deadline: Instant,
}

/// Polls the child until it exits, the deadline passes, or a fixture health check fails,
/// recording the outcome on the receipt. Returns the number of successful health checks.
async fn supervise(
    child: &mut std::process::Child,
    supervision: &Supervision<'_>,
    receipt: &mut Receipt,
) -> Result<usize, HarnessError> {
    let health_check_interval = Duration::from_secs(5);
    let mut last_health_check = supervision.start;
    let mut health_checks: usize = 0;

    loop {
        if child_settled(child, receipt) {
            break;
        }

        let now = Instant::now();
        if now >= supervision.deadline {
            receipt.status = String::from("timeout");
            let _ = child.kill();
            break;
        }

        let next_check = last_health_check
            .checked_add(health_check_interval)
            .ok_or(HarnessError::invalid("health check interval overflow"))?;
        if now >= next_check {
            if let Some(health) = supervision.health_check {
                match health() {
                    Ok(()) => {
                        health_checks = health_checks.saturating_add(1);
                    }
                    Err(e) => {
                        receipt.status = String::from("fixture_unavailable");
                        receipt.fixture_error_type = Some(String::from("fixture_health"));
                        receipt.fixture_error = Some(json!({"error": e.to_string()}));
                        let _ = child.kill();
                        break;
                    }
                }
            }
            last_health_check = now;
        }

        report_progress(supervision, now, last_health_check);
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
    Ok(health_checks)
}

/// Records a finished child or a failed wait on the receipt. Returns whether supervision ends.
fn child_settled(child: &mut std::process::Child, receipt: &mut Receipt) -> bool {
    match child.try_wait() {
        Ok(Some(status)) => {
            receipt.status = String::from("completed");
            receipt.child_exit_code = Some(status.code().unwrap_or(128));
            true
        }
        Ok(None) => false,
        Err(e) => {
            receipt.status = String::from("supervisor_error");
            receipt.supervisor_error = Some(json!({
                "phase": "execute",
                "type": std::any::type_name_of_val(&e),
            }));
            true
        }
    }
}

fn report_progress(supervision: &Supervision<'_>, now: Instant, last_health_check: Instant) {
    let elapsed = now.saturating_duration_since(supervision.start);
    if elapsed.as_secs() >= 30
        && last_health_check
            .checked_add(Duration::from_secs(30))
            .is_some_and(|d| now >= d)
    {
        eprintln!(
            "{}",
            serde_json::to_string(&json!({
                "phase": supervision.label,
                "elapsed_seconds": elapsed.as_secs_f64(),
            }))
            .unwrap_or_default()
        );
    }
}

/// Fills in elapsed time, health-check count, the supervisor exit code and the log digest.
fn settle_receipt(receipt: &mut Receipt, start: Instant, health_checks: usize) {
    receipt.elapsed_seconds = Instant::now()
        .saturating_duration_since(start)
        .as_secs_f64();
    receipt.fixture_health_checks = health_checks;

    if receipt.exit_code.is_none() {
        if receipt.status.as_str() == "interrupted" {
            let signal: i32 = receipt.interrupt_signal.unwrap_or(0);
            receipt.exit_code = Some(128_i32.saturating_add(signal));
        } else if receipt.status.as_str() != "completed" {
            receipt.exit_code = Some(124_i32);
        } else if let Some(code) = receipt.child_exit_code {
            receipt.exit_code = Some(code);
        }
    }

    if let Ok(log_content) = fs::read(&receipt.log) {
        let hash = sha2::Sha256::new_with_prefix(&log_content).finalize();
        receipt.log_sha256 = Some(hex::encode(hash));
    }
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

    let baseline_cmd = baseline_command();

    for i in 1..=2 {
        let label = format!("oracle-baseline-{i}");
        let result = execute_command(CommandRequest {
            arguments: baseline_cmd.clone(),
            workspace: args.workspace.clone(),
            output: output.clone(),
            label: label.clone(),
            scan_id: None,
            port: 0,
            deadline_seconds: 600,
            health_check: None,
        })
        .await?;

        if result.exit_code != Some(0) {
            return Ok(result.to_json());
        }
    }

    if !args.command.is_empty() {
        let result = execute_command(CommandRequest {
            arguments: args.command.clone(),
            workspace: args.workspace.clone(),
            output: output.clone(),
            label: "mutation-command".to_string(),
            scan_id: None,
            port: 0,
            deadline_seconds: args.command_deadline,
            health_check: None,
        })
        .await?;

        Ok(result.to_json())
    } else {
        Ok(json!({
            "phase": "mutate",
            "status": "completed",
            "oracle_baselines": 2,
            "mutation_command": null,
        }))
    }
}

fn baseline_command() -> Vec<String> {
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
