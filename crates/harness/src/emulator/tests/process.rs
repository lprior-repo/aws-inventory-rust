//! Linux process identity checks that distinguish PID reuse from leaked children.

use std::time::{Duration, Instant};

use super::{ChildOwner, TestResult};

type TestError = Box<dyn std::error::Error>;

#[derive(Clone, Copy)]
pub(super) struct Identity {
    pid: u32,
    start_time: u64,
}

struct ProcessStat {
    state: u8,
    start_time: u64,
}

fn process_stat(pid: u32) -> Result<Option<ProcessStat>, TestError> {
    let text = match std::fs::read_to_string(format!("/proc/{pid}/stat")) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    let (_, fields) = text
        .rsplit_once(')')
        .ok_or("process stat has no command terminator")?;
    let state = fields
        .split_whitespace()
        .next()
        .ok_or("process state missing")?;
    let state = state
        .as_bytes()
        .first()
        .copied()
        .ok_or("process state empty")?;
    let start_time = fields
        .split_whitespace()
        .nth(19)
        .ok_or("process start time missing")?
        .parse()?;
    Ok(Some(ProcessStat { state, start_time }))
}

pub(super) fn capture(owner: &ChildOwner) -> Result<Identity, TestError> {
    let pid = owner.child.id();
    let stat = process_stat(pid)?.ok_or("child disappeared before identity capture")?;
    Ok(Identity {
        pid,
        start_time: stat.start_time,
    })
}

pub(super) fn assert_reaped(identity: Identity) -> TestResult {
    let current = process_stat(identity.pid)?;
    assert_ne!(
        current.map(|stat| stat.start_time),
        Some(identity.start_time),
        "original child {} is still running or unreaped",
        identity.pid
    );
    Ok(())
}

pub(super) fn await_zombie(identity: Identity) -> TestResult {
    let started = Instant::now();
    let observed: Option<TestResult> = (0..5_000)
        .take_while(|_| started.elapsed() < Duration::from_secs(5))
        .find_map(|_| match process_stat(identity.pid) {
            Ok(Some(stat)) if stat.start_time == identity.start_time && stat.state == b'Z' => {
                Some(Ok(()))
            }
            Ok(Some(stat)) if stat.start_time == identity.start_time => {
                std::thread::sleep(Duration::from_millis(1));
                None
            }
            Ok(_) => Some(Err(
                "original child disappeared before becoming a zombie".into()
            )),
            Err(error) => Some(Err(error)),
        });
    observed.ok_or("child did not exit within the process observation bound")?
}
