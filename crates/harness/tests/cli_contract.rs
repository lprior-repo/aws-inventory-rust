use std::path::Path;
use std::process::{Command, Output};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn harness(arguments: &[&str]) -> std::io::Result<Output> {
    Command::new(env!("CARGO_BIN_EXE_inventory-harness"))
        .args(arguments)
        .env_clear()
        .output()
}

fn text(path: &Path) -> Result<&str, &'static str> {
    path.to_str().ok_or("temporary path is not UTF-8")
}

#[test]
fn failed_assertion_exits_nonzero_and_still_prints_the_checks() -> TestResult {
    let directory = tempfile::tempdir()?;
    let database = directory.path().join("missing.sqlite");
    let root = format!("{}/../..", env!("CARGO_MANIFEST_DIR"));
    let arguments = ["assert", "cloud-control", "--root", &root, "--db"];
    let output = harness(&[&arguments[..], &[text(&database)?, "--scan-id", "missing"]].concat())?;

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.contains("\"passed\": false"), "{stdout}");
    assert!(!database.exists());
    Ok(())
}

#[test]
fn install_rejects_an_unknown_flag_before_creating_the_destination() -> TestResult {
    let directory = tempfile::tempdir()?;
    let destination = directory.path().join("out");
    let root = text(directory.path())?;
    let output = harness(&[
        "install",
        text(&destination)?,
        "--root",
        root,
        "--unexpected",
        "junk",
    ])?;

    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8(output.stderr)?;
    assert!(stderr.starts_with("unexpected --unexpected;"), "{stderr}");
    assert!(!destination.exists());
    Ok(())
}

#[test]
fn repeated_flag_is_rejected_before_any_request() -> TestResult {
    let output = harness(&["seed", "s3", "--bucket", "first", "--bucket", "second"])?;

    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8(output.stderr)?;
    assert!(stderr.starts_with("unexpected --bucket;"), "{stderr}");
    Ok(())
}

#[test]
fn verify_delivery_accepts_run_workspace_tests_flag() -> TestResult {
    let directory = tempfile::tempdir()?;
    let database = directory.path().join("inventory.sqlite");
    let output_path = directory.path().join("output.json");
    let binary = text(directory.path())?;
    let redis_cli = text(directory.path())?;

    let output = harness(&[
        "verify-delivery",
        "--binary",
        binary,
        "--redis-cli",
        redis_cli,
        "--database",
        text(&database)?,
        "--output",
        text(&output_path)?,
        "--run-workspace-tests",
    ])?;

    // Expect failure (missing database) but not a CLI parse error
    let stderr = String::from_utf8(output.stderr)?;
    assert!(
        !stderr.starts_with("unexpected --run-workspace-tests;"),
        "flag should be recognized: {stderr}",
    );
    Ok(())
}

#[test]
fn verify_delivery_rejects_unknown_flag() -> TestResult {
    let directory = tempfile::tempdir()?;
    let database = directory.path().join("inventory.sqlite");
    let output_path = directory.path().join("output.json");
    let binary = text(directory.path())?;
    let redis_cli = text(directory.path())?;

    let output = harness(&[
        "verify-delivery",
        "--binary",
        binary,
        "--redis-cli",
        redis_cli,
        "--database",
        text(&database)?,
        "--output",
        text(&output_path)?,
        "--unknown-flag",
    ])?;

    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8(output.stderr)?;
    assert!(stderr.starts_with("unexpected --unknown-flag;"), "{stderr}");
    Ok(())
}
