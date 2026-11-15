use super::*;
use std::cell::RefCell;
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};

struct PublicationBoundary {
    observations: std::sync::mpsc::SyncSender<(bool, std::fs::Metadata)>,
    resume: std::sync::mpsc::Receiver<()>,
}

thread_local! {
    static BOUNDARY: RefCell<Option<PublicationBoundary>> = const { RefCell::new(None) };
}

pub(in super::super) fn publication_checkpoint(
    staged: &std::fs::File,
    before: bool,
) -> Result<(), crate::HarnessError> {
    BOUNDARY.with(|boundary| {
        let borrowed = boundary.borrow();
        let Some(boundary) = borrowed.as_ref() else { return Ok(()); };
        boundary.observations.send((before, staged.metadata()?))
            .map_err(|error| crate::HarnessError::invalid(error.to_string()))?;
        boundary.resume.recv_timeout(std::time::Duration::from_secs(5))
            .map_err(|error| crate::HarnessError::invalid(error.to_string()))
    })
}

fn replace_parent(parent: &Path, moved: &Path) -> std::io::Result<Vec<PathBuf>> {
    std::fs::rename(parent, moved)?;
    std::fs::create_dir(parent)?;
    let staged = entries(moved)?;
    staged.iter().try_for_each(|path| {
        let name = path.file_name().ok_or_else(|| std::io::Error::other("missing staging name"))?;
        std::fs::write(parent.join(name), b"foreign staging replacement")
    })?;
    std::fs::write(parent.join("foreign"), b"foreign parent")?;
    Ok(staged.into_iter().filter_map(|path| path.file_name().map(PathBuf::from)).collect())
}

fn parent_swap(download_matches: bool) -> TestResult {
    // Given a transfer whose response is held until the real staging parent is replaced.
    let directory = tempfile::tempdir()?;
    let parent = directory.path().join("cache");
    let moved = directory.path().join("moved");
    std::fs::create_dir(&parent)?;
    let output = parent.join("published");
    let replacing = parent.clone();
    let retained = moved.clone();
    let (sent, received) = std::sync::mpsc::channel();
    let (url, worker) = serve_once(move |stream| {
        let staged = replace_parent(&replacing, &retained)?;
        // Recreate every visible staging name: the old implementation linked/deleted these.
        sent.send(staged).map_err(std::io::Error::other)?;
        stream.write_all(b"HTTP/1.1 200 OK\r\nConnection: close\r\n\r\nwanted")
    })?;
    let expected = if download_matches { b"wanted".as_slice() } else { b"different".as_slice() };
    // When the public production download finishes or rejects the received digest.
    let result = pinned(&url, &output, &sha256_hex(expected));
    worker.join().map_err(|_| std::io::Error::other("server thread failed"))??;
    let staged = received.recv()?;
    // Then publication is FD-relative and error cleanup never removes foreign replacements.
    assert_eq!(result.is_ok(), download_matches);
    assert_eq!(std::fs::read(parent.join("foreign"))?, b"foreign parent");
    staged.iter().try_for_each(|name| -> TestResult {
        assert_eq!(std::fs::read(parent.join(name))?, b"foreign staging replacement");
        Ok(())
    })?;
    assert!(!output.try_exists()?);
    if download_matches {
        assert_eq!(std::fs::read(moved.join("published"))?, b"wanted");
        assert_eq!(entries(&moved)?, vec![moved.join("published")]);
    } else {
        assert!(entries(&moved)?.is_empty());
    }
    directory.close()?;
    Ok(())
}

#[test]
fn given_replaced_staging_parent_when_pinned_then_only_retained_parent_receives_verified_bytes() -> TestResult {
    parent_swap(true)
}

#[test]
fn given_replaced_staging_names_when_digest_fails_then_foreign_replacements_survive() -> TestResult {
    parent_swap(false)
}

#[test]
fn given_competing_fifo_when_pinned_then_download_refuses_without_open_block_or_unlink() -> TestResult {
    use std::os::unix::fs::FileTypeExt;
    let directory = tempfile::tempdir()?;
    let output = directory.path().join("published");
    let competing = output.clone();
    let (url, worker) = serve_once(move |stream| {
        let status = std::process::Command::new("mkfifo").arg(&competing).status()?;
        if !status.success() {
            return Err(std::io::Error::other("FIFO creation failed"));
        }
        stream.write_all(b"HTTP/1.1 200 OK\r\nConnection: close\r\n\r\nwanted")
    })?;
    let downloading = output.clone();
    let (sent, received) = std::sync::mpsc::channel();
    let downloader = std::thread::spawn(move || {
        sent.send(pinned(&url, &downloading, &sha256_hex(b"wanted")))
            .map_err(std::io::Error::other)
    });
    worker.join().map_err(|_| std::io::Error::other("server thread failed"))??;
    let result = received.recv_timeout(std::time::Duration::from_secs(5));
    // Release a regressed blocking FIFO open before joining the download region.
    let rescue = std::fs::OpenOptions::new().read(true).write(true)
        .custom_flags(libc::O_NONBLOCK).open(&output)?;
    downloader.join().map_err(|_| "download thread failed")??;
    assert!(result?.is_err());
    drop(rescue);
    assert!(std::fs::symlink_metadata(&output)?.file_type().is_fifo());
    assert_eq!(entries(directory.path())?, vec![output]);
    directory.close()?;
    Ok(())
}

