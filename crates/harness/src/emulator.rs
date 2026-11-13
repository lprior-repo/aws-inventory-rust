//! Owned AWS Emulator process for Rust tests.
//!
//! The emulator is a development fixture: it is started on an ephemeral loopback
//! port with signature checks and telemetry off. Its endpoint and generated
//! credentials are discovered from the pinned binary's own log and home folder,
//! exactly as the collector's environment expects them.

use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use serde_json::Value;

use crate::HarnessError;

/// Pinned emulator release string.
pub const PINNED_VERSION: &str = "1.0.348";
/// Pinned emulator source commit.
pub const PINNED_COMMIT: &str = "7461ab1bbe540afe8063ce03ee5477ebb649df48";
/// Readiness budget for a fresh process.
const READY_TIMEOUT: Duration = Duration::from_secs(30);
/// Poll interval while waiting for the endpoint announcement.
const READY_POLL: Duration = Duration::from_millis(200);

/// A running emulator whose lifetime is tied to this value.
pub struct Emulator {
    child: Child,
    endpoint: String,
    credentials: PathBuf,
    config: PathBuf,
    ca_bundle: PathBuf,
}

fn spawn(binary: &Path, log: &Path, idle_timeout_seconds: u64) -> Result<Child, HarnessError> {
    let handle = std::fs::File::create(log)?;
    Ok(Command::new(binary)
        .args([
            "--port",
            "0",
            "--bind",
            "127.0.0.1",
            "--auth",
            "on",
            "--telemetry",
            "off",
            "--idle-timeout",
        ])
        .arg(idle_timeout_seconds.to_string())
        .stdout(Stdio::from(handle.try_clone()?))
        .stderr(Stdio::from(handle))
        .spawn()?)
}

fn launched(
    binary: &Path,
    work: &Path,
    idle_timeout_seconds: u64,
) -> Result<(PathBuf, Child), HarnessError> {
    verify_binary(binary)?;
    std::fs::create_dir_all(work)?;
    let log = work.join("emulator.jsonl");
    let child = spawn(binary, &log, idle_timeout_seconds)?;
    Ok((log, child))
}

fn emulator_from(child: Child, endpoint: String) -> Result<Emulator, HarnessError> {
    let port = endpoint_port(&endpoint)?;
    let directory = home()?.join(".aws/emulator").join(port.to_string());
    Ok(Emulator {
        child,
        endpoint,
        credentials: directory.join("emulator_credentials"),
        config: directory.join("config"),
        ca_bundle: directory.join("ca.pem"),
    })
}

impl Emulator {
    /// Start the pinned binary and wait until it announces its endpoint.
    pub fn start(
        binary: &Path,
        work: &Path,
        idle_timeout_seconds: u64,
    ) -> Result<Self, HarnessError> {
        let (log, child) = launched(binary, work, idle_timeout_seconds)?;
        match await_endpoint(&log, child) {
            Ok((child, endpoint)) => emulator_from(child, endpoint),
            Err((mut child, error)) => {
                let _ = child.kill();
                let _ = child.wait();
                Err(error)
            }
        }
    }

    /// Loopback endpoint the emulator is serving.
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    /// Credential file holding this run's generated keys.
    pub fn credentials(&self) -> &Path {
        &self.credentials
    }

    /// Generated TLS authority used by the loopback listener.
    pub fn ca_bundle(&self) -> &Path {
        &self.ca_bundle
    }

    /// Environment the collector and any SDK client must run with.
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

    /// Stop the process and release its port.
    pub fn stop(mut self) -> Result<(), HarnessError> {
        self.terminate();
        Ok(())
    }

    fn terminate(&mut self) {
        if self.child.try_wait().ok().flatten().is_none() {
            let _ = self.child.kill();
        }
        let _ = self.child.wait();
    }
}

impl Drop for Emulator {
    fn drop(&mut self) {
        self.terminate();
    }
}

/// Fail unless the binary is the reviewed release and commit.
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

fn await_endpoint(log: &Path, mut child: Child) -> Result<(Child, String), (Child, HarnessError)> {
    let started = Instant::now();
    while started.elapsed() < READY_TIMEOUT {
        if let Ok(Some(status)) = child.try_wait() {
            return Err((
                child,
                HarnessError::invalid(format!("emulator exited before readiness: {status}")),
            ));
        }
        if let Ok(text) = std::fs::read_to_string(log)
            && let Some(endpoint) = text.lines().find_map(endpoint_from_line)
        {
            return Ok((child, endpoint));
        }
        std::thread::sleep(READY_POLL);
    }
    Err((
        child,
        HarnessError::invalid("emulator did not become ready within 30 seconds"),
    ))
}

/// Extract the announced endpoint from one JSON log line.
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

/// Parse `https://127.0.0.1:<port>` and nothing else.
fn loopback_port(endpoint: &str) -> Option<u16> {
    let rest = endpoint.strip_prefix("https://127.0.0.1:")?;
    if rest.is_empty() || !rest.chars().all(|character| character.is_ascii_digit()) {
        return None;
    }
    rest.parse::<u16>().ok().filter(|port| *port != 0)
}

fn home() -> Result<PathBuf, HarnessError> {
    std::env::var_os("HOME").map(PathBuf::from).ok_or_else(|| {
        HarnessError::invalid("HOME is not set; emulator credentials are unfindable")
    })
}
