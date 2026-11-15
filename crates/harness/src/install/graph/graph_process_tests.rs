use super::*;
use std::cell::RefCell;
use std::ffi::OsString;
use std::io::Read;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::time::Duration;

struct Configuration {
    program: PathBuf,
    timeout: Duration,
}

thread_local! {
    static CONFIGURATION: RefCell<Option<Configuration>> = const { RefCell::new(None) };
}

pub(crate) fn download_program(default: &str) -> OsString {
    CONFIGURATION.with(|configuration| configuration.borrow().as_ref()
        .map_or_else(|| OsString::from(default), |configuration| configuration.program.clone().into_os_string()))
}

pub(crate) fn archive_program() -> OsString {
    download_program("bsdtar")
}

pub(crate) fn download_timeout(default: Duration) -> Duration {
    CONFIGURATION.with(|configuration| configuration.borrow().as_ref()
        .map_or(default, |configuration| configuration.timeout))
}

pub(crate) fn archive_timeout() -> Duration {
    download_timeout(Duration::from_secs(30))
}

struct ProcessFixture {
    directory: tempfile::TempDir,
    program: PathBuf,
    descendant: PathBuf,
    witness: File,
}

fn quoted(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    Ok(format!("'{}'", path.to_str().ok_or("non-UTF8 fixture path")?.replace('\'', "'\\''")))
}

fn fifo(path: &Path) -> TestResult {
    if !Command::new("mkfifo").arg(path).status()?.success() {
        return Err("FIFO creation failed".into());
    }
    Ok(())
}

impl ProcessFixture {
    fn new(retain_stdout: bool, action: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let directory = tempfile::tempdir()?;
        let program = directory.path().join("program");
        let descendant = directory.path().join("descendant");
        let witness = directory.path().join("witness");
        let ready = directory.path().join("ready");
        fifo(&witness)?;
        fifo(&ready)?;
        let witness_file = std::fs::OpenOptions::new().read(true)
            .custom_flags(libc::O_NONBLOCK).open(&witness)?;
        let stdout = if retain_stdout { "" } else { "exec 1>/dev/null;" };
        // The handshake guarantees a live witness-owning descendant before the leader exits.
        let script = format!(
            "#!/bin/sh\nif [ -e {descendant} ]; then {action}; fi\n({stdout} exec 3>{witness}; printf 'ready\\n' >&3; printf 'ready\\n' >{ready}; exec sleep 30) &\nprintf '%s\\n' \"$!\" >>{descendant}\nread -r ready <{ready}\n{action}\n",
            witness = quoted(&witness)?, ready = quoted(&ready)?, descendant = quoted(&descendant)?,
        );
        std::fs::write(&program, script)?;
        std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o755))?;
        Ok(Self { directory, program, descendant, witness: witness_file })
    }

    fn bounded_run<F>(&mut self, action: F) -> Result<Result<(), HarnessError>, Box<dyn std::error::Error>>
    where F: FnOnce() -> Result<(), HarnessError> + Send + 'static {
        let program = self.program.clone();
        let (sent, received) = std::sync::mpsc::channel();
        let worker = std::thread::spawn(move || {
            CONFIGURATION.with(|configuration| {
                *configuration.borrow_mut() = Some(Configuration { program, timeout: Duration::from_secs(1) });
            });
            let result = action();
            CONFIGURATION.with(|configuration| drop(configuration.borrow_mut().take()));
            sent.send(result).map_err(std::io::Error::other)
        });
        let result = received.recv_timeout(Duration::from_secs(5));
        let closed = self.closed();
        // Rescue omitted group cleanup or an unbounded pipe join before joining the test worker.
        let rescue = if matches!(&closed, Ok(true)) { Ok(()) } else { self.kill_descendant() };
        let joined = worker.join().map_err(|_| "process fixture worker failed");
        rescue?;
        joined??;
        assert!(closed?, "owned descendant retained its witness after the route completed");
        Ok(result?)
    }

    fn closed(&mut self) -> Result<bool, Box<dyn std::error::Error>> {
        let mut descriptors = [rustix::event::PollFd::new(&self.witness, rustix::event::PollFlags::HUP)];
        let timeout = rustix::event::Timespec { tv_sec: 1, tv_nsec: 0 };
        rustix::event::poll(&mut descriptors, Some(&timeout)).map_err(std::io::Error::from)?;
        let closed = descriptors.first().is_some_and(|descriptor| descriptor.revents().contains(rustix::event::PollFlags::HUP));
        if closed {
            let mut bytes = Vec::new();
            self.witness.read_to_end(&mut bytes)?;
            assert!(bytes.starts_with(b"ready\n"));
        }
        Ok(closed)
    }

    fn kill_descendant(&self) -> TestResult {
        std::fs::read_to_string(&self.descendant)?.lines().take(3).try_for_each(|line| -> TestResult {
            let raw = line.parse::<i32>()?;
            let pid = rustix::process::Pid::from_raw(raw).ok_or("invalid descendant pid")?;
            match rustix::process::kill_process(pid, rustix::process::Signal::KILL) {
                Ok(()) | Err(rustix::io::Errno::SRCH) => Ok(()),
                Err(error) => Err(std::io::Error::from(error).into()),
            }
        })
    }
}

