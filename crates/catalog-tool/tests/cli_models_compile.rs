#[path = "cli_models_compile/drift.rs"]
mod drift;
#[path = "../src/smithy/compile/tests/git.rs"]
mod git;
#[path = "cli_models_compile/provenance.rs"]
mod provenance;
use inventory_core::catalog::Catalog;
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};
type TestResult = Result<(), Box<dyn std::error::Error>>;
const MODEL: &str = r#"{"shapes":{
 "n#Svc":{"type":"service","traits":{"smithy.protocols#rpcv2Cbor":{}}},
 "n#ListItems":{"type":"operation"},
 "n#Item":{"type":"structure","members":{"Name":{"target":"smithy.api#String"}}}
}}"#;
fn source() -> Result<tempfile::TempDir, Box<dyn std::error::Error>> {
    git::repository(MODEL)
}
fn root() -> Result<tempfile::TempDir, Box<dyn std::error::Error>> {
    let root = tempfile::tempdir()?;
    fs::create_dir(root.path().join("catalog"))?;
    fs::write(root.path().join("catalog/field-ids.txt"), "Name\n")?;
    Ok(root)
}
fn compile(source: &Path, root: &Path, flags: &[&str]) -> std::io::Result<Output> {
    Command::new(env!("CARGO_BIN_EXE_catalog-tool"))
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .args(["models", "compile", "--source"])
        .arg(source)
        .arg("--root")
        .arg(root)
        .args(flags)
        .output()
}
fn success(output: &Output, length: usize) {
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stderr, b"");
    assert_eq!(
        output.stdout,
        format!("catalog {length} bytes\n").as_bytes()
    );
}
#[test]
fn write_twice_and_check_publish_identical_reader_visible_bytes() -> TestResult {
    // Given isolated source and root, when writing twice and checking, no bytes or IDs drift.
    let source = source()?;
    let root = root()?;
    let path = root.path().join("catalog/aws-api.catalog");
    let first = compile(source.path(), root.path(), &["--write"])?;
    let bytes = fs::read(&path)?;
    success(&first, bytes.len());
    success(
        &compile(source.path(), root.path(), &["--write"])?,
        bytes.len(),
    );
    assert_eq!(fs::read(&path)?, bytes);
    success(
        &compile(source.path(), root.path(), &["--check"])?,
        bytes.len(),
    );
    Ok(())
}
#[test]
fn written_catalog_is_readable_without_changing_the_source_or_registry() -> TestResult {
    // Given a valid source, when publishing, the consumer sees exact counts and IDs stay fixed.
    let source = source()?;
    let root = root()?;
    let output = compile(source.path(), root.path(), &["--write"])?;
    let path = root.path().join("catalog/aws-api.catalog");
    let bytes = fs::read(&path)?;
    success(&output, bytes.len());
    let counts = Catalog::borrow(&bytes)?.counts()?;
    assert_eq!(counts.get("services"), Some(1));
    assert_eq!(counts.get("operations"), Some(1));
    assert_eq!(counts.get("cbor_services"), Some(1));
    assert_eq!(
        fs::read(root.path().join("catalog/field-ids.txt"))?,
        b"Name\n"
    );
    assert!(!path.with_file_name("aws-api.catalog.tmp").exists());
    assert!(!source.path().join("catalog").exists());
    Ok(())
}
#[test]
fn check_rejects_missing_and_drifted_catalog_without_writing() -> TestResult {
    // Given missing then altered committed bytes, when checking, the command fails read-only.
    let source = source()?;
    let root = root()?;
    let path = root.path().join("catalog/aws-api.catalog");
    let output = compile(source.path(), root.path(), &["--check"])?;
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("catalog differs"));
    assert!(!path.exists());
    fs::write(&path, b"drift")?;
    let output = compile(source.path(), root.path(), &["--check"])?;
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(fs::read(path)?, b"drift");
    assert_eq!(output.stdout, b"");
    Ok(())
}
#[test]
fn print_only_reports_size_without_creating_catalog() -> TestResult {
    // Given an existing registry, when no mode is selected, only text is printed.
    let source = source()?;
    let root = root()?;
    let output = compile(source.path(), root.path(), &[])?;
    assert_eq!(output.status.code(), Some(0));
    assert!(String::from_utf8(output.stdout)?.starts_with("catalog "));
    assert!(!root.path().join("catalog/aws-api.catalog").exists());
    Ok(())
}
#[test]
fn stale_registry_preserves_previous_catalog_and_names_missing_member() -> TestResult {
    // Given a missing member ID, when writing, neither output nor registry is changed.
    let source = source()?;
    let root = root()?;
    let path = root.path().join("catalog/aws-api.catalog");
    fs::write(root.path().join("catalog/field-ids.txt"), b"")?;
    fs::write(&path, b"previous")?;
    let output = compile(source.path(), root.path(), &["--write"])?;
    assert_eq!(output.status.code(), Some(1));
    let error = String::from_utf8(output.stderr)?;
    assert!(error.contains("Name"));
    assert!(error.contains("models fields --write"));
    assert_eq!(fs::read(path)?, b"previous");
    assert_eq!(fs::read(root.path().join("catalog/field-ids.txt"))?, b"");
    Ok(())
}
#[test]
fn owned_lock_is_never_overwritten() -> TestResult {
    // Given another publisher's lock, when writing, the lock and original are preserved.
    let source = source()?;
    let root = root()?;
    let lock = root.path().join("catalog/aws-api.catalog.tmp");
    fs::write(&lock, b"other writer")?;
    let output = compile(source.path(), root.path(), &["--write"])?;
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("another run is writing"));
    assert_eq!(fs::read(lock)?, b"other writer");
    assert!(!root.path().join("catalog/aws-api.catalog").exists());
    Ok(())
}
#[test]
fn conflicting_publication_modes_are_rejected() -> TestResult {
    // Given contradictory flags, when parsing the CLI, no compilation is published.
    let source = source()?;
    let root = root()?;
    let output = compile(source.path(), root.path(), &["--write", "--check"])?;
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("conflicting flag"));
    assert!(!root.path().join("catalog/aws-api.catalog").exists());
    Ok(())
}
