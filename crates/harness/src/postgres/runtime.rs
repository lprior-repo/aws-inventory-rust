use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

use tokio::process::Command;

use super::NATIVE_INSTRUCTIONS;
use crate::HarnessError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Runtime {
    Podman,
    Docker,
}

impl Runtime {
    pub fn parse(value: &str) -> Result<Self, HarnessError> {
        match value {
            "podman" => Ok(Self::Podman),
            "docker" => Ok(Self::Docker),
            _ => Err(HarnessError::invalid("--runtime must be docker or podman")),
        }
    }

    fn executable(self) -> &'static str {
        match self {
            Self::Podman => "podman",
            Self::Docker => "docker",
        }
    }
}

#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum Action {
    Up,
    Down,
    DownWithVolumes,
}

pub async fn compose(runtime: Option<Runtime>, action: Action) -> Result<(), HarnessError> {
    let path = std::env::var_os("PATH").ok_or_else(|| HarnessError::invalid(NATIVE_INSTRUCTIONS))?;
    let executable = detect_in(&path, runtime).await?;
    let compose_file = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../deploy/local/compose.yaml");
    compose_at(&executable, &compose_file, action).await
}

async fn detect_in(path: &OsStr, requested: Option<Runtime>) -> Result<PathBuf, HarnessError> {
    if let Some(runtime) = requested {
        return available_in(path, runtime).await
            .ok_or_else(|| HarnessError::invalid(NATIVE_INSTRUCTIONS));
    }
    if let Some(executable) = available_in(path, Runtime::Podman).await {
        return Ok(executable);
    }
    available_in(path, Runtime::Docker).await
        .ok_or_else(|| HarnessError::invalid(NATIVE_INSTRUCTIONS))
}

async fn available_in(path: &OsStr, runtime: Runtime) -> Option<PathBuf> {
    let executable = std::env::split_paths(path).take(128)
        .map(|directory| directory.join(runtime.executable())).find(|file| file.is_file())?;
    let mut command = Command::new(&executable);
    command.arg("info").stdout(Stdio::null()).stderr(Stdio::null());
    match run(&mut command, Duration::from_secs(10)).await {
        Ok(()) => Some(executable),
        Err(_) => None,
    }
}

async fn compose_at(executable: &Path, file: &Path, action: Action) -> Result<(), HarnessError> {
    let mut command = Command::new(executable);
    command.args(["compose", "--project-name", "inventory-local", "-f"]).arg(file);
    match action {
        Action::Up => { command.args(["up", "-d", "--wait"]); }
        Action::Down => { command.arg("down"); }
        Action::DownWithVolumes => { command.args(["down", "--volumes"]); }
    }
    command.stdout(Stdio::inherit()).stderr(Stdio::inherit());
    run(&mut command, Duration::from_secs(120)).await
}

async fn run(command: &mut Command, deadline: Duration) -> Result<(), HarnessError> {
    let mut child = command.stdin(Stdio::null()).kill_on_drop(true).spawn()?;
    let status = match tokio::time::timeout(deadline, child.wait()).await {
        Ok(status) => status?,
        Err(_) => {
            child.kill().await?;
            return Err(HarnessError::invalid("container runtime exceeded its deadline"));
        }
    };
    if !status.success() {
        return Err(HarnessError::invalid("container runtime command failed"));
    }
    Ok(())
}

#[cfg(test)]
#[path = "runtime_tests.rs"]
mod tests;
