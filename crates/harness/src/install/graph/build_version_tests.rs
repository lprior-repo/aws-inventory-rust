use std::fs::File;
use std::io::{Read, Write};
use std::os::fd::OwnedFd;
use std::os::unix::net::UnixStream;
use std::os::unix::process::CommandExt;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use rustix::process::{Pid, WaitId, WaitIdOptions};

use super::super::installer_tests::TestResult;

struct Fixture {
    _directory: tempfile::TempDir,
    child: Child,
    handshake: UnixStream,
    release: File,
    finalized: bool,
}

impl Fixture {
    fn descendant(close_stdout: bool) -> Result<Self, Box<dyn std::error::Error>> {
        let directory = tempfile::tempdir()?;
        let fifo = directory.path().join("release");
        rustix::fs::mkfifoat(rustix::fs::CWD, &fifo, rustix::fs::Mode::RWXU)?;
        let release = release_file(&fifo)?;
        let (handshake, output) = UnixStream::pair()?;
        handshake.set_read_timeout(Some(Duration::from_secs(2)))?;
        let close = if close_stdout { "exec 1>&-;" } else { "" };
        let script = format!(
            "( {close} printf ready >&2; read release < \"$1\" ) & printf 'redis-cli 8.10.3\\n'; exit 0"
        );
        let child = Command::new("sh")
            .args(["-c", &script, "version-fixture"])
            .arg(&fifo)
            .process_group(0)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::from(OwnedFd::from(output)))
            .spawn()?;
        let mut fixture = Self {
            _directory: directory,
            child,
            handshake,
            release,
            finalized: false,
        };
        let mut ready = [0; 5];
        fixture.handshake.read_exact(&mut ready)?;
        assert_eq!(&ready, b"ready");
        let status = rustix::process::waitid(
            WaitId::Pid(Pid::from_child(&fixture.child)),
            WaitIdOptions::EXITED | WaitIdOptions::NOWAIT,
        )?;
        assert!(
            status.is_some(),
            "direct child must exit before the probe starts"
        );
        Ok(fixture)
    }

    fn probe(&mut self, timeout: Duration) -> Result<String, crate::HarnessError> {
        let result = super::probe(&mut self.child, timeout);
        self.finalized = true;
        result
    }

    fn assert_reaped_and_closed(&mut self) -> TestResult {
        assert_eq!(
            self.handshake.read(&mut [0; 1])?,
            0,
            "descendant still holds its handshake FD"
        );
        assert_eq!(
            rustix::process::waitid(
                WaitId::Pid(Pid::from_child(&self.child)),
                WaitIdOptions::EXITED | WaitIdOptions::NOHANG | WaitIdOptions::NOWAIT,
            )
            .err(),
            Some(rustix::io::Errno::CHILD),
            "owned direct child was not reaped"
        );
        Ok(())
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        if let Err(error) = self.release.write_all(b"release\n") {
            eprintln!("fixture FIFO release failed: {error}");
        }
        if !self.finalized {
            match rustix::process::kill_process_group(
                Pid::from_child(&self.child),
                rustix::process::Signal::KILL,
            ) {
                Ok(()) | Err(rustix::io::Errno::SRCH) => {}
                Err(error) => eprintln!("fixture process group cleanup failed: {error}"),
            }
            if let Err(error) = self.child.wait() {
                eprintln!("fixture child reap failed: {error}");
            }
        }
        let mut remaining = Vec::new();
        if let Err(error) = (&mut self.handshake).take(6).read_to_end(&mut remaining) {
            eprintln!("fixture descendant drain failed: {error}");
        }
    }
}

fn release_file(path: &std::path::Path) -> Result<File, std::io::Error> {
    rustix::fs::open(
        path,
        rustix::fs::OFlags::RDWR | rustix::fs::OFlags::NONBLOCK | rustix::fs::OFlags::CLOEXEC,
        rustix::fs::Mode::empty(),
    )
    .map(File::from)
    .map_err(std::io::Error::from)
}

#[test]
fn given_exited_leader_with_descendant_stdout_when_probed_then_wall_deadline_kills_group()
-> TestResult {
    let mut fixture = Fixture::descendant(false)?;
    let started = Instant::now();
    let error = fixture
        .probe(Duration::from_millis(150))
        .err()
        .ok_or("descendant-held stdout was accepted")?;
    assert!(error.to_string().contains("wall deadline expired"));
    assert!(started.elapsed() < Duration::from_secs(2));
    fixture.assert_reaped_and_closed()
}

#[test]
fn given_descendant_closed_stdout_when_leader_exits_then_success_still_kills_owned_group()
-> TestResult {
    let mut fixture = Fixture::descendant(true)?;
    assert_eq!(fixture.probe(Duration::from_secs(1))?, "redis-cli 8.10.3");
    fixture.assert_reaped_and_closed()
}
#[test]
fn given_live_leader_closing_stdout_when_probed_then_exit_wait_has_same_wall_deadline() -> TestResult
{
    let directory = tempfile::tempdir()?;
    let fifo = directory.path().join("release");
    rustix::fs::mkfifoat(rustix::fs::CWD, &fifo, rustix::fs::Mode::RWXU)?;
    let release = release_file(&fifo)?;
    let (handshake, output) = UnixStream::pair()?;
    handshake.set_read_timeout(Some(Duration::from_secs(2)))?;
    let child = Command::new("sh")
        .args([
            "-c",
            "exec 1>&-; printf ready >&2; read release < \"$1\"",
            "version-fixture",
        ])
        .arg(&fifo)
        .process_group(0)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::from(OwnedFd::from(output)))
        .spawn()?;
    let mut fixture = Fixture {
        _directory: directory,
        child,
        handshake,
        release,
        finalized: false,
    };
    let mut ready = [0; 5];
    fixture.handshake.read_exact(&mut ready)?;
    assert_eq!(&ready, b"ready");
    let error = fixture
        .probe(Duration::from_millis(150))
        .err()
        .ok_or("live leader with closed stdout was accepted")?;
    assert!(error.to_string().contains("wall deadline expired"));
    fixture.assert_reaped_and_closed()
}

#[test]
fn given_reaped_leader_without_group_cleanup_when_fixture_drops_then_descendant_is_released()
-> TestResult {
    let mut fixture = Fixture::descendant(true)?;
    let mut handshake = fixture.handshake.try_clone()?;
    let mut emergency_release = fixture.release.try_clone()?;
    let status = fixture.child.wait();
    fixture.finalized = true;
    assert!(status?.success());
    drop(fixture);

    let observed = handshake.read(&mut [0; 1]);
    emergency_release.write_all(b"release\n")?;
    assert_eq!(handshake.read(&mut [0; 1])?, 0);
    assert_eq!(observed?, 0, "fixture Drop left its descendant alive");
    Ok(())
}
