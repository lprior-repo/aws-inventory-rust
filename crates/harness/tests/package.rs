//! Offline SDK catalog contracts.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use inventory_harness::HarnessError;
use inventory_harness::package::{
    SdkMode, sdk_check, sdk_refresh, sdk_run_in, verify_catalog_cache,
};
use inventory_harness::provenance::file_sha256;
use serde_json::json;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn workspace() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn copy_tree(source: &Path, destination: &Path) -> TestResult {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let target = destination.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_tree(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}

const CLIENT_FEATURES: [&str; 3] = ["default-https-client", "rt-tokio", "sigv4"];

fn synthetic_crate(directory: &Path, name: &str, version: &str) -> TestResult<PathBuf> {
    let member = format!("{name}-{version}");
    let staging = directory.join(&member);
    fs::create_dir_all(&staging)?;
    let list: Vec<String> = CLIENT_FEATURES
        .iter()
        .map(|feature| format!("\"{feature}\""))
        .collect();
    fs::write(
        staging.join("Cargo.toml"),
        format!("[features]\ndefault = [{}]\n", list.join(", ")),
    )?;
    let archive = directory.join(format!("{member}.crate"));
    let mut command = Command::new("tar");
    command
        .arg("-czf")
        .arg(&archive)
        .arg("-C")
        .arg(directory)
        .arg(member);
    let status = command.status()?;
    if !status.success() {
        return Err("synthetic .crate creation failed".into());
    }
    Ok(archive)
}

fn invalid_message(error: HarnessError) -> TestResult<String> {
    match error {
        HarnessError::Invalid(message) => Ok(message),
        other => Err(format!("expected invalid, got {other:?}").into()),
    }
}

#[test]
fn given_committed_catalog_when_checked_then_rust_reproduces_every_file() -> TestResult {
    let report = sdk_check(&workspace())?;
    assert_eq!(report.service_count, 437);
    assert_eq!(report.mode, SdkMode::Check);
    assert!(report.newer_registry_versions.is_empty());
    assert!(report.cache.is_none());
    assert_eq!(report.release, json!("release-2026-11-13"));
    Ok(())
}

#[test]
fn given_tampered_generated_file_when_checked_then_rust_reports_the_file() -> TestResult {
    let fixture = tempfile::tempdir()?;
    let root = fixture.path();
    copy_tree(
        &workspace().join("crates/sdk-catalog"),
        &root.join("crates/sdk-catalog"),
    )?;
    let fragmented = root.join("crates/sdk-catalog/src/lib.rs");
    let mut text = fs::read_to_string(&fragmented)?;
    text.push_str("// tampered\n");
    fs::write(&fragmented, text)?;
    let error = sdk_check(root).err().ok_or("tampered catalog accepted")?;
    assert_eq!(
        invalid_message(error)?,
        "generated file differs: crates/sdk-catalog/src/lib.rs"
    );
    Ok(())
}

#[test]
fn given_refresh_requested_when_called_then_typed_failure_is_returned() -> TestResult {
    let error = sdk_refresh()
        .err()
        .ok_or("network refresh unexpectedly succeeded")?;
    assert_eq!(
        invalid_message(error)?,
        "network refresh is not implemented in the Rust harness"
    );
    Ok(())
}

#[test]
fn given_empty_cargo_home_when_verifying_cache_then_registry_bound_is_reported() -> TestResult {
    let home = tempfile::tempdir()?;
    let error = sdk_run_in(&workspace(), SdkMode::Check, true, Some(home.path()))
        .err()
        .ok_or("empty Cargo home accepted")?;
    assert_eq!(
        invalid_message(error)?,
        "expected 1..=16 Cargo registry cache directories"
    );
    Ok(())
}

#[test]
fn given_cached_crate_when_verified_then_features_and_size_are_accepted() -> TestResult {
    let home = tempfile::tempdir()?;
    let registry = home.path().join("registry/cache/one");
    fs::create_dir_all(&registry)?;
    let (name, version) = ("aws-sdk-test", "1.0.0");
    let archive = synthetic_crate(&registry, name, version)?;
    let digest = file_sha256(&archive)?;
    let bytes = fs::metadata(&archive)?.len();
    let document = json!({
        "service_count": 1,
        "services": [{
            "name": name,
            "version": version,
            "checksum_sha256": digest,
            "client_features": CLIENT_FEATURES,
        }],
    });
    let report = verify_catalog_cache(&document, home.path())?;
    assert_eq!(report.verified_sdk_archives, 1);
    assert_eq!(report.verified_client_feature_sets, 1);
    assert_eq!(report.sdk_archive_bytes, bytes);
    Ok(())
}
