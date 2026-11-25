use std::ops::ControlFlow;
use std::os::unix::process::CommandExt;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

use rustix::process::{Pid, Signal, WaitId, WaitIdOptions};

pub(super) use super::build_cleanup::clear_owned;
pub(super) use super::build_version::version;
use super::{OwnedDirectory, complete};
use crate::HarnessError;

const BUILD_TIMEOUT: Duration = Duration::from_secs(1_800);

pub(super) fn compile(source: &OwnedDirectory, jobs: usize) -> Result<(), HarnessError> {
    run_compile(compile_command(source, jobs), BUILD_TIMEOUT)
}

fn compile_command(source: &OwnedDirectory, jobs: usize) -> Command {
    let mut command = Command::new("make");
    command
        .arg("-C")
        .arg(source.anchor())
        .arg(format!("-j{jobs}"))
        .args(["MALLOC=libc", "OPT=-O2", "redis-server", "redis-cli"]);
    command
}

fn run_compile(mut command: Command, timeout: Duration) -> Result<(), HarnessError> {
    let mut child = command.process_group(0).stdin(Stdio::null()).spawn()?;
    let result = wait_compile(Pid::from_child(&child), timeout);
    let cleanup = terminate_and_reap(&mut child).and_then(compile_status);
    complete(result, cleanup)
}

fn compile_status(status: ExitStatus) -> Result<(), HarnessError> {
    if status.success() {
        return Ok(());
    }
    Err(HarnessError::invalid(format!(
        "Redis compilation failed or timed out: {status}"
    )))
}

fn wait_compile(pid: Pid, timeout: Duration) -> Result<(), HarnessError> {
    let deadline = Instant::now()
        .checked_add(timeout)
        .ok_or_else(|| HarnessError::invalid("invalid compilation deadline"))?;
    match (0..1_000_000).try_for_each(|_| match compile_step(pid, deadline) {
        Ok(true) => ControlFlow::Break(Ok(())),
        Ok(false) => ControlFlow::Continue(()),
        Err(error) => ControlFlow::Break(Err(error)),
    }) {
        ControlFlow::Break(result) => result,
        ControlFlow::Continue(()) => Err(HarnessError::invalid(
            "Redis compilation polling exceeds iteration limit",
        )),
    }
}

fn compile_step(pid: Pid, deadline: Instant) -> Result<bool, HarnessError> {
    let remaining = deadline
        .checked_duration_since(Instant::now())
        .ok_or_else(|| {
            HarnessError::invalid("Redis compilation failed or timed out: wall deadline expired")
        })?;
    if exited(pid)? {
        return Ok(true);
    }
    let timeout = rustix::event::Timespec::try_from(remaining.min(Duration::from_millis(10)))
        .map_err(|error| {
            HarnessError::invalid(format!("invalid compilation poll duration: {error}"))
        })?;
    match rustix::event::poll(&mut [], Some(&timeout)) {
        Ok(_) | Err(rustix::io::Errno::INTR) => Ok(false),
        Err(error) => Err(std::io::Error::from(error).into()),
    }
}

pub(super) fn exited(pid: Pid) -> Result<bool, HarnessError> {
    let flags = WaitIdOptions::EXITED | WaitIdOptions::NOHANG | WaitIdOptions::NOWAIT;
    rustix::process::waitid(WaitId::Pid(pid), flags)
        .map(|status| status.is_some())
        .map_err(|error| std::io::Error::from(error).into())
}

pub(super) fn terminate_and_reap(child: &mut Child) -> Result<ExitStatus, HarnessError> {
    let killed = match rustix::process::kill_process_group(Pid::from_child(child), Signal::KILL) {
        Ok(()) | Err(rustix::io::Errno::SRCH) => Ok(()),
        Err(error) => Err(std::io::Error::from(error).into()),
    };
    let waited = child.wait().map_err(HarnessError::from);
    complete(waited, killed)
}

#[cfg(test)]
pub(super) fn versions(source: &OwnedDirectory) -> Result<(String, String), HarnessError> {
    let server = version(&source.open_binary("redis-server")?)?;
    let client = version(&source.open_binary("redis-cli")?)?;
    validate_versions(&server, &client)?;
    Ok((server, client))
}

pub(super) fn validate_versions(server: &str, client: &str) -> Result<(), HarnessError> {
    let mut server_tokens = server.split_ascii_whitespace();
    let mut client_tokens = client.split_ascii_whitespace();
    if server_tokens.next() != Some("Redis")
        || server_tokens.next() != Some("server")
        || server_tokens
            .next()
            .and_then(|token| token.strip_prefix("v="))
            != Some(super::REDIS_VERSION)
        || client_tokens.next() != Some("redis-cli")
        || client_tokens.next() != Some(super::REDIS_VERSION)
        || server.contains(['\n', '\r'])
        || client.contains(['\n', '\r'])
    {
        return Err(HarnessError::invalid(
            "compiled Redis version differs from the pin",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super as build;
    use super::super::installer_tests::{CLIENT_VERSION, SERVER_VERSION, TestResult};

    #[test]
    fn given_native_versions_when_parsed_then_exact_pins_are_accepted() -> TestResult {
        build::validate_versions(SERVER_VERSION, CLIENT_VERSION)?;
        build::validate_versions(SERVER_VERSION, "redis-cli 8.10.3")?;
        [
            "redis-cli 8.10.30",
            "redis-cli 8.10.3evil",
            "prefix redis-cli 8.10.3",
        ]
        .into_iter()
        .for_each(|client| assert!(build::validate_versions(SERVER_VERSION, client).is_err()));
        [
            "Redis server v=8.10.30 sha=x",
            "Redis server v=8.10.3evil",
            "Redis server v=8.10.3\nextra",
        ]
        .into_iter()
        .for_each(|server| assert!(build::validate_versions(server, CLIENT_VERSION).is_err()));
        Ok(())
    }
}

#[cfg(test)]
#[path = "build_process_tests.rs"]
mod process_tests;
