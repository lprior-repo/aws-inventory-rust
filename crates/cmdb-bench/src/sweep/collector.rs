use super::summary::{self, SweepLines};
use std::{
    path::Path,
    process::Stdio,
    time::{Duration, Instant},
};
use tokio::process::Command;
use tokio_util::sync::CancellationToken;
#[derive(Debug)]
pub(crate) struct Finished {
    pub wall_ms: u64,
    pub code: i32,
    pub lines: SweepLines,
}
pub(crate) async fn sweep(
    mut command: Command,
    log: &Path,
    cancel: &CancellationToken,
) -> Result<Finished, String> {
    if cancel.is_cancelled() {
        return Err("collector cancelled before launch".into());
    }
    let file = std::fs::File::create(log).map_err(|error| error.to_string())?;
    command
        .stdin(Stdio::null())
        .stdout(Stdio::from(
            file.try_clone().map_err(|error| error.to_string())?,
        ))
        .stderr(Stdio::from(file))
        .kill_on_drop(true);
    let started = Instant::now();
    let mut child = command
        .spawn()
        .map_err(|error| format!("start collector: {error}"))?;
    let waited = tokio::select! {
        biased;
        () = cancel.cancelled() => {
            child.kill().await.map_err(|error| format!("kill cancelled collector: {error}"))?;
            return Err("collector cancelled and reaped".into());
        }
        status = tokio::time::timeout(Duration::from_secs(3600), child.wait()) => status,
    };
    let status = match waited {
        Ok(status) => status.map_err(|error| format!("wait for collector: {error}"))?,
        Err(_) => {
            child
                .kill()
                .await
                .map_err(|error| format!("kill timed-out collector: {error}"))?;
            return Err(format!(
                "collector exceeded one hour; log {}",
                log.display()
            ));
        }
    };
    let wall_ms =
        u64::try_from(started.elapsed().as_millis()).map_err(|error| error.to_string())?;
    let code = status
        .code()
        .ok_or_else(|| format!("collector terminated by signal; log {}", log.display()))?;
    if code != 0 && code != 2 {
        return Err(format!("collector exit={code}; log {}", log.display()));
    }
    let text = super::bounded::read(log, super::bounded::REPORT_LIMIT).await?;
    let lines = summary::parse(&text).map_err(|error| format!("{error}; log {}", log.display()))?;
    Ok(Finished {
        wall_ms,
        code,
        lines,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn actual_process_exit_and_both_output_streams_are_observed() -> Result<(), String> {
        let directory = tempfile::tempdir().map_err(|error| error.to_string())?;
        let valid = "resource_writes_scope=invocation resource_writes=1 rows=1 scopes=1 complete=1 partial=0 inaccessible=0 unsupported=0 failed=0 unobserved=0 graph_pending=0 work_remaining=0 unattributed_resources=0\nmetrics bootstrap_ms=1 seed_ms=1 drain_ms=1 api_attempts=1 api_retries=0 api_throttles=0 connections_opened=1 requests_sent=1 requests_reused=0 http2_requests=0 bytes_read=1 handshake_ns=1 hosts=1 pages=1 decode_ns=1 store_commit_ns=1 store_commits=1 claims=1 progress_dropped=0 child_requests_skipped=0 cpu_user_ms=1 cpu_system_ms=1 peak_rss_kb=1 process_metrics=available";
        let mut command = Command::new("/bin/sh");
        command.args([
            "-c",
            "printf '%s\\n' \"$1\" >&2; exit 2",
            "collector",
            valid,
        ]);
        let result = sweep(
            command,
            &directory.path().join("log"),
            &CancellationToken::new(),
        )
        .await?;
        assert_eq!(result.code, 2);
        assert_eq!(result.lines.get("rows")?, 1);
        let mut command = Command::new("/bin/sh");
        command.args(["-c", "printf '%s\\n' \"$1\"", "collector", valid]);
        let path = directory.path().join("stdout");
        let result = sweep(command, &path, &CancellationToken::new()).await?;
        assert_eq!(result.code, 0);
        assert_eq!(result.lines.get("api_attempts")?, 1);
        assert!(
            tokio::fs::read_to_string(path)
                .await
                .map_err(|error| error.to_string())?
                .starts_with("resource_writes_scope=invocation")
        );
        let mut command = Command::new("/bin/sh");
        command.args(["-c", "exit 1"]);
        assert!(
            sweep(
                command,
                &directory.path().join("error"),
                &CancellationToken::new()
            )
            .await
            .is_err()
        );
        Ok(())
    }
    #[tokio::test]
    async fn oversized_collector_log_is_rejected_before_parsing() -> Result<(), String> {
        let directory = tempfile::tempdir().map_err(|error| error.to_string())?;
        let mut command = Command::new("/bin/sh");
        command.args(["-c", "truncate -s 16777217 /proc/self/fd/1"]);
        let error = sweep(
            command,
            &directory.path().join("sparse-log"),
            &CancellationToken::new(),
        )
        .await
        .err()
        .ok_or("oversized log accepted")?;
        assert!(error.contains("16777216-byte"), "{error}");
        Ok(())
    }
}
