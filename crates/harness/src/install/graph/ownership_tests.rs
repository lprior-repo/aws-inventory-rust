use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::Path;

use super::super::installer_tests::{
    CLIENT_VERSION, SERVER_VERSION, TestResult, executable, fixture, value,
};
use super::super::{build, finish_installation, receipt};
use super::{OwnedDirectory, Removal};
use crate::HarnessError;

#[test]
fn given_swap_after_identity_check_when_claiming_then_foreign_empty_root_is_restored() -> TestResult
{
    let (_directory, layout) = fixture()?;
    let owned = OwnedDirectory::capture(&layout.source)?;
    owned.verify()?;
    let quarantine = OwnedDirectory::private(owned.parent.try_clone()?)?;
    let saved = layout.root.join("saved");
    fs::rename(&layout.source, &saved)?;
    fs::create_dir(&layout.source)?;
    let foreign = fs::symlink_metadata(&layout.source)?;
    assert!(owned.claim_and_remove(&quarantine, Removal::Tree).is_err());
    let restored = fs::symlink_metadata(&layout.source)?;
    assert_eq!(
        (foreign.dev(), foreign.ino()),
        (restored.dev(), restored.ino())
    );
    assert_eq!(fs::read(saved.join("src/redis-server"))?, b"abc");
    quarantine.remove_private_empty()?;
    Ok(())
}

#[test]
fn given_restore_collision_when_restoring_then_foreign_destination_is_not_overwritten() -> TestResult
{
    let (_directory, layout) = fixture()?;
    let owned = OwnedDirectory::capture(&layout.source)?;
    let quarantine = OwnedDirectory::private(owned.parent.try_clone()?)?;
    fs::rename(&layout.source, quarantine.anchor().join("owned"))?;
    fs::create_dir(&layout.source)?;
    fs::write(layout.source.join("foreign"), b"foreign")?;
    assert!(owned.restore(&quarantine).is_err());
    assert_eq!(fs::read(layout.source.join("foreign"))?, b"foreign");
    assert_eq!(
        fs::read(quarantine.anchor().join("owned/src/redis-server"))?,
        b"abc"
    );
    Ok(())
}

#[test]
fn given_bad_download_when_publishing_then_owned_source_is_rolled_back() -> TestResult {
    let (_directory, layout) = fixture()?;
    let source = OwnedDirectory::capture(&layout.source)?;
    let result = receipt::publish(&layout, &source);
    assert!(finish_installation(source, result).is_err());
    assert!(!layout.source.exists());
    assert!(!layout.receipt.exists());
    Ok(())
}

#[test]
fn given_existing_receipt_when_publishing_then_owned_source_is_rolled_back() -> TestResult {
    let (_directory, layout) = fixture()?;
    let source = OwnedDirectory::capture(&layout.source)?;
    fs::write(&layout.receipt, b"foreign")?;
    let result = receipt::store(&source, &value(&layout)?).map(|()| serde_json::Value::Null);
    assert!(finish_installation(source, result).is_err());
    assert!(!layout.source.exists());
    assert_eq!(fs::read(&layout.receipt)?, b"foreign");
    Ok(())
}

#[test]
fn given_committed_receipt_when_durability_fails_then_source_is_preserved() -> TestResult {
    let (_directory, layout) = fixture()?;
    let source = path_only_parent(OwnedDirectory::capture(&layout.source)?, &layout.root)?;
    let expected = value(&layout)?;
    let result = receipt::store(&source, &expected).map(|()| serde_json::Value::Null);
    assert!(matches!(
        &result,
        Err(receipt::PublicationError::Committed(_))
    ));
    assert!(finish_installation(source, result).is_err());
    assert_eq!(fs::read(&layout.server)?, b"abc");
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&fs::read(&layout.receipt)?)?,
        expected
    );
    Ok(())
}

fn path_only_parent(source: OwnedDirectory, path: &Path) -> Result<OwnedDirectory, HarnessError> {
    let flags =
        rustix::fs::OFlags::PATH | rustix::fs::OFlags::DIRECTORY | rustix::fs::OFlags::CLOEXEC;
    let parent =
        rustix::fs::open(path, flags, rustix::fs::Mode::empty()).map_err(std::io::Error::from)?;
    Ok(OwnedDirectory { parent, ..source })
}

