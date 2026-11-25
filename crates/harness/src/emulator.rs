use std::io::Write;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use serde_json::Value;

use crate::HarnessError;
use rustix::process::{Pid, Signal, WaitId, WaitIdOptions, WaitIdStatus};

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

pub(crate) struct ChildOwner {
    child: Child,
    finished: OnceLock<FinishedChild>,
}

enum FinishedChild {
    Lost,
    Reaped(std::process::ExitStatus),
}

impl ChildOwner {
    pub(crate) fn spawn(command: &mut Command) -> std::io::Result<Self> {
        ensure_waitable_children()?;
        let child = command.process_group(0).spawn()?;
        Ok(Self {
            child,
            finished: OnceLock::new(),
        })
    }

    pub(crate) fn pid(&self) -> Pid {
        Pid::from_child(&self.child)
    }

    pub(crate) fn exited(&self) -> Result<Option<WaitIdStatus>, HarnessError> {
        if self.finished.get().is_some() {
            return Err(std::io::Error::from(rustix::io::Errno::CHILD).into());
        }
        let flags = WaitIdOptions::EXITED | WaitIdOptions::NOHANG | WaitIdOptions::NOWAIT;
        let result = rustix::process::waitid(WaitId::Pid(self.pid()), flags);
        if matches!(result, Err(rustix::io::Errno::CHILD)) {
            self.finished.get_or_init(|| FinishedChild::Lost);
        }
        result.map_err(|error| std::io::Error::from(error).into())
    }

    pub(crate) fn terminate(&mut self) -> Result<std::process::ExitStatus, HarnessError> {
        if let Some(FinishedChild::Reaped(status)) = self.finished.get() {
            return Ok(*status);
        }
        self.exited()?;
        match rustix::process::kill_process_group(self.pid(), Signal::KILL) {
            Ok(()) | Err(rustix::io::Errno::SRCH) => {}
            Err(error) => return Err(std::io::Error::from(error).into()),
        }
        match self.child.wait() {
            Ok(status) => {
                self.finished.get_or_init(|| FinishedChild::Reaped(status));
                Ok(status)
            }
            Err(error) => {
                if error.raw_os_error() == Some(rustix::io::Errno::CHILD.raw_os_error()) {
                    self.finished.get_or_init(|| FinishedChild::Lost);
                }
                Err(error.into())
            }
        }
    }

    #[cfg(test)]
    pub(crate) fn child_mut(&mut self) -> &mut Child {
        &mut self.child
    }
}

fn ensure_waitable_children() -> std::io::Result<()> {
    static INSTALLED: OnceLock<()> = OnceLock::new();
    if INSTALLED.get().is_some() {
        return Ok(());
    }

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .build()?;
    let installed = {
        let _entered = runtime.enter();
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::child()).map(drop)
    };
    runtime.shutdown_background();
    installed?;
    INSTALLED.get_or_init(|| ());
    Ok(())
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
        "owned child cleanup failed: {error}"
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

fn command(binary: &Path) -> Command {
    let mut command = Command::new(binary);
    command
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("EMULATOR_TELEMETRY", "off")
        .stdin(Stdio::null());
    command
}

fn spawn(
    binary: &Path,
    log: &Path,
    home: &Path,
    idle_timeout_seconds: u64,
) -> Result<ChildOwner, HarnessError> {
    let handle = std::fs::File::create(log)?;
    ChildOwner::spawn(
        command(binary)
            .args(ARGUMENTS)
            .arg(idle_timeout_seconds.to_string())
            .env("HOME", home)
            .stdout(Stdio::from(handle.try_clone()?))
            .stderr(Stdio::from(handle)),
    )
    .map_err(HarnessError::from)
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
        self.child.terminate().map(drop)
    }
}

pub fn verify_binary(binary: &Path) -> Result<(), HarnessError> {
    ensure_waitable_children()?;
    let output = command(binary).arg("--version").output()?;
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
    if let Some(status) = child.exited()? {
        return Err(HarnessError::invalid(format!(
            "emulator exited before readiness: {status:?}"
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
