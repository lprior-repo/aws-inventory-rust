use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::Path;

use serde_json::{Value, json};

use super::{
    FALKOR_VERSION, Layout, MODULE_SHA256, MODULE_URL, OwnedDirectory, REDIS_SHA256, REDIS_URL,
    REDIS_VERSION, build, complete, install, receipt,
};
use crate::HarnessError;

pub(super) type TestResult = Result<(), Box<dyn std::error::Error>>;

pub(super) const SERVER_VERSION: &str =
    "Redis server v=8.10.3 sha=6018c5a5:0 malloc=libc bits=64 build=fd14ddffccc4aa26";
pub(super) const CLIENT_VERSION: &str = "redis-cli 8.10.3 (git:6018c5a5)";

pub(super) fn fixture() -> Result<(tempfile::TempDir, Layout), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let layout = Layout::new(&fs::canonicalize(directory.path())?);
    fs::create_dir_all(layout.source.join("src"))?;
    fs::write(&layout.server, b"abc")?;
    fs::write(&layout.client, b"client")?;
    super::set_executable(&layout.server)?;
    super::set_executable(&layout.client)?;
    fs::write(&layout.archive, b"archive")?;
    fs::write(&layout.module, b"module")?;
    Ok((directory, layout))
}

pub(super) fn value(layout: &Layout) -> Result<Value, HarnessError> {
    Ok(json!({
        "schema_version": 2,
        "redis_version": REDIS_VERSION,
        "falkordb_version": FALKOR_VERSION,
        "redis_url": REDIS_URL,
        "redis_archive_sha256": REDIS_SHA256,
        "module_url": MODULE_URL,
        "module_sha256": MODULE_SHA256,
        "versions": {"server": SERVER_VERSION, "client": CLIENT_VERSION},
        "redis_server": layout.server,
        "redis_cli": layout.client,
        "falkor_module": layout.module,
        "redis_server_sha256": receipt::digest(layout, &layout.server)?,
        "redis_cli_sha256": receipt::digest(layout, &layout.client)?,
    }))
}

fn replace(value: &mut Value, key: &str, replacement: Value) -> TestResult {
    value
        .as_object_mut()
        .ok_or("receipt is not an object")?
        .insert(key.to_owned(), replacement);
    Ok(())
}

pub(super) fn executable(path: &Path, body: &str) -> TestResult {
    fs::write(path, format!("#!/bin/sh\n{body}\n"))?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o755))?;
    Ok(())
}

#[test]
fn given_unreceipted_binaries_when_installing_then_they_never_run() -> TestResult {
    let (_directory, layout) = fixture()?;
    let marker = layout.root.join("executed");
    let body = format!("touch '{}'", marker.display());
    executable(&layout.server, &body)?;
    executable(&layout.client, &body)?;
    let error = install(&layout.root, 1)
        .err()
        .ok_or("installation was accepted")?;
    assert!(error.to_string().contains("unreceipted"));
    assert!(!marker.exists());
    assert!(layout.server.is_file());
    assert!(!layout.lock.exists());
    Ok(())
}

#[test]
fn given_foreign_receipt_pins_when_installing_then_cache_is_rejected_without_execution()
-> TestResult {
    let (_directory, layout) = fixture()?;
    let marker = layout.root.join("executed");
    executable(&layout.server, &format!("touch '{}'", marker.display()))?;
    let mut receipt = value(&layout)?;
    replace(&mut receipt, "redis_version", json!("8.10.30"))?;
    fs::write(&layout.receipt, serde_json::to_vec(&receipt)?)?;
    let error = install(&layout.root, 1)
        .err()
        .ok_or("foreign pin was accepted")?;
    assert!(
        error
            .to_string()
            .contains("receipt pin mismatch: redis_version")
    );
    assert!(!marker.exists());
    assert!(layout.source.is_dir());
    Ok(())
}

#[test]
fn given_receipt_pins_when_any_pin_changes_then_verification_fails() -> TestResult {
    let (_directory, layout) = fixture()?;
    let original = value(&layout)?;
    receipt::verify_pins(&original)?;
    ["redis_version", "falkordb_version", "redis_url", "module_url", "redis_archive_sha256", "module_sha256"]
        .into_iter()
        .try_for_each(|key| -> TestResult {
            let mut changed = original.clone();
            replace(&mut changed, key, json!("foreign"))?;
            assert!(receipt::verify_pins(&changed).is_err());
            Ok(())
        })?;
    Ok(())
}

