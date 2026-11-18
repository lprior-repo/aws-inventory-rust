use std::fs::File;
use std::io::{Read, Seek};
use std::ops::ControlFlow;
use std::os::unix::process::CommandExt;
use std::process::{Child, ChildStdout, Command, Stdio};
use std::time::{Duration, Instant};

use rustix::process::Pid;

use super::super::{build, complete};
use super::{MAX_LISTING, reject};
use crate::HarnessError;

const POLL_INTERVAL: Duration = Duration::from_millis(10);
const POLL_BUDGET: usize = MAX_LISTING.saturating_add(30_001);

pub(super) fn run(
    command: &mut Command,
    file: &mut File,
    timeout: Duration,
) -> Result<Vec<u8>, HarnessError> {
    file.rewind()?;
    let mut child = command
        .env("LC_ALL", "C")
        .process_group(0)
        .stdin(Stdio::from(file.try_clone()?))
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    let output = capture(&mut child, timeout);
    // The leader stays unreaped until its whole owned group is signalled, even on success.
    let cleanup = build::terminate_and_reap(&mut child).and_then(|status| {
        if status.success() {
            Ok(())
        } else {
            Err(reject("bsdtar failed"))
        }
    });
    complete(output, cleanup)
}

struct Output {
    bytes: Vec<u8>,
    eof: bool,
}

impl Output {
    fn new() -> Result<Self, HarnessError> {
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(MAX_LISTING.saturating_add(1))
            .map_err(|_| reject("listing allocation"))?;
        Ok(Self { bytes, eof: false })
    }

    fn read(&mut self, stdout: &mut ChildStdout) -> Result<(), HarnessError> {
        if self.eof {
            return Ok(());
        }
        let available = MAX_LISTING
            .saturating_add(1)
            .saturating_sub(self.bytes.len());
        let mut buffer = [0_u8; 16_384];
        let capacity = available.min(buffer.len());
        let buffer = buffer
            .get_mut(..capacity)
            .ok_or_else(|| reject("listing buffer bound"))?;
        match stdout.read(buffer) {
            Ok(0) => self.eof = true,
            Ok(count) => self.bytes.extend_from_slice(
                buffer
                    .get(..count)
                    .ok_or_else(|| reject("listing buffer length"))?,
            ),
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::Interrupted
                ) => {}
            Err(error) => return Err(error.into()),
        }
        if self.bytes.len() > MAX_LISTING {
            return Err(reject("listing byte bound"));
        }
        Ok(())
    }
}

fn capture(child: &mut Child, timeout: Duration) -> Result<Vec<u8>, HarnessError> {
    let deadline = Instant::now()
        .checked_add(timeout)
        .ok_or_else(|| reject("invalid listing deadline"))?;
    let mut stdout = child
        .stdout
        .take()
        .ok_or_else(|| reject("missing listing pipe"))?;
    let flags = rustix::fs::fcntl_getfl(&stdout).map_err(std::io::Error::from)?;
    rustix::fs::fcntl_setfl(&stdout, flags | rustix::fs::OFlags::NONBLOCK)
        .map_err(std::io::Error::from)?;
    let mut output = Output::new()?;
    let pid = Pid::from_child(child);
    let result =
        (0..POLL_BUDGET).try_for_each(|_| match advance(&mut stdout, pid, deadline, &mut output) {
            Ok(true) => ControlFlow::Break(Ok(())),
            Ok(false) => ControlFlow::Continue(()),
            Err(error) => ControlFlow::Break(Err(error)),
        });
    match result {
        ControlFlow::Break(result) => result.map(|()| output.bytes),
        ControlFlow::Continue(()) => Err(reject("listing polling iteration limit")),
    }
}

fn advance(
    stdout: &mut ChildStdout,
    pid: Pid,
    deadline: Instant,
    output: &mut Output,
) -> Result<bool, HarnessError> {
    let remaining = deadline
        .checked_duration_since(Instant::now())
        .ok_or_else(|| reject("bsdtar deadline"))?;
    output.read(stdout)?;
    if output.eof && build::exited(pid)? {
        return Ok(true);
    }
    let timeout = rustix::event::Timespec::try_from(remaining.min(POLL_INTERVAL))
        .map_err(|error| reject(&format!("listing poll duration: {error}")))?;
    let mut descriptors = [rustix::event::PollFd::new(
        stdout,
        rustix::event::PollFlags::IN,
    )];
    let active = if output.eof {
        descriptors.get_mut(..0)
    } else {
        descriptors.get_mut(..1)
    }
    .ok_or_else(|| reject("listing poll descriptors"))?;
    match rustix::event::poll(active, Some(&timeout)) {
        Ok(_) | Err(rustix::io::Errno::INTR) => Ok(false),
        Err(error) => Err(std::io::Error::from(error).into()),
    }
}
