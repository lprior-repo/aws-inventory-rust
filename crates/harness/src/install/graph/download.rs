use std::fs::File;
use std::io::{Read, Write};
use std::ops::ControlFlow;
use std::os::fd::OwnedFd;
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

use sha2::{Digest, Sha256};
use tempfile::NamedTempFile;

use super::MAX_DOWNLOAD;
use crate::HarnessError;
use crate::provenance::file_sha256;

#[cfg(test)]
#[path = "download_tests.rs"]
mod tests;

const LIFETIME: Duration = Duration::from_secs(180);

struct DownloadBudget {
    started: Instant,
    max_bytes: u64,
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
    if existing(path, digest)? {
        return Ok(());
    }
    let parent = path.parent().filter(|parent| !parent.as_os_str().is_empty());
    let parent = parent.map_or_else(|| Path::new("."), |parent| parent);
    let mut temporary = NamedTempFile::new_in(parent)?;
    let fetched = fetch(url, temporary.as_file_mut(), digest, max_bytes.min(MAX_DOWNLOAD))
        .and_then(|()| publish(&temporary, path, digest));
    combine(fetched, temporary.close().map_err(Into::into))
}

pub(super) fn existing(path: &Path, digest: &str) -> Result<bool, HarnessError> {
    if !bounded_file(path)? {
        return Ok(false);
    }
    if file_sha256(path)? == digest {
        return Ok(true);
    }
    Err(HarnessError::invalid(format!(
        "existing pinned download has a different digest: {}",
        path.display()
    )))
}

fn bounded_file(path: &Path) -> Result<bool, HarnessError> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_file() && metadata.len() <= MAX_DOWNLOAD => {
            Ok(true)
        }
        Ok(_) => Err(HarnessError::invalid(format!(
            "fixture path is not a bounded regular file: {}",
            path.display()
        ))),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error.into()),
    }
}

fn publish(temporary: &NamedTempFile, path: &Path, digest: &str) -> Result<(), HarnessError> {
    temporary.as_file().sync_all()?;
    match std::fs::hard_link(temporary.path(), path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            require_existing(path, digest)
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
    let mut command = Command::new("curl");
    command
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
    let budget = DownloadBudget { started: Instant::now(), max_bytes };
    let mut child = curl(url, output).spawn()?;
    let copied = copy_body(&mut input, file, &budget);
    drop(input);
    let finished = match &copied {
        Ok(_) => wait_status(&mut child, budget.started),
        Err(_) => Ok(()),
    };
    let reaped = terminate(&mut child);
    let actual = combine(combine(copied, finished), reaped)?;
    if actual != digest {
        return Err(HarnessError::invalid("pinned download digest mismatch"));
    }
    Ok(())
}

fn remaining(started: Instant) -> Result<Duration, HarnessError> {
    LIFETIME
        .checked_sub(started.elapsed())
        .filter(|remaining| !remaining.is_zero())
        .ok_or_else(|| HarnessError::invalid("fixture download deadline exceeded"))
}

fn copy_body(
    input: &mut UnixStream,
    file: &mut File,
    budget: &DownloadBudget,
) -> Result<String, HarnessError> {
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 16_384];
    let copied = (0..=budget.max_bytes).try_fold(0_u64, |total, _| {
        match copy_chunk(input, file, &mut hasher, &mut buffer, (total, budget)) {
            Ok(0) => ControlFlow::Break(Ok(())),
            Ok(count) => ControlFlow::Continue(total.saturating_add(count)),
            Err(error) => ControlFlow::Break(Err(error)),
        }
    });
    match copied {
        ControlFlow::Break(result) => result?,
        ControlFlow::Continue(_) => {
            return Err(HarnessError::invalid("fixture download size limit exceeded"));
        }
    }
    Ok(format!("{:x}", base16ct::HexDisplay(&hasher.finalize())))
}

fn copy_chunk(
    input: &mut UnixStream,
    file: &mut File,
    hasher: &mut Sha256,
    buffer: &mut [u8],
    state: (u64, &DownloadBudget),
) -> Result<u64, HarnessError> {
    input.set_read_timeout(Some(remaining(state.1.started)?))?;
    let available = state.1.max_bytes.saturating_sub(state.0).saturating_add(1);
    let count = input.take(available).read(buffer)?;
    let count_u64 = u64::try_from(count)
        .map_err(|_| HarnessError::invalid("fixture download byte count overflow"))?;
    if state.0.checked_add(count_u64).is_none_or(|total| total > state.1.max_bytes) {
        return Err(HarnessError::invalid("fixture download size limit exceeded"));
    }
    let bytes = buffer
        .get(..count)
        .ok_or_else(|| HarnessError::invalid("fixture download buffer length invalid"))?;
    file.write_all(bytes)?;
    hasher.update(bytes);
    Ok(count_u64)
}

fn wait_status(child: &mut Child, started: Instant) -> Result<(), HarnessError> {
    let result = (0..18_001).find_map(|_| match child.try_wait() {
        Ok(Some(status)) => Some(check_status(status)),
        Err(error) => Some(Err(error.into())),
        Ok(None) => match remaining(started) {
            Err(error) => Some(Err(error)),
            Ok(duration) => {
                std::thread::sleep(duration.min(Duration::from_millis(10)));
                None
            }
        },
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

fn terminate(child: &mut Child) -> Result<(), HarnessError> {
    let killed = child.kill();
    let waited = child.wait().map(|_| ()).map_err(HarnessError::from);
    match killed {
        Ok(()) => waited,
        Err(error) if error.kind() == std::io::ErrorKind::InvalidInput => waited,
        Err(error) => combine(Err(error.into()), waited),
    }
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
