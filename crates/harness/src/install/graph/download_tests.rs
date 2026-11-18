#[path = "download_race_tests.rs"]
pub(super) mod races;

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::thread::JoinHandle;

use super::{MAX_DOWNLOAD, existing, pinned};
use crate::provenance::sha256_hex;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn file_url(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let absolute = path.canonicalize()?;
    Ok(format!("file://{}", absolute.display()))
}

fn entries(path: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
    std::fs::read_dir(path)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect()
}

#[test]
fn given_verified_source_when_pinned_then_exact_bytes_are_published_and_reused() -> TestResult {
    let directory = tempfile::tempdir()?;
    let source = directory.path().join("source");
    let output = directory.path().join("published");
    let bytes = b"verified graph fixture\n";
    std::fs::write(&source, bytes)?;
    let digest = sha256_hex(bytes);
    pinned(&file_url(&source)?, &output, &digest)?;
    std::fs::remove_file(&source)?;
    pinned("file:///missing-source", &output, &digest)?;
    assert_eq!(std::fs::read(&output)?, bytes);
    assert!(existing(&output, &digest)?);
    assert_eq!(entries(directory.path())?, vec![output]);
    directory.close()?;
    Ok(())
}

#[test]
fn given_foreign_destination_when_pinned_then_original_bytes_are_not_clobbered() -> TestResult {
    let directory = tempfile::tempdir()?;
    let output = directory.path().join("published");
    std::fs::write(&output, b"foreign")?;
    let result = pinned("file:///missing-source", &output, &sha256_hex(b"wanted"));
    assert!(result.is_err());
    assert_eq!(std::fs::read(&output)?, b"foreign");
    assert_eq!(entries(directory.path())?, vec![output]);
    directory.close()?;
    Ok(())
}

#[test]
fn given_foreign_source_digest_when_pinned_then_nothing_is_published() -> TestResult {
    let directory = tempfile::tempdir()?;
    let source = directory.path().join("source");
    let output = directory.path().join("published");
    std::fs::write(&source, b"foreign source")?;
    let result = pinned(&file_url(&source)?, &output, &sha256_hex(b"wanted"));
    assert!(
        matches!(result, Err(crate::HarnessError::Invalid(error)) if error == "pinned download digest mismatch")
    );
    assert!(!output.try_exists()?);
    assert_eq!(entries(directory.path())?, vec![source]);
    directory.close()?;
    Ok(())
}

#[test]
fn given_predictable_temp_symlink_when_pinned_then_victim_and_link_are_untouched() -> TestResult {
    let directory = tempfile::tempdir()?;
    let source = directory.path().join("source");
    let victim = directory.path().join("victim");
    let output = directory.path().join("published");
    let old_temp = output.with_file_name(format!("published.downloading-{}", std::process::id()));
    std::fs::write(&source, b"wanted")?;
    std::fs::write(&victim, b"do not truncate")?;
    std::os::unix::fs::symlink(&victim, &old_temp)?;
    pinned(&file_url(&source)?, &output, &sha256_hex(b"wanted"))?;
    assert_eq!(std::fs::read(&output)?, b"wanted");
    assert_eq!(std::fs::read(&victim)?, b"do not truncate");
    assert_eq!(std::fs::read_link(&old_temp)?, victim);
    assert_eq!(entries(directory.path())?.len(), 4);
    directory.close()?;
    Ok(())
}

fn read_request(stream: &mut TcpStream) -> std::io::Result<()> {
    stream.set_read_timeout(Some(std::time::Duration::from_secs(5)))?;
    let mut tail = [0_u8; 4];
    let result = stream.take(16_384).bytes().find_map(|byte| match byte {
        Err(error) => Some(Err(error)),
        Ok(byte) => {
            tail.rotate_left(1);
            let Some(last) = tail.last_mut() else {
                return Some(Err(std::io::Error::other("missing header buffer")));
            };
            *last = byte;
            (tail == *b"\r\n\r\n").then_some(Ok(()))
        }
    });
    result.ok_or_else(|| std::io::Error::other("request header exceeded limit"))?
}

