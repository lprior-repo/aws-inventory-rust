//! Packaging and offline SDK catalog contracts.

use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

use inventory_harness::HarnessError;
use inventory_harness::package::{
    SdkMode, package, sdk_check, sdk_refresh, sdk_run_in, verify_catalog_cache,
};
use inventory_harness::provenance::{file_sha256, sha256_hex};
use serde_json::json;
use zip::ZipArchive;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const INCLUDED: [&str; 4] = [
    "README.md",
    "artifacts/emulator-v04/inventory.sqlite",
    "artifacts/keep/note.txt",
    "src/main.rs",
];

fn workspace() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn synthetic_tree(root: &Path) -> TestResult {
    let files = [
        ("README.md", "readme\n"),
        ("src/main.rs", "fn main() {}\n"),
        ("artifacts/keep/note.txt", "note\n"),
        ("artifacts/emulator-v04/inventory.sqlite", "sqlite\n"),
        ("target/debug/out", "binary\n"),
        (".git/config", "git\n"),
        ("__pycache__/mod.pyc", "cache\n"),
        ("notes.pyc", "cache\n"),
        ("pkg.deb", "package\n"),
        ("lib.so", "shared\n"),
        ("MANIFEST.sha256", "stale\n"),
        ("artifacts/emulator-seed-preflight/x.txt", "seed\n"),
        ("artifacts/emulator-oracle-123/x.txt", "oracle\n"),
        ("credentials", "secret\n"),
        ("emulator_credentials", "secret\n"),
        ("ca.pem", "secret\n"),
        ("server.key", "secret\n"),
        ("leaf.key", "secret\n"),
        ("inventory.db", "database\n"),
        ("inventory.db-wal", "database\n"),
    ];
    for (name, content) in files {
        let path = root.join(name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, content)?;
    }
    std::os::unix::fs::symlink("README.md", root.join("link.md"))?;
    Ok(())
}

fn expected_manifest(root: &Path) -> TestResult<String> {
    let mut text = String::new();
    for name in INCLUDED {
        let data = fs::read(root.join(name))?;
        text.push_str(&sha256_hex(&data));
        text.push_str("  ");
        text.push_str(name);
        text.push('\n');
    }
    Ok(text)
}

fn entry_names(path: &Path) -> TestResult<Vec<String>> {
    let mut archive = ZipArchive::new(File::open(path)?)?;
    let mut names = Vec::new();
    for index in 0..archive.len() {
        names.push(archive.by_index(index)?.name().to_string());
    }
    Ok(names)
}

fn entry(path: &Path, name: &str) -> TestResult<(Vec<u8>, u32)> {
    let mut archive = ZipArchive::new(File::open(path)?)?;
    let mut entry = archive.by_name(name)?;
    let crc = entry.crc32();
    let mut bytes = Vec::new();
    entry.read_to_end(&mut bytes)?;
    Ok((bytes, crc))
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
fn given_tree_with_exclusions_when_packaged_then_manifest_and_entries_are_exact() -> TestResult {
    let tree = tempfile::tempdir()?;
    let output = tempfile::tempdir()?;
    synthetic_tree(tree.path())?;
    let destination = output.path().join("bundle.zip");
    let receipt = package(tree.path(), &destination)?;
    let manifest = fs::read_to_string(tree.path().join("MANIFEST.sha256"))?;
    assert_eq!(manifest, expected_manifest(tree.path())?);
    assert_eq!(receipt.files, 5);
    assert_eq!(receipt.bytes, fs::metadata(&destination)?.len());
    assert_eq!(receipt.sha256.len(), 64);

    let prefix = "aws-inventory-rs/";
    let mut names: Vec<String> = INCLUDED.iter().map(|n| format!("{prefix}{n}")).collect();
    names.push(format!("{prefix}MANIFEST.sha256"));
    assert_eq!(entry_names(&destination)?, names);
    for name in INCLUDED {
        let member = format!("{prefix}{name}");
        assert_eq!(
            entry(&destination, &member)?.0,
            fs::read(tree.path().join(name))?
        );
    }
    let manifest_member = format!("{prefix}MANIFEST.sha256");
    assert_eq!(
        entry(&destination, &manifest_member)?.0,
        manifest.as_bytes()
    );
    Ok(())
}

#[test]
fn given_tree_with_local_directory_when_packaged_then_local_files_are_excluded() -> TestResult {
    let tree = tempfile::tempdir()?;
    let output = tempfile::tempdir()?;
    synthetic_tree(tree.path())?;
    fs::write(tree.path().join(".localrc"), "keep")?;
    fs::create_dir_all(tree.path().join(".local"))?;
    fs::write(tree.path().join(".local/config.json"), "config")?;
    fs::create_dir_all(tree.path().join("src/.local"))?;
    fs::write(tree.path().join("src/.local/cache"), "nested")?;
    let receipt = package(tree.path(), &output.path().join("bundle.zip"))?;
    let mut expected = vec!["aws-inventory-rs/.localrc".to_string()];
    expected.extend(INCLUDED.iter().map(|n| format!("aws-inventory-rs/{n}")));
    expected.push("aws-inventory-rs/MANIFEST.sha256".to_string());
    assert_eq!(entry_names(&output.path().join("bundle.zip"))?, expected);
    assert_eq!(receipt.files, 6);
    Ok(())
}

#[test]
fn given_same_tree_when_packaged_by_catalog_tool_and_rust_then_bytes_match() -> TestResult {
    let tree = tempfile::tempdir()?;
    let reference = tempfile::tempdir()?;
    let output = tempfile::tempdir()?;
    synthetic_tree(tree.path())?;
    synthetic_tree(reference.path())?;
    let rust_zip = output.path().join("rust.zip");
    let tool_zip = output.path().join("catalog-tool.zip");
    let rust = package(tree.path(), &rust_zip)?;
    let tool = catalog_tool::package::package(reference.path(), &tool_zip)?;
    assert_eq!(rust.files, tool.files);
    assert_eq!(rust.bytes, tool.bytes);
    let rust_manifest = fs::read_to_string(tree.path().join("MANIFEST.sha256"))?;
    let tool_manifest = fs::read_to_string(reference.path().join("MANIFEST.sha256"))?;
    assert_eq!(rust_manifest, tool_manifest);
    let rust_names = entry_names(&rust_zip)?;
    assert_eq!(rust_names, entry_names(&tool_zip)?);
    for name in &rust_names {
        assert_eq!(entry(&rust_zip, name)?, entry(&tool_zip, name)?);
    }
    Ok(())
}

#[test]
fn given_committed_catalog_when_checked_then_rust_reproduces_every_file() -> TestResult {
    let report = sdk_check(&workspace())?;
    assert_eq!(report.service_count, 437);
    assert_eq!(report.mode, SdkMode::Check);
    assert!(report.newer_registry_versions.is_empty());
    assert!(report.cache.is_none());
    assert_eq!(report.release, json!("release-2026-11-12"));
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
