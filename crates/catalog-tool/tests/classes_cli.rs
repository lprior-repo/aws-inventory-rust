mod classes_support;

use classes_support::{REPORT, fixture, write};
use std::{io, path::Path, process::Output};

type TestResult = Result<(), Box<dyn std::error::Error>>;
const COMMITTED: &str = "catalog/aws-classes.report";

fn classes(root: &Path, flags: &[&str]) -> io::Result<Output> {
    std::process::Command::new(env!("CARGO_BIN_EXE_catalog-tool"))
        .args(["classes", "--root"])
        .arg(root)
        .arg("--source")
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
    assert_eq!(output.stdout, REPORT.as_bytes());
    assert_eq!(output.stderr, b"");
}

#[test]
fn cli_print_emits_exact_report_without_creating_an_artifact() -> TestResult {
    // Given models and no committed report, when printing through the real CLI,
    // then stdout contains the entire report and no artifact is created.
    let dir = fixture()?;
    success(&classes(dir.path(), &[])?);
    assert!(!dir.path().join(COMMITTED).exists());
    Ok(())
}

#[test]
fn cli_check_emits_exact_report_and_preserves_the_artifact() -> TestResult {
    // Given exact committed bytes, when checking through the real CLI,
    // then it succeeds with the complete stdout report and no diagnostics or changes.
    let dir = fixture()?;
    write(dir.path(), COMMITTED, REPORT)?;
    success(&classes(dir.path(), &["--check"])?);
    assert_eq!(std::fs::read_to_string(dir.path().join(COMMITTED))?, REPORT);
    Ok(())
}

#[test]
fn cli_write_emits_exact_report_and_replaces_stale_bytes() -> TestResult {
    // Given stale committed bytes, when writing through the real CLI,
    // then both stdout and the replacement artifact equal the expected report.
    let dir = fixture()?;
    write(dir.path(), COMMITTED, "stale\n")?;
    success(&classes(dir.path(), &["--write"])?);
    assert_eq!(std::fs::read_to_string(dir.path().join(COMMITTED))?, REPORT);
    Ok(())
}