fn accept(listener: &TcpListener) -> std::io::Result<TcpStream> {
    listener.set_nonblocking(true)?;
    let mut descriptors = [rustix::event::PollFd::new(
        listener,
        rustix::event::PollFlags::IN,
    )];
    let deadline = rustix::event::Timespec {
        tv_sec: 5,
        tv_nsec: 0,
    };
    let ready =
        rustix::event::poll(&mut descriptors, Some(&deadline)).map_err(std::io::Error::from)?;
    if ready != 1
        || !descriptors
            .first()
            .is_some_and(|descriptor| descriptor.revents().contains(rustix::event::PollFlags::IN))
    {
        return Err(std::io::Error::other(
            "fixture connection deadline exceeded",
        ));
    }
    listener.accept().map(|(stream, _)| stream)
}

fn serve_once<F>(action: F) -> Result<(String, JoinHandle<std::io::Result<()>>), std::io::Error>
where
    F: FnOnce(&mut TcpStream) -> std::io::Result<()> + Send + 'static,
{
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let url = format!("http://{}/fixture", listener.local_addr()?);
    let worker = std::thread::spawn(move || {
        let mut stream = accept(&listener)?;
        read_request(&mut stream)?;
        stream.set_write_timeout(Some(std::time::Duration::from_secs(5)))?;
        action(&mut stream)
    });
    Ok((url, worker))
}

fn oversized_server() -> Result<(String, JoinHandle<std::io::Result<()>>), std::io::Error> {
    serve_once(|stream| {
        stream.write_all(b"HTTP/1.1 200 OK\r\nConnection: close\r\n\r\n")?;
        stream.write_all(&[0_u8; 33])
    })
}

#[test]
fn given_no_content_length_when_actual_bytes_exceed_limit_then_download_is_rejected() -> TestResult
{
    let directory = tempfile::tempdir()?;
    let output = directory.path().join("published");
    let (url, worker) = oversized_server()?;
    let result = super::pinned_with_limit(&url, &output, &sha256_hex(b"not the streamed body"), 32);
    let served = worker
        .join()
        .map_err(|_| std::io::Error::other("server thread failed"))?;
    served?;
    let error = result
        .err()
        .ok_or_else(|| std::io::Error::other("oversized body accepted"))?;
    assert_eq!(error.to_string(), "fixture download size limit exceeded");
    assert!(!output.try_exists()?);
    assert!(entries(directory.path())?.is_empty());
    directory.close()?;
    Ok(())
}

#[test]
fn given_no_content_length_when_body_equals_limit_then_exact_bytes_are_published() -> TestResult {
    let directory = tempfile::tempdir()?;
    let output = directory.path().join("published");
    let bytes = b"01234567890123456789012345678901";
    let (url, worker) = serve_once(move |stream| {
        stream.write_all(b"HTTP/1.1 200 OK\r\nConnection: close\r\n\r\n")?;
        stream.write_all(bytes)
    })?;
    let result = super::pinned_with_limit(&url, &output, &sha256_hex(bytes), 32);
    worker
        .join()
        .map_err(|_| std::io::Error::other("server thread failed"))??;
    result?;
    assert_eq!(std::fs::read(&output)?, bytes);
    assert_eq!(entries(directory.path())?, vec![output]);
    directory.close()?;
    Ok(())
}

#[test]
fn given_oversized_existing_file_when_pinned_then_size_is_rejected_without_changes() -> TestResult {
    let directory = tempfile::tempdir()?;
    let output = directory.path().join("published");
    let length = MAX_DOWNLOAD
        .checked_add(1)
        .ok_or_else(|| std::io::Error::other("overflow"))?;
    std::fs::File::create_new(&output)?.set_len(length)?;
    let result = pinned("file:///missing-source", &output, "00");
    assert!(result.is_err());
    assert_eq!(std::fs::metadata(&output)?.len(), length);
    assert_eq!(entries(directory.path())?, vec![output]);
    directory.close()?;
    Ok(())
}

