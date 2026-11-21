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
const MODEL: &str = r#"{"smithy":"2.0","shapes":{
 "n#Svc":{"type":"service","version":"2020-01-01","traits":{
   "smithy.protocols#rpcv2Cbor":{},
   "smithy.rules#endpointRuleSet":{"version":"1.0","parameters":{},"rules":[
     {"type":"endpoint","conditions":[],"endpoint":{"url":"https://example.test"}}
   ]},
   "smithy.rules#endpointTests":{"testCases":[
     {"params":{},"expect":{"endpoint":{"url":"https://example.test"}}}
   ]},
   "aws.auth#sigv4":{"name":"example"}}},
 "n#ListItems":{"type":"operation"},
 "n#Item":{"type":"structure","members":{"Name":{"target":"smithy.api#String"}}}
}}"#;
fn source() -> Result<tempfile::TempDir, Box<dyn std::error::Error>> {
    git::repository(MODEL)
}
fn root() -> Result<tempfile::TempDir, Box<dyn std::error::Error>> {
    let root = tempfile::tempdir()?;
    let catalog = root.path().join("catalog");
    fs::create_dir_all(&catalog)?;
    fs::write(catalog.join("field-ids.txt"), "Name\n")?;
    fs::write(
        catalog.join("cloudcontrol-universal.json"),
        include_bytes!("../../../catalog/cloudcontrol-universal.json"),
    )?;
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
fn success(output: &Output) {
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stderr, b"");
    assert_eq!(output.stdout, b"");
}
#[test]
fn write_twice_and_check_publish_identical_reader_visible_bytes() -> TestResult {
    let source = source()?;
    let root = root()?;
    let path = root.path().join("catalog/aws-api.catalog");
    let first = compile(source.path(), root.path(), &["--write"])?;
    let bytes = fs::read(&path)?;
    success(&first);
    success(&compile(source.path(), root.path(), &["--write"])?);
    assert_eq!(fs::read(&path)?, bytes);
    success(&compile(source.path(), root.path(), &["--check"])?);
    Ok(())
}
#[test]
fn written_catalog_is_readable_without_changing_the_source_or_registry() -> TestResult {
    let source = source()?;
    let root = root()?;
    let output = compile(source.path(), root.path(), &["--write"])?;
    let path = root.path().join("catalog/aws-api.catalog");
    let bytes = fs::read(&path)?;
    success(&output);
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
    let source = source()?;
    let root = root()?;
    let output = compile(source.path(), root.path(), &["--write", "--check"])?;
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("conflicting flag"));
    assert!(!root.path().join("catalog/aws-api.catalog").exists());
    Ok(())
}

#[test]
fn pending_report_is_deterministic_and_default_compile_is_silent() -> TestResult {
    let source = source()?;
    let root = root()?;
    let first = root.path().join("pending-first.txt");
    let second = root.path().join("pending-second.txt");
    for report in [&first, &second] {
        let report = report.to_string_lossy();
        success(&compile(
            source.path(),
            root.path(),
            &["--write", "--pending-report", report.as_ref()],
        )?);
    }
    let first = fs::read(first)?;
    assert_eq!(first, fs::read(second)?);
    assert!(!first.is_empty());
    success(&compile(source.path(), root.path(), &["--write"])?);
    Ok(())
}