fn download_route(retain_stdout: bool, action: &str) -> TestResult {
    let mut given = ProcessFixture::new(retain_stdout, action)?;
    let output = given.directory.path().join("published");
    let path = output.clone();
    let result = given.bounded_run(move || {
        let digest = crate::provenance::sha256_hex(b"wanted");
        crate::install::graph::download::pinned("fixture://command", &path, &digest)
    })?;
    let succeeds = !retain_stdout && action == "printf wanted; exit 0";
    assert_eq!(result.is_ok(), succeeds);
    if retain_stdout {
        assert!(matches!(&result, Err(HarnessError::Io(error)) if error.kind() == std::io::ErrorKind::TimedOut));
    } else if action == "printf foreign; exit 0" {
        assert!(matches!(&result, Err(HarnessError::Invalid(error)) if error == "pinned download digest mismatch"));
    } else if action.ends_with("exit 7") {
        assert!(matches!(&result, Err(HarnessError::Invalid(error)) if error == "fixture download failed: exit status: 7"));
    }
    if succeeds { assert_eq!(std::fs::read(&output)?, b"wanted"); }
    else { assert!(!output.try_exists()?); }
    Ok(())
}

#[test]
fn given_curl_leader_exit_with_inherited_output_when_public_download_times_out_then_descendant_is_terminated() -> TestResult {
    download_route(true, "printf wanted; exit 0")
}

#[test]
fn given_curl_success_with_closed_descendant_output_when_public_download_finishes_then_group_is_terminated() -> TestResult {
    download_route(false, "printf wanted; exit 0")
}

#[test]
fn given_curl_failed_status_when_public_download_finishes_then_descendant_is_terminated() -> TestResult {
    download_route(false, "printf wanted; exit 7")
}

#[test]
fn given_curl_digest_failure_when_public_download_rejects_then_descendant_is_terminated() -> TestResult {
    download_route(false, "printf foreign; exit 0")
}

fn archive_route(retain_stdout: bool) -> TestResult {
    let (given, source, _input) = fixture(&[member("redis-8.10.3/kept", b"verified")])?;
    let path = given.path().join("input.tar");
    let digest = crate::provenance::sha256_hex(&std::fs::read(&path)?);
    let output = source.clone();
    let mut process = ProcessFixture::new(retain_stdout, "exec bsdtar \"$@\"")?;
    let result = process.bounded_run(move || extract_pinned(&path, &output, &digest).map(|_| ()))?;
    if retain_stdout {
        assert!(matches!(result, Err(HarnessError::Invalid(error)) if error == "unexpected Redis archive bsdtar deadline"));
        assert!(!source.try_exists()?);
    } else {
        result?;
        assert_eq!(std::fs::read(source.join("kept"))?, b"verified");
    }
    Ok(())
}

#[test]
fn given_tar_leader_exit_with_inherited_output_when_extraction_times_out_then_reader_and_group_finish() -> TestResult {
    archive_route(true)
}

#[test]
fn given_tar_success_with_closed_descendant_output_when_extraction_finishes_then_each_group_is_terminated() -> TestResult {
    archive_route(false)
}