#[test]
fn given_receipted_binary_when_bytes_change_then_digest_verification_fails() -> TestResult {
    let (_directory, layout) = fixture()?;
    let receipt = value(&layout)?;
    assert_eq!(
        receipt::digest(&layout, &layout.server)?,
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    receipt::verify_binary(&layout, &receipt, &layout.server, "redis_server_sha256")?;
    fs::write(&layout.server, b"modified")?;
    assert!(
        receipt::verify_binary(&layout, &receipt, &layout.server, "redis_server_sha256").is_err()
    );
    Ok(())
}

#[test]
fn given_receipt_paths_when_a_path_escapes_then_verification_fails() -> TestResult {
    let (_directory, layout) = fixture()?;
    let mut receipt = value(&layout)?;
    receipt::verify_paths(&layout, &receipt)?;
    replace(&mut receipt, "redis_server", json!("/bin/sh"))?;
    assert!(receipt::verify_paths(&layout, &receipt).is_err());
    assert!(receipt::regular(&layout, Path::new("/bin/sh")).is_err());
    Ok(())
}

#[test]
fn given_symlinked_binary_or_ancestor_when_verifying_then_both_are_rejected() -> TestResult {
    let (_directory, layout) = fixture()?;
    fs::remove_file(&layout.server)?;
    symlink(&layout.client, &layout.server)?;
    assert!(receipt::regular(&layout, &layout.server).is_err());
    fs::remove_file(&layout.server)?;
    let moved = layout.root.join("elsewhere");
    fs::rename(layout.source.join("src"), &moved)?;
    symlink(&moved, layout.source.join("src"))?;
    assert!(receipt::regular(&layout, &layout.client).is_err());
    Ok(())
}

#[test]
fn given_exclusive_lock_when_installing_then_existing_tree_is_untouched() -> TestResult {
    let directory = tempfile::tempdir()?;
    let layout = Layout::new(directory.path());
    fs::create_dir(&layout.lock)?;
    fs::write(layout.lock.join("owner"), b"concurrent")?;
    assert!(install(directory.path(), 1).is_err());
    assert_eq!(fs::read(layout.lock.join("owner"))?, b"concurrent");
    assert!(!layout.archive.exists());
    Ok(())
}


#[test]
fn given_operation_and_cleanup_failures_when_returning_then_both_are_reported() -> TestResult {
    let result = complete::<()>(
        Err(HarnessError::invalid("build failed")),
        Err(HarnessError::invalid("permission denied")),
    );
    let error = result.err().ok_or("errors were swallowed")?;
    assert_eq!(
        error.to_string(),
        "build failed; cleanup failed: permission denied"
    );
    Ok(())
}

#[test]
fn given_complete_receipt_when_publishing_then_only_final_complete_bytes_remain() -> TestResult {
    let (_directory, layout) = fixture()?;
    let lock = OwnedDirectory::claim(&layout.lock)?;
    let receipt = value(&layout)?;
    receipt::store(&OwnedDirectory::capture(&layout.source)?, &receipt)?;
    assert_eq!(serde_json::from_slice::<Value>(&fs::read(&layout.receipt)?)?, receipt);
    assert!(!layout.lock.join("installation.json").exists());
    lock.remove_empty()?;
    Ok(())
}

#[test]
fn given_preexisting_receipt_when_publishing_then_original_is_preserved() -> TestResult {
    let (_directory, layout) = fixture()?;
    let lock = OwnedDirectory::claim(&layout.lock)?;
    fs::write(&layout.receipt, b"original")?;
    assert!(receipt::store(&OwnedDirectory::capture(&layout.source)?, &value(&layout)?).is_err());
    assert_eq!(fs::read(&layout.receipt)?, b"original");
    assert!(!layout.lock.join("installation.json").exists());
    lock.remove_empty()?;
    Ok(())
}

#[test]
fn given_foreign_staging_file_when_publishing_then_it_is_not_removed() -> TestResult {
    let (_directory, layout) = fixture()?;
    let lock = OwnedDirectory::claim(&layout.lock)?;
    let temporary = layout.lock.join("installation.json");
    fs::write(&temporary, b"other owner")?;
    receipt::store(&OwnedDirectory::capture(&layout.source)?, &value(&layout)?)?;
    assert_eq!(fs::read(&temporary)?, b"other owner");
    assert!(layout.receipt.exists());
    fs::remove_file(temporary)?;
    lock.remove_empty()?;
    Ok(())
}


#[test]
fn given_stalled_version_process_when_querying_then_timeout_reports_failure_and_cleans_output()
-> TestResult {
    let (_directory, layout) = fixture()?;
    let lock = OwnedDirectory::claim(&layout.lock)?;
    let source = OwnedDirectory::capture(&layout.source)?;
    executable(&layout.server, "exec sleep 60")?;
    let result = build::versions(&source)
        .map(|_| Value::Null)
        .map_err(receipt::PublicationError::Uncommitted);
    let error = super::finish_installation(source, result).err().ok_or("stalled version succeeded")?;
    assert!(error.to_string().contains("version command failed or timed out"));
    assert!(!layout.lock.join("server.version").exists());
    assert!(!layout.source.exists());
    lock.remove_empty()?;
    Ok(())
}

#[test]
fn given_missing_binary_digest_when_verifying_then_receipt_is_rejected() -> TestResult {
    let (_directory, layout) = fixture()?;
    let mut receipt = value(&layout)?;
    let removed = receipt
        .as_object_mut()
        .ok_or("not an object")?
        .remove("redis_server_sha256");
    assert!(removed.is_some());
    assert!(
        receipt::verify_binary(&layout, &receipt, &layout.server, "redis_server_sha256").is_err()
    );
    Ok(())
}
