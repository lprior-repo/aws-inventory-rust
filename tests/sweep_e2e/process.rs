use super::guard::Environment;
use anyhow::{Context, Result, ensure};
use std::io::{Read, Seek, SeekFrom};
use std::process::{Child, Command, Stdio};
use std::time::Duration;

const POLL_ATTEMPTS: usize = 600;
const POLL_INTERVAL: Duration = Duration::from_millis(100);

struct ChildOwner(Child);

pub(super) struct RunningCli {
    child: ChildOwner,
    stdout: std::fs::File,
    stderr: std::fs::File,
}

impl Drop for ChildOwner {
    fn drop(&mut self) {
        if self.0.try_wait().ok().flatten().is_none() {
            drop(self.0.kill());
            drop(self.0.wait());
        }
    }
}

pub(super) async fn run_cli(environment: &Environment, arguments: &[String]) -> Result<String> {
    run_cli_with_status(environment, arguments, 0).await
}

pub(super) async fn run_cli_degraded(
    environment: &Environment,
    arguments: &[String],
) -> Result<String> {
    run_cli_with_status(environment, arguments, 2).await
}

pub(super) async fn run_cli_graph_pending(
    environment: &Environment,
    arguments: &[String],
) -> Result<String> {
    run_cli_with_status(environment, arguments, 5).await
}

async fn run_cli_with_status(
    environment: &Environment,
    arguments: &[String],
    expected_code: i32,
) -> Result<String> {
    let environment = environment.clone();
    let arguments = arguments.to_vec();
    tokio::task::spawn_blocking(move || {
        let mut process = spawn_cli_sync(&environment, &arguments)?;
        wait_cli_sync(&mut process, expected_code)
    })
    .await
    .context("joining bounded sweep CLI runner")?
}

pub(super) async fn spawn_cli(
    environment: &Environment,
    arguments: &[String],
) -> Result<RunningCli> {
    let environment = environment.clone();
    let arguments = arguments.to_vec();
    tokio::task::spawn_blocking(move || spawn_cli_sync(&environment, &arguments))
        .await
        .context("spawning bounded sweep CLI runner")?
}

pub(super) async fn cancel_cli(process: RunningCli) -> Result<()> {
    tokio::task::spawn_blocking(move || cancel_cli_sync(process))
        .await
        .context("reaping cancelled sweep CLI process")?
}

pub(super) async fn crash_cli(mut process: RunningCli) -> Result<()> {
    tokio::task::spawn_blocking(move || {
        ensure!(
            process.child.0.try_wait()?.is_none(),
            "CLI exited before SIGKILL"
        );
        process.child.0.kill().context("crashing owned sweep CLI")?;
        let status = process
            .child
            .0
            .wait()
            .context("reaping crashed sweep CLI")?;
        ensure!(
            status.code().is_none(),
            "SIGKILL did not terminate CLI: {status:?}"
        );
        Ok(())
    })
    .await
    .context("joining crashed sweep CLI runner")?
}

pub(super) async fn run_cli_lease_blocked(
    environment: &Environment,
    arguments: &[String],
) -> Result<()> {
    let environment = environment.clone();
    let arguments = arguments.to_vec();
    tokio::task::spawn_blocking(move || {
        let mut process = spawn_cli_sync(&environment, &arguments)?;
        wait_cli_sync(&mut process, 1)?;
        let diagnostic = read_output(&mut process.stderr)?;
        ensure!(
            diagnostic.contains("sweep has live leases"),
            "crash resume failed for an unexpected reason: {diagnostic}"
        );
        Ok(())
    })
    .await
    .context("joining live-lease resume rejection")?
}

fn spawn_cli_sync(environment: &Environment, arguments: &[String]) -> Result<RunningCli> {
    let stdout = tempfile::tempfile()?;
    let stderr = tempfile::tempfile()?;
    let child = spawn(environment, arguments, &stdout, &stderr)?;
    Ok(RunningCli {
        child,
        stdout,
        stderr,
    })
}

fn wait_cli_sync(process: &mut RunningCli, expected_code: i32) -> Result<String> {
    let started = std::time::Instant::now();
    let status = wait_bounded(&mut process.child);
    let output = read_output(&mut process.stdout)?;
    let diagnostic = read_output(&mut process.stderr)?;
    if std::env::var_os("INVENTORY_E2E_TRACE").is_some() {
        eprintln!(
            "sweep elapsed_ms={}\n{output}{diagnostic}",
            started.elapsed().as_millis()
        );
    }
    let status = status.with_context(|| format!("sweep stderr: {diagnostic}"))?;
    ensure!(
        status.code() == Some(expected_code),
        "sweep CLI exited with {:?}, expected {expected_code}: {diagnostic}",
        status.code()
    );
    Ok(output)
}

fn cancel_cli_sync(mut process: RunningCli) -> Result<()> {
    ensure!(
        process.child.0.try_wait()?.is_none(),
        "checkpoint CLI finished before Ctrl-C cancellation"
    );
    let signal = Command::new("kill")
        .args(["-INT", "--", &process.child.0.id().to_string()])
        .status()
        .context("sending Ctrl-C to owned sweep CLI")?;
    ensure!(signal.success(), "could not interrupt owned sweep CLI");
    let status = wait_bounded(&mut process.child)?;
    let diagnostic = read_output(&mut process.stderr)?;
    ensure!(
        status.code() == Some(1) && diagnostic.contains("API execution cancelled"),
        "Ctrl-C did not cancel sweep collection: {status:?}: {diagnostic}"
    );
    Ok(())
}

fn spawn(
    environment: &Environment,
    arguments: &[String],
    stdout: &std::fs::File,
    stderr: &std::fs::File,
) -> Result<ChildOwner> {
    let child = Command::new(env!("CARGO_BIN_EXE_aws-inventory"))
        .env_clear()
        .env("AWS_CONFIG_FILE", &environment.aws_config)
        .env("AWS_SHARED_CREDENTIALS_FILE", &environment.aws_credentials)
        .env("AWS_EC2_METADATA_DISABLED", "true")
        .env("RUST_LOG", log_filter())
        .args(arguments)
        .stdin(Stdio::null())
        .stdout(Stdio::from(stdout.try_clone()?))
        .stderr(Stdio::from(stderr.try_clone()?))
        .spawn()
        .context("spawning production aws-inventory sweep")?;
    Ok(ChildOwner(child))
}

fn wait_bounded(child: &mut ChildOwner) -> Result<std::process::ExitStatus> {
    for _ in 0..POLL_ATTEMPTS {
        if let Some(status) = child.0.try_wait()? {
            return Ok(status);
        }
        std::thread::sleep(POLL_INTERVAL);
    }
    timeout_child(child)
}

fn timeout_child(child: &mut ChildOwner) -> Result<std::process::ExitStatus> {
    child.0.kill().context("killing timed-out sweep CLI")?;
    child.0.wait().context("reaping timed-out sweep CLI")?;
    anyhow::bail!("sweep CLI exceeded its 60-second deadline")
}

fn read_output(file: &mut std::fs::File) -> Result<String> {
    file.seek(SeekFrom::Start(0))?;
    let mut output = String::new();
    file.take(1_048_576).read_to_string(&mut output)?;
    Ok(output)
}

fn log_filter() -> &'static str {
    if std::env::var_os("INVENTORY_E2E_TRACE").is_some() {
        "inventory_sweep=debug"
    } else {
        "warn"
    }
}
