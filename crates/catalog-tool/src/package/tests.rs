//! Packaging contracts: selection, ordering, hashes and determinism.

use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;

use zip::{DateTime, ZipArchive};

use super::{package, sha256_hex};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const INCLUDED: [&str; 4] = [
    "README.md",
    "artifacts/emulator-v04/inventory.sqlite",
    "artifacts/keep/note.txt",
    "src/main.rs",
];
const PREFIX: &str = "aws-inventory-rs/";
const TREE: [(&str, &str); 24] = [
    ("README.md", "readme\n"),
    ("src/main.rs", "fn main() {}\n"),
    ("artifacts/keep/note.txt", "note\n"),
    ("artifacts/emulator-v04/inventory.sqlite", "sqlite\n"),
    ("target/debug/out", "binary\n"),
    (".git/config", "git\n"),
    ("__pycache__/mod.pyc", "cache\n"),
    (".pytest_cache/state", "cache\n"),
    ("mutants.out.old/log.txt", "mutation\n"),
    ("notes.pyc", "cache\n"),
    ("pkg.deb", "package\n"),
    ("lib.so", "shared\n"),
    ("MANIFEST.sha256", "stale\n"),
    ("artifacts/emulator-seed-preflight/x.txt", "seed\n"),
    ("artifacts/emulator-v04-first/x.txt", "first\n"),
    ("artifacts/emulator-oracle-123/x.txt", "oracle\n"),
    ("credentials", "secret\n"),
    ("emulator_credentials", "secret\n"),
    ("ca.pem", "secret\n"),
    ("server.key", "secret\n"),
    ("leaf.key", "secret\n"),
    ("inventory.db", "database\n"),
    ("inventory.db-wal", "database\n"),
    ("inventory.sqlite", "database\n"),
];

fn synthetic_tree(root: &Path) -> TestResult {
    for (name, content) in TREE {
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
        text.push_str(&sha256_hex(&fs::read(root.join(name))?));
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
        names.push(archive.by_index(index)?.name()?.into_owned());
    }
    Ok(names)
}

fn entry_bytes(path: &Path, name: &str) -> TestResult<Vec<u8>> {
    let mut archive = ZipArchive::new(File::open(path)?)?;
    let mut bytes = Vec::new();
    archive.by_name(name)?.read_to_end(&mut bytes)?;
    Ok(bytes)
}

fn expected_entry_names() -> Vec<String> {
    let mut names: Vec<String> = INCLUDED
        .iter()
        .map(|name| format!("{PREFIX}{name}"))
        .collect();
    names.push(format!("{PREFIX}MANIFEST.sha256"));
    names
}

#[test]
fn given_tree_with_exclusions_when_packaged_then_payload_and_manifest_are_exact() -> TestResult {
    let tree = tempfile::tempdir()?;
    let output = tempfile::tempdir()?;
    synthetic_tree(tree.path())?;
    let destination = output.path().join("nested/bundle.zip");
    let receipt = package(tree.path(), &destination)?;

    let manifest = fs::read_to_string(tree.path().join("MANIFEST.sha256"))?;
    assert_eq!(manifest, expected_manifest(tree.path())?);
    assert_eq!(receipt.files, 5);
    assert_eq!(receipt.bytes, fs::metadata(&destination)?.len());
    assert_eq!(receipt.sha256, sha256_hex(&fs::read(&destination)?));
    assert_eq!(entry_names(&destination)?, expected_entry_names());
    for name in INCLUDED {
        assert_eq!(
            entry_bytes(&destination, &format!("{PREFIX}{name}"))?,
            fs::read(tree.path().join(name))?
        );
    }
    assert_eq!(
        entry_bytes(&destination, &format!("{PREFIX}MANIFEST.sha256"))?,
        manifest.as_bytes()
    );
    Ok(())
}