#[test]
fn given_replaced_source_when_compiling_then_only_retained_tree_is_built() -> TestResult {
    let (_directory, layout) = fixture()?;
    fs::write(
        layout.source.join("Makefile"),
        "redis-server redis-cli:\n\tprintf owned > built\n",
    )?;
    let source = OwnedDirectory::capture(&layout.source)?;
    let saved = layout.root.join("saved");
    fs::rename(&layout.source, &saved)?;
    fs::create_dir(&layout.source)?;
    fs::write(
        layout.source.join("Makefile"),
        "redis-server redis-cli:\n\tprintf foreign > built\n",
    )?;
    build::compile(&source, 1)?;
    assert_eq!(fs::read(saved.join("built"))?, b"owned");
    assert!(!layout.source.join("built").exists());
    Ok(())
}

#[test]
fn given_flooding_version_when_querying_then_output_memory_and_time_are_bounded() -> TestResult {
    let (_directory, layout) = fixture()?;
    executable(
        &layout.server,
        "while :; do printf '012345678901234567890123456789'; done",
    )?;
    let source = OwnedDirectory::capture(&layout.source)?;
    let error = build::versions(&source).err().ok_or("flood accepted")?;
    assert!(error.to_string().contains("output exceeds byte limit"));
    assert!(!layout.lock.exists());
    Ok(())
}

#[test]
fn given_replaced_named_staging_when_linking_then_only_opened_anonymous_inode_is_published()
-> TestResult {
    let (_directory, layout) = fixture()?;
    let source = OwnedDirectory::capture(&layout.source)?;
    let lock = OwnedDirectory::claim(&layout.lock)?;
    let mut staged = receipt::stage_file(&source)?;
    let saved = layout.root.join("saved-lock");
    fs::rename(&layout.lock, &saved)?;
    fs::create_dir(&layout.lock)?;
    let foreign = layout.lock.join("installation.json");
    fs::write(&foreign, b"foreign")?;
    let expected = value(&layout)?;
    receipt::write_receipt(&source, &mut staged, &expected)?;
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&fs::read(&layout.receipt)?)?,
        expected
    );
    assert_eq!(fs::read(&foreign)?, b"foreign");
    assert!(lock.remove_empty().is_err());
    Ok(())
}
#[test]
fn given_replaced_source_when_cleanup_runs_then_concurrent_tree_survives() -> TestResult {
    let directory = tempfile::tempdir()?;
    let source = directory.path().join("source");
    let owned = OwnedDirectory::claim(&source)?;
    fs::write(source.join("original"), b"original")?;
    let saved = directory.path().join("saved");
    fs::rename(&source, &saved)?;
    fs::create_dir(&source)?;
    fs::write(source.join("concurrent"), b"concurrent")?;
    assert!(owned.remove_tree().is_err());
    assert_eq!(fs::read(source.join("concurrent"))?, b"concurrent");
    assert!(!saved.join("original").exists());
    Ok(())
}
#[test]
fn given_native_git_suffix_when_querying_versions_then_full_output_is_preserved() -> TestResult {
    let (_directory, layout) = fixture()?;
    let lock = OwnedDirectory::claim(&layout.lock)?;
    let source = OwnedDirectory::capture(&layout.source)?;
    executable(
        &layout.server,
        &format!("printf '%s\\n' '{SERVER_VERSION}'"),
    )?;
    executable(
        &layout.client,
        &format!("printf '%s\\n' '{CLIENT_VERSION}'"),
    )?;
    assert_eq!(
        build::versions(&source)?,
        (SERVER_VERSION.to_owned(), CLIENT_VERSION.to_owned())
    );
    lock.remove_empty()?;
    Ok(())
}
#[test]
fn given_chmod_without_byte_change_when_verifying_then_cache_is_rejected() -> TestResult {
    let (_directory, layout) = fixture()?;
    let receipt = value(&layout)?;
    let before = receipt::digest(&layout, &layout.server)?;
    fs::set_permissions(&layout.server, fs::Permissions::from_mode(0o600))?;
    assert_eq!(receipt::digest(&layout, &layout.server)?, before);
    assert!(
        receipt::verify_binary(&layout, &receipt, &layout.server, "redis_server_sha256").is_err()
    );
    Ok(())
}

#[test]
fn given_path_swap_after_open_when_chmodding_then_only_captured_inode_changes() -> TestResult {
    let (_directory, layout) = fixture()?;
    fs::set_permissions(&layout.module, fs::Permissions::from_mode(0o600))?;
    let captured = super::super::open_regular(&layout.module)?;
    let saved = layout.root.join("saved-module");
    fs::rename(&layout.module, &saved)?;
    fs::write(&layout.module, b"foreign")?;
    fs::set_permissions(&layout.module, fs::Permissions::from_mode(0o600))?;
    super::super::make_executable(&captured)?;
    assert_eq!(fs::metadata(saved)?.permissions().mode() & 0o777, 0o755);
    assert_eq!(
        fs::metadata(&layout.module)?.permissions().mode() & 0o777,
        0o600
    );
    Ok(())
}
