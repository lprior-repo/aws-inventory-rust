use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use serde_json::Value;

use crate::HarnessError;

#[cfg(test)]
mod tests;

pub const PINNED_VERSION: &str = "1.0.348";

pub const PINNED_COMMIT: &str = "7461ab1bbe540afe8063ce03ee5477ebb649df48";

const READY_TIMEOUT: Duration = Duration::from_secs(30);

const READY_POLL: Duration = Duration::from_millis(200);

const READY_ATTEMPTS: usize = 150;

const ARGUMENTS: [&str; 9] = [
    "--port",
    "0",
    "--bind",
    "127.0.0.1",
    "--auth",
    "on",
    "--telemetry",
    "off",
    "--idle-timeout",
];

struct ChildOwner {
    child: Child,
}

impl ChildOwner {
    fn terminate(&mut self) -> Result<(), HarnessError> {
        match self.child.kill() {
            Ok(()) => {
                self.child.wait()?;
                Ok(())
            }
            Err(error) => match self.child.try_wait()? {
                Some(_) => Ok(()),
                None => Err(error.into()),
            },
        }
    }
}

impl Drop for ChildOwner {
    fn drop(&mut self) {
        if let Err(error) = self.terminate() {
            report(&error);
        }
    }
}

fn report(error: &HarnessError) {
    match writeln!(
        std::io::stderr().lock(),
        "emulator child cleanup failed: {error}"
    ) {
        Ok(()) | Err(_) => {}
    }
}

pub struct Emulator {
    child: ChildOwner,
    endpoint: String,
    credentials: PathBuf,
    config: PathBuf,
    ca_bundle: PathBuf,
}

fn spawn(
    binary: &Path,
    log: &Path,
    home: &Path,
    idle_timeout_seconds: u64,
) -> Result<ChildOwner, HarnessError> {
    let handle = std::fs::File::create(log)?;
    let child = Command::new(binary)
        .args(ARGUMENTS)
        .arg(idle_timeout_seconds.to_string())
        .env("HOME", home)
        .stdout(Stdio::from(handle.try_clone()?))
        .stderr(Stdio::from(handle))
        .spawn()?;
    Ok(ChildOwner { child })
}

fn launched(
    binary: &Path,
    work: &Path,
    idle_timeout_seconds: u64,
) -> Result<(PathBuf, PathBuf, ChildOwner), HarnessError> {
    verify_binary(binary)?;
    let home = work.join("home");
    std::fs::create_dir_all(&home)?;
    let log = work.join("emulator.jsonl");
    let child = spawn(binary, &log, &home, idle_timeout_seconds)?;
    Ok((log, home, child))
}

fn emulator_from(
    child: ChildOwner,
    endpoint: String,
    home: &Path,
) -> Result<Emulator, HarnessError> {
    let port = endpoint_port(&endpoint)?;
    let directory = home.join(".aws/emulator").join(port.to_string());
    Ok(Emulator {
        child,
        endpoint,
        credentials: directory.join("emulator_credentials"),
        config: directory.join("config"),
        ca_bundle: directory.join("ca.pem"),
    })
}

impl Emulator {
    pub fn start(
        binary: &Path,
        work: &Path,
        idle_timeout_seconds: u64,
    ) -> Result<Self, HarnessError> {
        let (log, home, mut child) = launched(binary, work, idle_timeout_seconds)?;
        let endpoint = await_endpoint(&log, &mut child, READY_TIMEOUT, READY_POLL, READY_ATTEMPTS)?;
        emulator_from(child, endpoint, &home)
    }

    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    pub fn credentials(&self) -> &Path {
        &self.credentials
    }

    pub fn ca_bundle(&self) -> &Path {
        &self.ca_bundle
    }