#[test]
fn given_competing_foreign_publication_when_download_finishes_then_existing_bytes_remain()
-> TestResult {
    let directory = tempfile::tempdir()?;
    let output = directory.path().join("published");
    let competing = output.clone();
    let (url, worker) = serve_once(move |stream| {
        std::fs::write(competing, b"foreign")?;
        stream.write_all(b"HTTP/1.1 200 OK\r\nConnection: close\r\n\r\nwanted")
    })?;
    let result = pinned(&url, &output, &sha256_hex(b"wanted"));
    worker
        .join()
        .map_err(|_| std::io::Error::other("server thread failed"))??;
    assert!(result.is_err());
    assert_eq!(std::fs::read(&output)?, b"foreign");
    assert_eq!(entries(directory.path())?, vec![output]);
    directory.close()?;
    Ok(())
}

#[test]
fn given_matching_competing_publication_when_download_finishes_then_existing_inode_is_kept()
-> TestResult {
    use std::os::unix::fs::MetadataExt;
    let directory = tempfile::tempdir()?;
    let output = directory.path().join("published");
    let competing = output.clone();
    let witness = directory.path().join("witness");
    let link = witness.clone();
    let (url, worker) = serve_once(move |stream| {
        std::fs::write(&competing, b"wanted")?;
        std::fs::hard_link(competing, link)?;
        stream.write_all(b"HTTP/1.1 200 OK\r\nConnection: close\r\n\r\nwanted")
    })?;
    let result = pinned(&url, &output, &sha256_hex(b"wanted"));
    worker
        .join()
        .map_err(|_| std::io::Error::other("server thread failed"))??;
    result?;
    assert_eq!(std::fs::read(&output)?, b"wanted");
    assert_eq!(
        std::fs::metadata(output)?.ino(),
        std::fs::metadata(witness)?.ino()
    );
    assert_eq!(entries(directory.path())?.len(), 2);
    directory.close()?;
    Ok(())
}

#[test]
fn given_vanished_competitor_when_publication_is_confirmed_then_success_is_not_reported()
-> TestResult {
    let directory = tempfile::tempdir()?;
    let output = directory.path().join("vanished");
    let result = super::require_existing(&output, &sha256_hex(b"wanted"));
    let error = result
        .err()
        .ok_or_else(|| std::io::Error::other("vanished file accepted"))?;
    assert_eq!(error.to_string(), "competing pinned download vanished");
    assert!(!output.try_exists()?);
    directory.close()?;
    Ok(())
}

#[test]
fn given_destination_symlink_when_pinned_then_target_and_symlink_remain_unchanged() -> TestResult {
    let directory = tempfile::tempdir()?;
    let output = directory.path().join("published");
    let victim = directory.path().join("victim");
    std::fs::write(&victim, b"wanted")?;
    std::os::unix::fs::symlink(&victim, &output)?;
    let result = pinned("file:///missing-source", &output, &sha256_hex(b"wanted"));
    assert!(result.is_err());
    assert_eq!(std::fs::read(&victim)?, b"wanted");
    assert_eq!(std::fs::read_link(&output)?, victim);
    assert_eq!(entries(directory.path())?.len(), 2);
    directory.close()?;
    Ok(())
}

#[test]
fn given_open_cache_when_path_is_replaced_and_inode_grows_then_reads_remain_owned_and_bounded()
-> TestResult {
    use std::io::Seek;
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("cached");
    let victim = directory.path().join("victim");
    std::fs::write(&victim, b"foreign")?;
    let mut writer = std::fs::File::create_new(&path)?;
    writer.write_all(b"wanted")?;
    let mut reader =
        super::open_cached(&path)?.ok_or_else(|| std::io::Error::other("cache missing"))?;
    std::fs::remove_file(&path)?;
    std::os::unix::fs::symlink(&victim, &path)?;
    assert_eq!(
        super::copy_body(&mut reader, &mut std::io::sink(), 6)?,
        sha256_hex(b"wanted")
    );
    writer.write_all(b"grew")?;
    reader.rewind()?;
    let result = super::copy_body(&mut reader, &mut std::io::sink(), 6);
    let error = result
        .err()
        .ok_or_else(|| std::io::Error::other("cache growth accepted"))?;
    assert_eq!(error.to_string(), "fixture download size limit exceeded");
    assert_eq!(std::fs::read(&victim)?, b"foreign");
    assert_eq!(std::fs::read_link(&path)?, victim);
    directory.close()?;
    Ok(())
}
