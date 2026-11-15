use super::{Duration, TestResult};
use std::{fmt, io, path::Path, process::{ExitStatus, Output, Stdio}};
use tokio::process::{Child, Command};

#[derive(Debug)]
enum ChildFailure {
    Deadline { pid: u32, reaped: ExitStatus },
    Kill { pid: u32, source: io::Error },
    Reap { pid: u32, source: io::Error },
    ReapDeadline { pid: u32 },
}

impl fmt::Display for ChildFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Deadline { pid, reaped } => write!(formatter, "collector execution deadline expired; child {pid} killed and reaped ({reaped})"),
            Self::Kill { pid, source } => write!(formatter, "collector child {pid} kill failed: {source}"),
            Self::Reap { pid, source } => write!(formatter, "collector child {pid} reap failed: {source}"),
            Self::ReapDeadline { pid } => write!(formatter, "collector child {pid} cleanup deadline expired before reap"),
        }
    }
}

impl std::error::Error for ChildFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Kill { source, .. } | Self::Reap { source, .. } => Some(source),
            Self::Deadline { .. } | Self::ReapDeadline { .. } => None,
        }
    }
}

pub(super) async fn run(command: &mut Command, stdout: &Path, stderr: &Path, budget: Duration) -> TestResult<Output> {
    // Regular owned logs eliminate pipe-drain workers and inherited-pipe EOF waits.
    let out = std::fs::File::create(stdout)?;
    let err = std::fs::File::create(stderr)?;
    let mut child = command.stdin(Stdio::null()).stdout(Stdio::from(out)).stderr(Stdio::from(err))
        .kill_on_drop(true).spawn()?;
    command.stdout(Stdio::null()).stderr(Stdio::null());
    let pid = child.id().ok_or("spawned collector has no process identity")?;
    let status = match tokio::time::timeout(budget, child.wait()).await {
        Ok(Ok(status)) => status,
        Ok(Err(error)) => {
            terminate(&mut child, pid).await?;
            return Err(error.into());
        }
        Err(_) => {
            let reaped = terminate(&mut child, pid).await?;
            return Err(ChildFailure::Deadline { pid, reaped }.into());
        }
    };
    let (stdout, stderr) = tokio::try_join!(tokio::fs::read(stdout), tokio::fs::read(stderr))?;
    Ok(Output { status, stdout, stderr })
}

async fn terminate(child: &mut Child, pid: u32) -> Result<ExitStatus, ChildFailure> {
    if child.try_wait().map_err(|source| ChildFailure::Reap { pid, source })?.is_none() {
        child.start_kill().map_err(|source| ChildFailure::Kill { pid, source })?;
    }
    tokio::time::timeout(Duration::from_secs(5), child.wait()).await
        .map_err(|_| ChildFailure::ReapDeadline { pid })?
        .map_err(|source| ChildFailure::Reap { pid, source })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "current_thread")]
    async fn given_live_collector_child_when_execution_expires_then_return_requires_reap_and_closed_logs() -> TestResult {
        let owned = tempfile::tempdir()?;
        let stdout = owned.path().join("collector.stdout");
        let stderr = owned.path().join("collector.stderr");
        let marker = owned.path().join("started");
        let mut command = Command::new("sh");
        command.env_clear().arg("-c").arg("printf started; printf diagnostic >&2; printf ready >\"$1\"; exec sleep 60")
            .arg("collector-deadline-fixture").arg(&marker);
        let result = tokio::time::timeout(Duration::from_secs(8), run(&mut command, &stdout, &stderr, Duration::from_secs(1))).await?;
        let error = result.err().ok_or("controlled collector unexpectedly exited before its deadline")?;
        let Some(ChildFailure::Deadline { pid, reaped }) = error.downcast_ref::<ChildFailure>() else {
            return Err(format!("execution timeout must remain distinct from cleanup failure: {error}").into());
        };
        assert!(!reaped.success(), "deadline must terminate the still-running owned child");
        assert!(!Path::new("/proc").join(pid.to_string()).exists(), "deadline return must already have reaped the child");
        assert_eq!(tokio::fs::read(&marker).await?, b"ready");
        assert_eq!(tokio::fs::read(&stdout).await?, b"started");
        assert_eq!(tokio::fs::read(&stderr).await?, b"diagnostic");
        drop(command);
        let directory = owned.path().to_path_buf();
        owned.close()?;
        assert!(!directory.exists(), "no child or pipe reader may outlive returned timeout cleanup");
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn given_collector_success_when_child_exits_then_exact_owned_logs_are_returned() -> TestResult {
        let owned = tempfile::tempdir()?;
        let mut command = Command::new("sh");
        command.env_clear().args(["-c", "printf result; printf diagnostic >&2"]);
        let output = run(&mut command, &owned.path().join("stdout"), &owned.path().join("stderr"), Duration::from_secs(3)).await?;
        assert!(output.status.success());
        assert_eq!(output.stdout, b"result");
        assert_eq!(output.stderr, b"diagnostic");
        Ok(())
    }
}