    pub fn environment(&self) -> Vec<(String, String)> {
        let text = |path: &Path| path.to_string_lossy().into_owned();
        vec![
            ("AWS_ENDPOINT_URL".to_owned(), self.endpoint.clone()),
            ("AWS_CA_BUNDLE".to_owned(), text(&self.ca_bundle)),
            ("SSL_CERT_FILE".to_owned(), text(&self.ca_bundle)),
            (
                "AWS_SHARED_CREDENTIALS_FILE".to_owned(),
                text(&self.credentials),
            ),
            ("AWS_CONFIG_FILE".to_owned(), text(&self.config)),
            ("AWS_REGION".to_owned(), "us-east-1".to_owned()),
            ("AWS_DEFAULT_REGION".to_owned(), "us-east-1".to_owned()),
            ("AWS_EC2_METADATA_DISABLED".to_owned(), "true".to_owned()),
            (
                "AWS_DISABLE_HOST_PREFIX_INJECTION".to_owned(),
                "true".to_owned(),
            ),
            ("NO_PROXY".to_owned(), "127.0.0.1,::1".to_owned()),
            ("EMULATOR_TELEMETRY".to_owned(), "off".to_owned()),
        ]
    }

    pub fn stop(mut self) -> Result<(), HarnessError> {
        self.child.terminate()
    }
}

pub fn verify_binary(binary: &Path) -> Result<(), HarnessError> {
    let output = Command::new(binary).arg("--version").output()?;
    if !output.status.success() {
        return Err(HarnessError::invalid(
            "emulator binary did not report its version",
        ));
    }
    let version = String::from_utf8_lossy(&output.stdout);
    if !version.contains(PINNED_VERSION) || !version.contains(PINNED_COMMIT) {
        return Err(HarnessError::invalid(
            "emulator binary does not match the reviewed version/commit",
        ));
    }
    Ok(())
}

fn await_endpoint(
    log: &Path,
    child: &mut ChildOwner,
    timeout: Duration,
    poll: Duration,
    attempts: usize,
) -> Result<String, HarnessError> {
    let started = Instant::now();
    (0..attempts)
        .find_map(|_| {
            if started.elapsed() >= timeout {
                Some(Err(HarnessError::invalid(
                    "emulator readiness time budget exhausted",
                )))
            } else {
                poll_and_pause(log, child, poll)
            }
        })
        .ok_or_else(|| HarnessError::invalid("emulator readiness attempt budget exhausted"))?
}

fn poll_and_pause(
    log: &Path,
    child: &mut ChildOwner,
    poll: Duration,
) -> Option<Result<String, HarnessError>> {
    match poll_endpoint(log, child) {
        Ok(None) => {
            std::thread::sleep(poll);
            None
        }
        Ok(Some(endpoint)) => Some(Ok(endpoint)),
        Err(error) => Some(Err(error)),
    }
}

fn poll_endpoint(log: &Path, child: &mut ChildOwner) -> Result<Option<String>, HarnessError> {
    if let Some(status) = child.child.try_wait()? {
        return Err(HarnessError::invalid(format!(
            "emulator exited before readiness: {status}"
        )));
    }
    let bytes = crate::files::read_bytes(log, crate::snapshot::MAX_DOCUMENT_BYTES)?;
    Ok(bytes
        .split_inclusive(|byte| *byte == b'\n')
        .filter(|line| line.last() == Some(&b'\n'))
        .filter_map(|line| std::str::from_utf8(line).ok())
        .find_map(endpoint_from_line))
}

pub fn endpoint_from_line(line: &str) -> Option<String> {
    let value: Value = serde_json::from_str(line).ok()?;
    value
        .get("fields")
        .and_then(|fields| fields.get("endpoint_url"))
        .and_then(Value::as_str)
        .map(str::to_owned)
        .filter(|endpoint| loopback_port(endpoint).is_some())
}

fn endpoint_port(endpoint: &str) -> Result<u16, HarnessError> {
    loopback_port(endpoint)
        .ok_or_else(|| HarnessError::invalid("emulator endpoint is not HTTPS loopback with a port"))
}

fn loopback_port(endpoint: &str) -> Option<u16> {
    let rest = endpoint.strip_prefix("https://127.0.0.1:")?;
    if rest.is_empty() || !rest.chars().all(|character| character.is_ascii_digit()) {
        return None;
    }
    rest.parse::<u16>().ok().filter(|port| *port != 0)
}
