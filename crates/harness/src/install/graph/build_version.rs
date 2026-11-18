use std::fs::File;
use std::io::Read;
use std::ops::ControlFlow;
use std::os::unix::process::CommandExt;
use std::process::{Child, ChildStdout, Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

use rustix::process::Pid;

use super::build::{exited, terminate_and_reap};
use super::{complete, descriptor_path};
use crate::HarnessError;

const VERSION_TIMEOUT: Duration = Duration::from_secs(5);
const OUTPUT_LIMIT: usize = 4_096;
const POLL_BUDGET: usize = 100_000;
const POLL_INTERVAL: Duration = Duration::from_millis(10);

pub(super) fn version(binary: &File) -> Result<String, HarnessError> {
    let mut child = Command::new(descriptor_path(binary))
        .arg("--version")
        .process_group(0)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    probe(&mut child, VERSION_TIMEOUT)
}

fn probe(child: &mut Child, timeout: Duration) -> Result<String, HarnessError> {
    let result = read_version(child, timeout);
    complete(result, terminate_and_reap(child).and_then(version_status))
}

struct VersionOutput {
    bytes: [u8; 4_097],
    length: usize,
    eof: bool,
}

impl VersionOutput {
    fn read(&mut self, stdout: &mut ChildStdout) -> Result<(), HarnessError> {
        if self.eof {
            return Ok(());
        }
        let remaining = self
            .bytes
            .get_mut(self.length..)
            .ok_or_else(|| HarnessError::invalid("invalid version output length"))?;
        match stdout.read(remaining) {
            Ok(0) => self.eof = true,
            Ok(count) => self.length = self.length.saturating_add(count),
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::Interrupted
                ) => {}
            Err(error) => return Err(error.into()),
        }
        if self.length > OUTPUT_LIMIT {
            return Err(HarnessError::invalid(
                "Redis version output exceeds byte limit",
            ));
        }
        Ok(())
    }

    fn text(&self) -> Result<String, HarnessError> {
        let bytes = self
            .bytes
            .get(..self.length)
            .ok_or_else(|| HarnessError::invalid("invalid version output length"))?;
        let text = std::str::from_utf8(bytes).map_err(|error| {
            HarnessError::invalid(format!("invalid Redis version UTF-8: {error}"))
        })?;
        Ok(text.trim().to_owned())
    }
}

fn read_version(child: &mut Child, timeout: Duration) -> Result<String, HarnessError> {
    let deadline = Instant::now()
        .checked_add(timeout)
        .ok_or_else(|| HarnessError::invalid("invalid version deadline"))?;
    let mut stdout = child
        .stdout
        .take()
        .ok_or_else(|| HarnessError::invalid("version stdout missing"))?;
    let flags = rustix::fs::fcntl_getfl(&stdout).map_err(std::io::Error::from)?;
    rustix::fs::fcntl_setfl(&stdout, flags | rustix::fs::OFlags::NONBLOCK)
        .map_err(std::io::Error::from)?;
    let mut output = VersionOutput {
        bytes: [0; 4_097],
        length: 0,
        eof: false,
    };
    let pid = Pid::from_child(child);
    match (0..POLL_BUDGET).try_for_each(|_| version_step(&mut stdout, pid, deadline, &mut output)) {
        ControlFlow::Break(result) => result.and_then(|()| output.text()),
        ControlFlow::Continue(()) => Err(HarnessError::invalid(
            "Redis version polling exceeds iteration limit",
        )),
    }
}

fn version_step(
    stdout: &mut ChildStdout,
    pid: Pid,
    deadline: Instant,
    output: &mut VersionOutput,
) -> ControlFlow<Result<(), HarnessError>> {
    match advance(stdout, pid, deadline, output) {
        Ok(true) => ControlFlow::Break(Ok(())),
        Ok(false) => ControlFlow::Continue(()),
        Err(error) => ControlFlow::Break(Err(error)),
    }
}

fn advance(
    stdout: &mut ChildStdout,
    pid: Pid,
    deadline: Instant,
    output: &mut VersionOutput,
) -> Result<bool, HarnessError> {
    let remaining = deadline
        .checked_duration_since(Instant::now())
        .ok_or_else(|| {
            HarnessError::invalid(
                "Redis version command failed or timed out: wall deadline expired",
            )
        })?;
    output.read(stdout)?;
    if output.eof && exited(pid)? {
        return Ok(true);
    }
    wait_readable(stdout, output.eof, remaining.min(POLL_INTERVAL))?;
    Ok(false)
}

fn wait_readable(stdout: &ChildStdout, eof: bool, duration: Duration) -> Result<(), HarnessError> {
    let timeout = rustix::event::Timespec::try_from(duration).map_err(|error| {
        HarnessError::invalid(format!("invalid version poll duration: {error}"))
    })?;
    let mut descriptors = [rustix::event::PollFd::new(
        stdout,
        rustix::event::PollFlags::IN,
    )];
    let active = if eof {
        descriptors.get_mut(..0)
    } else {
        descriptors.get_mut(..1)
    }
    .ok_or_else(|| HarnessError::invalid("invalid version poll descriptors"))?;
    match rustix::event::poll(active, Some(&timeout)) {
        Ok(_) => Ok(()),
        Err(rustix::io::Errno::INTR) => Ok(()),
        Err(error) => Err(std::io::Error::from(error).into()),
    }
}

fn version_status(status: ExitStatus) -> Result<(), HarnessError> {
    if status.success() {
        return Ok(());
    }
    Err(HarnessError::invalid(format!(
        "Redis version command failed or timed out: {status}"
    )))
}

#[cfg(test)]
#[path = "build_version_tests.rs"]
mod tests;
