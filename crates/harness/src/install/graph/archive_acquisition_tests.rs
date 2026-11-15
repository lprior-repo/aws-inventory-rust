use super::*;
use std::cell::RefCell;
use std::io::Seek;
use std::os::unix::fs::{FileExt, FileTypeExt, OpenOptionsExt};
use std::time::Duration;

struct Scenario {
    mutation: Option<(File, Vec<u8>, Vec<u8>)>,
    checkpoints: usize,
}

thread_local! {
    static SCENARIO: RefCell<Option<Scenario>> = const { RefCell::new(None) };
}

struct Reset;

impl Drop for Reset {
    fn drop(&mut self) {
        SCENARIO.with(|scenario| drop(scenario.borrow_mut().take()));
    }
}

fn configure(mutation: Option<(File, Vec<u8>, Vec<u8>)>) -> Reset {
    SCENARIO.with(|scenario| {
        *scenario.borrow_mut() = Some(Scenario { mutation, checkpoints: 0 });
    });
    Reset
}


pub(in super::super) fn checkpoint(snapshot: &File) -> Result<(), HarnessError> {
    SCENARIO.with(|scenario| {
        let mut borrowed = scenario.borrow_mut();
        let Some(scenario) = borrowed.as_mut() else { return Ok(()); };
        let Some((writer, first, second)) = scenario.mutation.as_mut() else { return Ok(()); };
        // The original inode is still independently writable after digest and after listing.
        let replacement = if scenario.checkpoints == 0 { first } else { second };
        writer.rewind()?;
        writer.write_all(replacement)?;
        writer.set_len(u64::try_from(replacement.len()).map_err(|error| HarnessError::invalid(error.to_string()))?)?;
        writer.sync_all()?;
        // Even a writer reopened through proc cannot change the snapshot before either tar read.
        let alias = std::fs::OpenOptions::new().write(true).open(crate::install::graph::descriptor_path(snapshot))?;
        assert_eq!(alias.write_at(b"changed", 0).err().and_then(|error| error.raw_os_error()), Some(libc::EPERM));
        assert_eq!(alias.set_len(0).err().and_then(|error| error.raw_os_error()), Some(libc::EPERM));
        scenario.checkpoints = scenario.checkpoints.saturating_add(1);
        Ok(())
    })
}

#[test]
fn given_cache_matching_symlink_after_cache_check_when_public_extract_runs_then_link_is_refused() -> TestResult {
    // Given a cache-validated fixture replaced by a symlink to the same bytes.
    let (given, source, _input) = fixture(&[member("redis-8.10.3/kept", b"verified")])?;
    let path = given.path().join("input.tar");
    let digest = crate::provenance::sha256_hex(&std::fs::read(&path)?);
    assert!(crate::install::graph::download::existing(&path, &digest)?);
    let target = given.path().join("target.tar");
    std::fs::rename(&path, &target)?;
    let before = std::fs::read(&target)?;
    std::os::unix::fs::symlink(&target, &path)?;
    // When the actual production extraction boundary opens the substituted cache path.
    let result = extract(&path, &source);
    // Then nofollow refuses acquisition, rather than accepting the digest-matching target.
    assert!(matches!(result, Err(HarnessError::Io(error)) if error.raw_os_error() == Some(libc::ELOOP)));
    assert_eq!(std::fs::read_link(&path)?, target);
    assert_eq!(std::fs::read(target)?, before);
    assert!(!source.try_exists()?);
    Ok(())
}

#[test]
fn given_fifo_after_cache_check_when_public_extract_runs_then_refusal_does_not_need_a_writer() -> TestResult {
    // Given an ordinary cache entry swapped for a no-writer FIFO after validation.
    let (given, source, _input) = fixture(&[member("redis-8.10.3/kept", b"verified")])?;
    let path = given.path().join("input.tar");
    let digest = crate::provenance::sha256_hex(&std::fs::read(&path)?);
    assert!(crate::install::graph::download::existing(&path, &digest)?);
    std::fs::remove_file(&path)?;
    assert!(Command::new("mkfifo").arg(&path).status()?.success());
    let opening = path.clone();
    let output = source.clone();
    let (sent, received) = std::sync::mpsc::channel();
    let worker = std::thread::spawn(move || {
        let result = extract(&opening, &output).map(|_| ());
        sent.send(result).map_err(std::io::Error::other)
    });
    // When production acquisition runs, it must finish while no FIFO writer exists.
    let result = received.recv_timeout(Duration::from_secs(5));
    // Rescue a regressed blocking open before joining, so this regression cannot hang the gate.
    let rescue = std::fs::OpenOptions::new().read(true).write(true)
        .custom_flags(libc::O_NONBLOCK).open(&path)?;
    worker.join().map_err(|_| "extraction thread failed")??;
    let error = result?.err().ok_or("FIFO accepted")?;
    assert_eq!(error.to_string(), "unexpected Redis archive input type or download byte bound");
    assert!(std::fs::symlink_metadata(&path)?.file_type().is_fifo());
    assert!(!source.try_exists()?);
    drop(rescue);
    Ok(())
}