#[test]
fn given_packaged_archive_when_manifest_is_rehashed_then_every_entry_matches() -> TestResult {
    let tree = tempfile::tempdir()?;
    let output = tempfile::tempdir()?;
    synthetic_tree(tree.path())?;
    let destination = output.path().join("bundle.zip");
    package(tree.path(), &destination)?;
    let manifest = fs::read_to_string(tree.path().join("MANIFEST.sha256"))?;
    let parsed = parse_manifest(&manifest)?;
    let expected: Vec<String> = INCLUDED.iter().map(|name| name.to_string()).collect();
    assert_eq!(parsed.keys().cloned().collect::<Vec<String>>(), expected);
    verify_manifest_entries(&destination, &parsed)?;
    Ok(())
}

/// Parse `sha256  name` lines, rejecting duplicates and malformed digests.
fn parse_manifest(manifest: &str) -> TestResult<BTreeMap<String, String>> {
    let mut parsed = BTreeMap::new();
    for line in manifest.lines() {
        let (digest, name) = line
            .split_once("  ")
            .ok_or("manifest line lacks the two-space separator")?;
        assert_eq!(digest.len(), 64);
        assert!(
            parsed
                .insert(name.to_string(), digest.to_string())
                .is_none()
        );
    }
    Ok(parsed)
}

/// Re-hash every archive payload and compare it with its manifest entry.
fn verify_manifest_entries(destination: &Path, parsed: &BTreeMap<String, String>) -> TestResult {
    for (name, digest) in parsed {
        let bytes = entry_bytes(destination, &format!("{PREFIX}{name}"))?;
        assert_eq!(sha256_hex(&bytes), *digest);
    }
    Ok(())
}

#[test]
fn given_same_tree_when_packaged_twice_then_bytes_order_and_timestamps_are_identical() -> TestResult
{
    let tree = tempfile::tempdir()?;
    let first = tempfile::tempdir()?;
    let second = tempfile::tempdir()?;
    synthetic_tree(tree.path())?;
    let one = first.path().join("bundle.zip");
    let two = second.path().join("bundle.zip");
    package(tree.path(), &one)?;
    package(tree.path(), &two)?;
    assert_eq!(fs::read(&one)?, fs::read(&two)?);
    assert_eq!(entry_names(&one)?, expected_entry_names());
    Ok(())
}

#[test]
fn given_packaged_archive_when_inspected_then_every_entry_has_the_fixed_timestamp() -> TestResult {
    let tree = tempfile::tempdir()?;
    let output = tempfile::tempdir()?;
    synthetic_tree(tree.path())?;
    let destination = output.path().join("bundle.zip");
    package(tree.path(), &destination)?;
    let expected = DateTime::from_date_and_time(1980, 1, 1, 0, 0, 0)?;
    let mut archive = ZipArchive::new(File::open(&destination)?)?;
    for index in 0..archive.len() {
        assert_eq!(archive.by_index(index)?.last_modified(), Some(expected));
    }
    Ok(())
}

#[test]
fn given_destination_child_when_packaged_then_parent_directories_are_created() -> TestResult {
    let tree = tempfile::tempdir()?;
    let output = tempfile::tempdir()?;
    synthetic_tree(tree.path())?;
    let destination = output.path().join("a/b/c/bundle.zip");
    let receipt = package(tree.path(), &destination)?;
    assert!(destination.is_file());
    assert_eq!(
        receipt.path,
        fs::canonicalize(&destination)?.to_string_lossy()
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
    let mut expected = vec![format!("{PREFIX}.localrc")];
    expected.extend(INCLUDED.iter().map(|n| format!("{PREFIX}{n}")));
    expected.push(format!("{PREFIX}MANIFEST.sha256"));
    assert_eq!(entry_names(&output.path().join("bundle.zip"))?, expected);
    assert_eq!(receipt.files, 6);
    Ok(())
}

#[test]
fn given_payload_root_when_packaged_then_only_one_manifest_entry_exists() -> TestResult {
    let tree = tempfile::tempdir()?;
    let output = tempfile::tempdir()?;
    synthetic_tree(tree.path())?;
    let destination = output.path().join("bundle.zip");
    package(tree.path(), &destination)?;
    let names = entry_names(&destination)?;
    assert_eq!(
        names
            .iter()
            .filter(|name| name.ends_with("MANIFEST.sha256"))
            .count(),
        1
    );
    Ok(())
}
