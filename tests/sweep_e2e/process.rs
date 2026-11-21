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

async fn run_cli_with_status(
    environment: &Environment,
    arguments: &[String],
    expected_code: i32,
) -> Result<String> {
    let environment = environment.clone();
    let arguments = arguments.to_vec();
    tokio::task::spawn_blocking(move || {
        wait_cli_sync(spawn_cli_sync(&environment, &arguments)?, expected_code)
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

pub(super) async fn terminate_cli(process: RunningCli) -> Result<()> {
    tokio::task::spawn_blocking(move || terminate_cli_sync(process))
        .await
        .context("reaping interrupted sweep CLI process")?
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

fn wait_cli_sync(mut process: RunningCli, expected_code: i32) -> Result<String> {
    let status = wait_bounded(&mut process.child)?;
    let output = read_output(&mut process.stdout)?;
    let diagnostic = read_output(&mut process.stderr)?;
    ensure!(
        status.code() == Some(expected_code),
        "sweep CLI exited with {:?}, expected {expected_code}: {diagnostic}",
        status.code()
    );
    Ok(output)
}

fn terminate_cli_sync(mut process: RunningCli) -> Result<()> {
    if process.child.0.try_wait()?.is_some() {
        return Ok(());
    }
    if let Err(error) = process.child.0.kill() {
        ensure!(
            process.child.0.try_wait()?.is_some(),
            "could not kill interrupted sweep CLI: {error}"
        );
        return Ok(());
    }
    process
        .child
        .0
        .wait()
        .context("reaping interrupted sweep CLI")?;
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
        .args(arguments)
        .stdin(Stdio::null())
        .stdout(Stdio::from(stdout.try_clone()?))
        .stderr(Stdio::from(stderr.try_clone()?))
        .spawn()
        .context("spawning production aws-inventory sweep")?;
    Ok(ChildOwner(child))
}

fn wait_bounded(child: &mut ChildOwner) -> Result<std::process::ExitStatus> {
    (0..POLL_ATTEMPTS)
        .find_map(|_| match child.0.try_wait() {
            Ok(Some(status)) => Some(Ok(status)),
            Ok(None) => {
                std::thread::sleep(POLL_INTERVAL);
                None
            }
            Err(error) => Some(Err(error.into())),
        })
        .unwrap_or_else(|| timeout_child(child))
}

fn timeout_child(child: &mut ChildOwner) -> Result<std::process::ExitStatus> {
    child.0.kill().context("killing timed-out sweep CLI")?;
    child.0.wait().context("reaping timed-out sweep CLI")?;
    anyhow::bail!("sweep CLI exceeded its 60-second deadline")
}

fn read_output(file: &mut std::fs::File) -> Result<String> {
    file.seek(SeekFrom::Start(0))?;
    let mut output = String::new();
    file.take(65_536).read_to_string(&mut output)?;
    Ok(output)
}
