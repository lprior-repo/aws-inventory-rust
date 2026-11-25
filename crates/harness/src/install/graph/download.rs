use std::ffi::OsStr;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::ops::ControlFlow;
use std::os::fd::OwnedFd;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::net::UnixStream;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

use rustix::fs::{AtFlags, Mode, OFlags};
use sha2::{Digest, Sha256};

use super::MAX_DOWNLOAD;
use crate::HarnessError;

#[cfg(test)]
#[path = "download_tests.rs"]
mod tests;

const LIFETIME: Duration = Duration::from_secs(180);

struct DownloadBudget {
    started: Instant,
    max_bytes: u64,
    lifetime: Duration,
}

struct DeadlineInput<'a> {
    input: &'a mut UnixStream,
    budget: &'a DownloadBudget,
}

impl Read for DeadlineInput<'_> {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        loop {
            let duration = remaining(self.budget)
                .map_err(|error| std::io::Error::new(std::io::ErrorKind::TimedOut, error))?;
            self.input.set_read_timeout(Some(duration))?;
            match self.input.read(buffer) {
                Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {}
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::TimedOut,
                        "fixture download deadline exceeded",
                    ));
                }
                result => return result,
            }
        }
    }
}

pub(super) fn pinned(url: &str, path: &Path, digest: &str) -> Result<(), HarnessError> {
    pinned_with_limit(url, path, digest, MAX_DOWNLOAD)
}

fn pinned_with_limit(
    url: &str,
    path: &Path,
    digest: &str,
    max_bytes: u64,
) -> Result<(), HarnessError> {
    let directory = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty());
    let directory = directory.map_or_else(|| Path::new("."), |parent| parent);
    let name = path
        .file_name()
        .ok_or_else(|| HarnessError::invalid("download has no name"))?;
    let parent = File::from(
        rustix::fs::open(directory, super::DIRECTORY_FLAGS, Mode::empty())
            .map_err(std::io::Error::from)?,
    );
    let anchored = super::descriptor_path(&parent).join(name);
    if existing(&anchored, digest)? {
        return Ok(());
    }
    let flags = OFlags::RDWR | OFlags::TMPFILE | OFlags::CLOEXEC;
    let mut temporary = File::from(
        rustix::fs::openat(&parent, ".", flags, Mode::RUSR | Mode::WUSR)
            .map_err(std::io::Error::from)?,
    );
    fetch(url, &mut temporary, digest, max_bytes.min(MAX_DOWNLOAD))?;
    publish(&temporary, &parent, name, digest)
}

pub(super) fn existing(path: &Path, digest: &str) -> Result<bool, HarnessError> {
    let Some(mut file) = open_cached(path)? else {
        return Ok(false);
    };
    if copy_body(&mut file, &mut std::io::sink(), MAX_DOWNLOAD)? == digest {
        return Ok(true);
    }
    Err(HarnessError::invalid(format!(
        "existing pinned download has a different digest: {}",
        path.display()
    )))
}

fn open_cached(path: &Path) -> Result<Option<File>, HarnessError> {
    let opened = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NONBLOCK | libc::O_NOFOLLOW)
        .open(path);
    let file = match opened {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    let metadata = file.metadata()?;
    if !metadata.file_type().is_file() || metadata.len() > MAX_DOWNLOAD {
        return Err(HarnessError::invalid(format!(
            "fixture path is not a bounded regular file: {}",
            path.display()
        )));
    }
    Ok(Some(file))
}

fn publish(
    temporary: &File,
    parent: &File,
    name: &OsStr,
    digest: &str,
) -> Result<(), HarnessError> {
    temporary.sync_all()?;
    #[cfg(test)]
    tests::races::publication_checkpoint(temporary, true)?;
    let linked = rustix::fs::linkat(
        rustix::fs::CWD,
        super::descriptor_path(temporary),
        parent,
        name,
        AtFlags::SYMLINK_FOLLOW,
    )
    .map_err(std::io::Error::from);
    match linked {
        Ok(()) => {
            #[cfg(test)]
            tests::races::publication_checkpoint(temporary, false)?;
            parent.sync_all().map_err(Into::into)
        }
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            require_existing(&super::descriptor_path(parent).join(name), digest)
        }
        Err(error) => Err(error.into()),
    }
}

fn require_existing(path: &Path, digest: &str) -> Result<(), HarnessError> {
    if existing(path, digest)? {
        return Ok(());
    }
    Err(HarnessError::invalid("competing pinned download vanished"))
}

fn curl(url: &str, output: UnixStream) -> Command {
    let program = "curl";
    #[cfg(test)]
    let program = super::archive::download_program(program);
    let mut command = Command::new(program);
    command
        .process_group(0)
        .arg("-q")
        .args([
            "--fail",
            "--location",
            "--silent",
            "--show-error",
            "--globoff",
        ])
        .args(["--max-time", "180", "--url"])
        .arg(url)
        .stdin(Stdio::null())
        .stdout(Stdio::from(OwnedFd::from(output)));
    command
}

