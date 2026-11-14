use std::fs::{File, OpenOptions};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use super::{HarnessError, MAX_DOCUMENT_BYTES, TestResult, read_json};

type ReadResult = Result<(serde_json::Value, String), HarnessError>;

fn create_pipe(path: &Path) -> io::Result<File> {
    if !Command::new("mkfifo").arg(path).status()?.success() {
        return Err(io::Error::other("mkfifo did not create the test pipe"));
    }

    OpenOptions::new().read(true).write(true).open(path)
}

fn write_until_released(path: PathBuf, length: u64, release: Receiver<()>) -> io::Result<()> {
    let mut pipe = OpenOptions::new().write(true).open(path)?;
    let written = io::copy(&mut io::repeat(b' ').take(length), &mut pipe)?;
    if written != length {
        return Err(io::Error::other(
            "FIFO producer stopped before the sentinel",
        ));
    }
    release
        .recv_timeout(Duration::from_secs(20))
        .map_err(io::Error::other)?;
    drop(pipe);
    Ok(())
}

fn join_worker(worker: JoinHandle<io::Result<()>>) -> TestResult {
    worker.join().map_err(|_| "FIFO worker panicked")??;
    Ok(())
}

fn collect_result(
    reader: JoinHandle<io::Result<()>>,
    writer: JoinHandle<io::Result<()>>,
    release: Sender<()>,
    results: Receiver<ReadResult>,
    bootstrap: File,
) -> Result<ReadResult, Box<dyn std::error::Error>> {
    let observed = results.recv_timeout(Duration::from_secs(10));
    let released = release.send(());
    drop(bootstrap);
    let written = join_worker(writer);
    let read = join_worker(reader);
    released?;
    written?;
    read?;
    Ok(observed?)
}

fn read_open_fifo(path: &Path, length: u64) -> Result<ReadResult, Box<dyn std::error::Error>> {
    let bootstrap = create_pipe(path)?;
    let (release, released) = mpsc::channel();
    let (result, results) = mpsc::channel();
    let writer_path = path.to_owned();
    let writer = thread::spawn(move || write_until_released(writer_path, length, released));
    let reader_path = path.to_owned();
    let reader = thread::spawn(move || {
        result
            .send(read_json(&reader_path))
            .map_err(|_| io::Error::other("FIFO result receiver disconnected"))
    });
    collect_result(reader, writer, release, results, bootstrap)
}

#[test]
fn given_open_fifo_over_public_limit_when_json_is_read_then_size_error_precedes_eof() -> TestResult
{
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("oversized.fifo");
    let length = u64::try_from(MAX_DOCUMENT_BYTES)?
        .checked_add(1)
        .ok_or("FIFO length overflow")?;
    let result = read_open_fifo(&path, length)?;
    assert!(matches!(result, Err(HarnessError::Invalid(reason))
        if reason == "JSON document exceeds its byte budget: oversized.fifo"));
    Ok(())
}
