use std::fs::{File, OpenOptions};
use std::path::Path;
use std::process::{Command, Stdio};

use super::{Layout, complete, receipt};
use crate::HarnessError;

const BUILD_TIMEOUT: &str = "1800s";
const VERSION_TIMEOUT: &str = "5s";

pub(super) fn compile(layout: &Layout, jobs: usize) -> Result<(), HarnessError> {
    let status = timeout(BUILD_TIMEOUT)
        .arg("make")
        .arg("-C")
        .arg(&layout.source)
        .arg(format!("-j{jobs}"))
        .args(["MALLOC=libc", "OPT=-O2", "redis-server", "redis-cli"])
        .status()?;
    if status.success() {
        return Ok(());
    }
    Err(HarnessError::invalid(format!(
        "Redis compilation failed or timed out: {status}"
    )))
}

fn timeout(duration: &str) -> Command {
    let mut command = Command::new("timeout");
    command.args(["--signal=KILL", duration]);
    command.stdin(Stdio::null());
    command
}

pub(super) fn versions(layout: &Layout) -> Result<(String, String), HarnessError> {
    let server = version(layout, &layout.server, "server.version")?;
    let client = version(layout, &layout.client, "client.version")?;
    validate_versions(&server, &client)?;
    Ok((server, client))
}

fn version(layout: &Layout, binary: &Path, name: &str) -> Result<String, HarnessError> {
    receipt::regular(layout, binary)?;
    let path = layout.lock.join(name);
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)?;
    let result = version_output(binary, file, &path);
    complete(result, std::fs::remove_file(path).map_err(Into::into))
}

fn version_output(binary: &Path, file: File, path: &Path) -> Result<String, HarnessError> {
    let status = timeout(VERSION_TIMEOUT)
        .arg(binary)
        .arg("--version")
        .stdout(file)
        .stderr(Stdio::null())
        .status()?;
    if !status.success() {
        return Err(HarnessError::invalid(format!(
            "Redis version command failed or timed out: {status}"
        )));
    }
    let bytes = receipt::read_bounded(path, 4_096)?;
    let version = std::str::from_utf8(&bytes)
        .map_err(|error| HarnessError::invalid(format!("invalid Redis version UTF-8: {error}")))?;
    Ok(version.trim().to_owned())
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
    use std::fs;

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

    fn terminated(pid: u32) -> std::io::Result<bool> {
        match fs::read_to_string(format!("/proc/{pid}/status")) {
            Ok(state) => Ok(state
                .lines()
                .any(|line| line.starts_with("State:") && line.contains("Z"))),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(true),
            Err(error) => Err(error),
        }
    }

    fn wait_for_exit(pid: u32) -> std::io::Result<bool> {
        (0..100).try_fold(false, |exited, _| {
            if exited {
                return Ok(true);
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
            terminated(pid)
        })
    }

    #[test]
    fn given_process_group_when_deadline_expires_then_descendants_are_killed() -> TestResult {
        let directory = tempfile::tempdir()?;
        let pid_file = directory.path().join("child.pid");
        let script = "sleep 60 & printf '%s' \"$!\" > \"$1\"; wait";
        let status = build::timeout("1s")
            .args(["sh", "-c", script, "group"])
            .arg(&pid_file)
            .status()?;
        assert!(!status.success());
        let pid = fs::read_to_string(pid_file)?.parse::<u32>()?;
        assert!(wait_for_exit(pid)?);
        Ok(())
    }
}