fn fetch(url: &str, file: &mut File, digest: &str, max_bytes: u64) -> Result<(), HarnessError> {
    let (mut input, output) = UnixStream::pair()?;
    let lifetime = LIFETIME;
    #[cfg(test)]
    let lifetime = super::archive::download_timeout(lifetime);
    let budget = DownloadBudget {
        started: Instant::now(),
        max_bytes,
        lifetime,
    };
    let mut child = curl(url, output).spawn()?;
    let copied = copy_body(
        &mut DeadlineInput {
            input: &mut input,
            budget: &budget,
        },
        file,
        budget.max_bytes,
    );
    drop(input);
    let finished = match &copied {
        Ok(_) => wait_status(&mut child, &budget),
        Err(_) => Ok(()),
    };
    let completed = copied.is_ok() && finished.is_ok();
    let reaped = super::build::terminate_and_reap(&mut child).and_then(|status| {
        if completed {
            check_status(status)
        } else {
            Ok(())
        }
    });
    let actual = combine(combine(copied, finished), reaped)?;
    if actual != digest {
        return Err(HarnessError::invalid("pinned download digest mismatch"));
    }
    Ok(())
}

fn remaining(budget: &DownloadBudget) -> Result<Duration, HarnessError> {
    budget
        .lifetime
        .checked_sub(budget.started.elapsed())
        .filter(|remaining| !remaining.is_zero())
        .ok_or_else(|| HarnessError::invalid("fixture download deadline exceeded"))
}

fn copy_body<R: Read, W: Write>(
    input: &mut R,
    file: &mut W,
    max_bytes: u64,
) -> Result<String, HarnessError> {
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 16_384];
    let copied = (0..=max_bytes).try_fold(0_u64, |total, _| {
        match copy_chunk(input, file, &mut hasher, &mut buffer, (total, max_bytes)) {
            Ok(0) => ControlFlow::Break(Ok(())),
            Ok(count) => ControlFlow::Continue(total.saturating_add(count)),
            Err(error) => ControlFlow::Break(Err(error)),
        }
    });
    match copied {
        ControlFlow::Break(result) => result?,
        ControlFlow::Continue(_) => {
            return Err(HarnessError::invalid(
                "fixture download size limit exceeded",
            ));
        }
    }
    Ok(format!("{:x}", base16ct::HexDisplay(&hasher.finalize())))
}

fn copy_chunk<R: Read, W: Write>(
    input: &mut R,
    file: &mut W,
    hasher: &mut Sha256,
    buffer: &mut [u8],
    state: (u64, u64),
) -> Result<u64, HarnessError> {
    let available = state.1.saturating_sub(state.0).saturating_add(1);
    let count = input.take(available).read(buffer)?;
    let count_u64 = u64::try_from(count)
        .map_err(|_| HarnessError::invalid("fixture download byte count overflow"))?;
    if state
        .0
        .checked_add(count_u64)
        .is_none_or(|total| total > state.1)
    {
        return Err(HarnessError::invalid(
            "fixture download size limit exceeded",
        ));
    }
    let bytes = buffer
        .get(..count)
        .ok_or_else(|| HarnessError::invalid("fixture download buffer length invalid"))?;
    file.write_all(bytes)?;
    hasher.update(bytes);
    Ok(count_u64)
}

fn wait_status(child: &mut Child, budget: &DownloadBudget) -> Result<(), HarnessError> {
    let result = (0..18_001).find_map(|_| {
        match super::build::exited(rustix::process::Pid::from_child(child)) {
            Ok(true) => Some(Ok(())),
            Err(error) => Some(Err(error)),
            Ok(false) => match remaining(budget) {
                Err(error) => Some(Err(error)),
                Ok(duration) => {
                    std::thread::sleep(duration.min(Duration::from_millis(10)));
                    None
                }
            },
        }
    });
    result.ok_or_else(|| HarnessError::invalid("fixture download wait limit exceeded"))?
}

fn check_status(status: ExitStatus) -> Result<(), HarnessError> {
    if status.success() {
        return Ok(());
    }
    Err(HarnessError::invalid(format!(
        "fixture download failed: {status}"
    )))
}

fn combine<T>(
    result: Result<T, HarnessError>,
    cleanup: Result<(), HarnessError>,
) -> Result<T, HarnessError> {
    match (result, cleanup) {
        (result, Ok(())) => result,
        (Ok(_), Err(error)) => Err(error),
        (Err(error), Err(cleanup)) => Err(HarnessError::invalid(format!(
            "{error}; cleanup failed: {cleanup}"
        ))),
    }
}
