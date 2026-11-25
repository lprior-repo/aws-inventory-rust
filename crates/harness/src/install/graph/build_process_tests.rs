use std::fs::{self, File};
use std::io::{Read, Write};
use std::os::fd::OwnedFd;
use std::os::unix::net::UnixStream;
use std::process::Stdio;
use std::time::Duration;

use super::super::OwnedDirectory;
use super::super::installer_tests::TestResult;
use crate::HarnessError;

struct EmergencyRelease(File);

impl Drop for EmergencyRelease {
    fn drop(&mut self) {
        if let Err(error) = self.0.write_all(b"release\n") {
            eprintln!("make fixture release failed: {error}");
        }
    }
}

struct BuildOutcome {
    result: Result<(), HarnessError>,
    diagnostics: Vec<u8>,
}

fn background_writer_case(
    final_recipe: &str,
    timeout: Duration,
) -> Result<BuildOutcome, Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let release_path = directory.path().join("release");
    ["ready", "release", "parent_release"]
        .into_iter()
        .try_for_each(|name| {
            rustix::fs::mkfifoat(
                rustix::fs::CWD,
                directory.path().join(name),
                rustix::fs::Mode::RWXU,
            )
        })?;
    let mut release = EmergencyRelease(File::from(rustix::fs::open(
        &release_path,
        rustix::fs::OFlags::RDWR | rustix::fs::OFlags::NONBLOCK | rustix::fs::OFlags::CLOEXEC,
        rustix::fs::Mode::empty(),
    )?));
    fs::write(
        directory.path().join("Makefile"),
        format!(
            ".PHONY: redis-server redis-cli\nredis-server:\n\t@(printf ready >&2; printf 'token\\n' > ready; read token < release; printf leaked > marker) & read token < ready\n\t@{final_recipe}\nredis-cli:\n\t@:\n"
        ),
    )?;
    let source = OwnedDirectory::capture(directory.path())?;
    let (mut handshake, output) = UnixStream::pair()?;
    handshake.set_read_timeout(Some(Duration::from_secs(2)))?;
    let mut command = super::compile_command(&source, 1);
    command
        .stdout(Stdio::null())
        .stderr(Stdio::from(OwnedFd::from(output)));
    let result = super::run_compile(command, timeout);
    let mut ready = [0; 5];
    handshake.read_exact(&mut ready)?;
    assert_eq!(&ready, b"ready");
    let mut diagnostics = Vec::new();
    let observed = (&mut handshake).take(4_097).read_to_end(&mut diagnostics);
    release.0.write_all(b"release\n")?;
    let mut remaining = Vec::new();
    (&mut handshake).take(4_097).read_to_end(&mut remaining)?;
    assert!(
        remaining.len() < 4_097,
        "make fixture diagnostics exceed limit"
    );
    assert!(
        !directory.path().join("marker").exists(),
        "writer mutated source after compile returned"
    );
    observed?;
    assert!(
        diagnostics.len() < 4_097,
        "make fixture diagnostics exceed limit"
    );
    Ok(BuildOutcome {
        result,
        diagnostics,
    })
}

#[test]
fn given_background_writer_when_make_succeeds_then_no_descendant_can_write_after_return()
-> TestResult {
    let outcome = background_writer_case("exit 0", Duration::from_secs(2))?;
    assert!(
        outcome.result.is_ok(),
        "make success fixture failed: {:?}; stderr: {}",
        outcome.result,
        String::from_utf8_lossy(&outcome.diagnostics),
    );
    Ok(())
}

#[test]
fn given_background_writer_when_make_fails_then_no_descendant_can_write_after_return() -> TestResult
{
    let outcome = background_writer_case("exit 7", Duration::from_secs(2))?;
    let error = outcome.result.err().ok_or("failing make succeeded")?;
    let diagnostics = String::from_utf8_lossy(&outcome.diagnostics);
    assert!(
        error.to_string().contains("exit status: 2"),
        "{error}; stderr: {diagnostics}"
    );
    assert!(
        !error.to_string().contains("wall deadline"),
        "{error}; stderr: {diagnostics}"
    );
    assert!(
        diagnostics.contains("Error 7"),
        "requested failing recipe was not reached: {diagnostics}"
    );
    Ok(())
}

#[test]
fn given_stalled_make_when_wall_deadline_expires_then_leader_and_writer_are_terminated()
-> TestResult {
    let outcome =
        background_writer_case("read token < parent_release", Duration::from_millis(150))?;
    let error = outcome.result.err().ok_or("stalled make succeeded")?;
    assert!(
        error.to_string().contains("wall deadline expired"),
        "{error}; stderr: {}",
        String::from_utf8_lossy(&outcome.diagnostics),
    );
    Ok(())
}