#[test]
fn given_replaced_visible_staging_when_pinned_then_replacement_never_becomes_publication_or_cleanup() -> TestResult {
    // Given the server can replace every visible staging name before releasing verified bytes.
    let directory = tempfile::tempdir()?;
    let output = directory.path().join("published");
    let parent = directory.path().to_owned();
    let (sent, received) = std::sync::mpsc::channel();
    let (url, worker) = serve_once(move |stream| {
        let staged = entries(&parent)?;
        staged.iter().try_for_each(|path| -> std::io::Result<()> {
            std::fs::remove_file(path)?;
            std::fs::write(path, b"foreign replacement")
        })?;
        sent.send(staged).map_err(std::io::Error::other)?;
        stream.write_all(b"HTTP/1.1 200 OK\r\nConnection: close\r\n\r\nwanted")
    })?;
    // When the public download verifies and publishes its owned staging descriptor.
    let result = pinned(&url, &output, &sha256_hex(b"wanted"));
    worker.join().map_err(|_| std::io::Error::other("server thread failed"))??;
    result?;
    // Then foreign replacements survive and cannot substitute their bytes in the cache.
    assert_eq!(std::fs::read(&output)?, b"wanted");
    received.recv()?.iter().try_for_each(|path| -> TestResult {
        assert_eq!(std::fs::read(path)?, b"foreign replacement");
        Ok(())
    })?;
    directory.close()?;
    Ok(())
}

#[test]
fn given_verified_staging_when_public_download_commits_then_first_visible_inode_is_complete_and_identical() -> TestResult {
    // Given actual downloaded bytes and a bounded observer at the production commit boundary.
    let directory = tempfile::tempdir()?;
    let source = directory.path().join("source");
    let output = directory.path().join("published");
    let bytes = vec![0x5a_u8; 65_536];
    std::fs::write(&source, &bytes)?;
    let digest = sha256_hex(&bytes);
    let url = file_url(&source)?;
    let destination = output.clone();
    let expected = digest.clone();
    let (observations, observed) = std::sync::mpsc::sync_channel(1);
    let (resume, gate) = std::sync::mpsc::sync_channel(0);
    std::thread::scope(|scope| -> TestResult {
        let publisher = scope.spawn(move || {
            BOUNDARY.with(|boundary| {
                *boundary.borrow_mut() = Some(PublicationBoundary { observations, resume: gate });
            });
            let result = pinned(&url, &destination, &expected);
            BOUNDARY.with(|boundary| drop(boundary.borrow_mut().take()));
            result
        });
        let observation = observe_commit(&output, &digest, observed, resume);
        let published = publisher.join().map_err(|_| "publisher thread failed")?;
        observation?;
        published?;
        Ok(())
    })?;
    assert_eq!(std::fs::read(&output)?, bytes);
    directory.close()?;
    Ok(())
}

fn observe_commit(
    output: &Path,
    digest: &str,
    observed: std::sync::mpsc::Receiver<(bool, std::fs::Metadata)>,
    resume: std::sync::mpsc::SyncSender<()>,
) -> TestResult {
    let deadline = std::time::Duration::from_secs(5);
    let (before, staged) = observed.recv_timeout(deadline)?;
    assert!(before);
    assert_eq!(staged.nlink(), 0);
    assert!(!output.try_exists()?);
    resume.send(())?;
    // The publisher is stopped immediately after the publication syscall, before returning.
    let (before, committed) = observed.recv_timeout(deadline)?;
    assert!(!before);
    let visible = std::fs::metadata(output)?;
    assert_eq!((visible.dev(), visible.ino()), (staged.dev(), staged.ino()));
    assert_eq!((committed.dev(), committed.ino()), (staged.dev(), staged.ino()));
    assert_eq!(committed.nlink(), 1);
    assert!(existing(output, digest)?);
    // Exact inode identity rules out a create-new + copy implementation and partial visibility.
    resume.send(())?;
    Ok(())
}