#[test]
fn given_oversized_regular_cache_when_public_extract_runs_then_fstat_refuses_before_reading() -> TestResult {
    let given = tempfile::tempdir()?;
    let path = given.path().join("input.tar");
    let source = given.path().join("source");
    let length = MAX_DOWNLOAD.checked_add(1).ok_or("fixture length overflow")?;
    File::create_new(&path)?.set_len(length)?;
    let result = extract(&path, &source);
    assert!(matches!(result, Err(HarnessError::Invalid(error)) if error == "unexpected Redis archive input type or download byte bound"));
    assert_eq!(std::fs::metadata(&path)?.len(), length);
    assert!(!source.try_exists()?);
    Ok(())
}

#[test]
fn given_original_writer_mutations_when_shared_production_extract_runs_then_both_tar_passes_use_sealed_pinned_bytes() -> TestResult {
    // Given a real tar with an explicit fixture pin and an independent original writer.
    let (given, source, _input) = fixture(&[member("redis-8.10.3/kept", b"verified")])?;
    let path = given.path().join("input.tar");
    let digest = crate::provenance::sha256_hex(&std::fs::read(&path)?);
    let first = given.path().join("first.tar");
    let second = given.path().join("second.tar");
    archive(&first, &[member("redis-8.10.3/substituted", b"unverified")])?;
    archive(&second, &[member("../escaped", b"unreviewed")])?;
    let writer = std::fs::OpenOptions::new().write(true).open(&path)?;
    let _reset = configure(Some((writer, std::fs::read(first)?, std::fs::read(second)?)));
    // When the original is overwritten after pin verification, then again after listing.
    let owned = extract_pinned(&path, &source, &digest)?;
    // Then only the original verified member is extracted; writable aliases cannot alter seals.
    assert_eq!(std::fs::read(owned.anchor().join("kept"))?, b"verified");
    assert!(!owned.anchor().join("substituted").try_exists()?);
    assert!(!given.path().join("escaped").try_exists()?);
    SCENARIO.with(|scenario| assert_eq!(scenario.borrow().as_ref().map(|scenario| scenario.checkpoints), Some(2)));
    owned.remove_tree()?;
    Ok(())
}

#[cfg(feature = "native-graph-integration")]
fn genuine_archive() -> Result<(tempfile::TempDir, PathBuf, PathBuf), Box<dyn std::error::Error>> {
    let artifacts = std::env::var_os("AWS_INVENTORY_GRAPH_ARTIFACTS")
        .ok_or("native integration requires AWS_INVENTORY_GRAPH_ARTIFACTS")?;
    let given = tempfile::tempdir()?;
    let path = given.path().join("input.tar.gz");
    std::fs::copy(Path::new(&artifacts).join("redis-8.10.3.tar.gz"), &path)?;
    assert!(crate::install::graph::download::existing(&path, REDIS_SHA256)?);
    let source = given.path().join("source");
    Ok((given, path, source))
}

#[cfg(feature = "native-graph-integration")]
#[test]
fn given_genuine_pinned_symlink_after_cache_check_when_public_extract_runs_then_target_is_not_followed() -> TestResult {
    let (given, path, source) = genuine_archive()?;
    let target = given.path().join("target.tar.gz");
    std::fs::rename(&path, &target)?;
    std::os::unix::fs::symlink(&target, &path)?;
    let result = extract(&path, &source);
    assert!(matches!(result, Err(HarnessError::Io(error)) if error.raw_os_error() == Some(libc::ELOOP)));
    assert_eq!(std::fs::read_link(&path)?, target);
    assert!(crate::install::graph::download::existing(&target, REDIS_SHA256)?);
    assert!(!source.try_exists()?);
    Ok(())
}

#[cfg(feature = "native-graph-integration")]
#[test]
fn given_genuine_pin_and_original_writer_mutations_when_public_extract_runs_then_verified_snapshot_is_extracted() -> TestResult {
    // Given the genuine provisioned Redis artifact, checked against the production constant.
    let (given, path, source) = genuine_archive()?;
    let first = given.path().join("first.tar");
    let second = given.path().join("second.tar");
    archive(&first, &[member("redis-8.10.3/substituted", b"unverified")])?;
    archive(&second, &[member("../escaped", b"unreviewed")])?;
    let writer = std::fs::OpenOptions::new().write(true).open(&path)?;
    let _reset = configure(Some((writer, std::fs::read(first)?, std::fs::read(second)?)));
    // When mutations occur after genuine production-pin verification and after listing.
    let owned = extract(&path, &source)?;
    // Then real pinned source survives both passes, not either injected archive.
    assert!(!std::fs::read(owned.anchor().join("Makefile"))?.is_empty());
    assert!(!std::fs::read(owned.anchor().join("src/server.c"))?.is_empty());
    assert!(!owned.anchor().join("substituted").try_exists()?);
    assert!(!given.path().join("escaped").try_exists()?);
    SCENARIO.with(|scenario| assert_eq!(scenario.borrow().as_ref().map(|scenario| scenario.checkpoints), Some(2)));
    owned.remove_tree()?;
    Ok(())
}
