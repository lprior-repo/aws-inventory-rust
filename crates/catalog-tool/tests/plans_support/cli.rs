use super::{
    TestResult,
    report_fixture::{EXPECTED, fixture},
    super_write,
};
use std::{io, path::Path, process::Output};

const REPORT: &str = "catalog/aws-plans.report";
fn plans(root: &Path, flags: &[&str]) -> io::Result<Output> {
    std::process::Command::new(env!("CARGO_BIN_EXE_catalog-tool"))
        .args(["plans", "--root"])
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
    assert_eq!(output.stdout, b"");
    assert_eq!(output.stderr, b"");
}

#[test]
fn print_emits_exact_report_without_creating_committed_artifact() -> TestResult {
    let dir = fixture()?;
    let output = plans(dir.path(), &[])?;
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stdout, EXPECTED.as_bytes());
    assert_eq!(output.stderr, b"");
    assert!(!dir.path().join(REPORT).exists());
    Ok(())
}

#[test]
fn write_atomically_replaces_stale_bytes_and_is_idempotent() -> TestResult {
    let dir = fixture()?;
    super_write(dir.path(), REPORT, "stale\n")?;
    success(&plans(dir.path(), &["--write"])?);
    assert_eq!(std::fs::read_to_string(dir.path().join(REPORT))?, EXPECTED);
    success(&plans(dir.path(), &["--write"])?);
    assert_eq!(std::fs::read_to_string(dir.path().join(REPORT))?, EXPECTED);
    Ok(())
}

#[test]
fn check_accepts_exact_bytes_and_preserves_artifact() -> TestResult {
    let dir = fixture()?;
    super_write(dir.path(), REPORT, EXPECTED)?;
    success(&plans(dir.path(), &["--check"])?);
    assert_eq!(std::fs::read_to_string(dir.path().join(REPORT))?, EXPECTED);
    Ok(())
}

#[test]
fn check_rejects_single_byte_difference_without_rewriting_it() -> TestResult {
    let dir = fixture()?;
    let stale = EXPECTED.replacen("operations 17", "operations 18", 1);
    super_write(dir.path(), REPORT, &stale)?;
    let result = plans(dir.path(), &["--check"])?;
    assert_eq!(result.status.code(), Some(1));
    assert_eq!(result.stdout, b"");
    let error = String::from_utf8(result.stderr)?;
    assert!(error.contains("operations 17") && error.contains("operations 18"));
    assert_eq!(std::fs::read_to_string(dir.path().join(REPORT))?, stale);
    Ok(())
}

#[test]
fn invalid_model_never_prints_or_replaces_a_partial_report() -> TestResult {
    let dir = fixture()?;
    super_write(dir.path(), REPORT, "preserve\n")?;
    super_write(dir.path(), "models/zeta/service/v1/broken.json", "{")?;
    let result = plans(dir.path(), &["--write"])?;
    assert_eq!(result.status.code(), Some(1));
    assert_eq!(result.stdout, b"");
    assert!(String::from_utf8(result.stderr)?.contains("broken.json"));
    assert_eq!(
        std::fs::read_to_string(dir.path().join(REPORT))?,
        "preserve\n"
    );
    Ok(())
}

#[test]
fn conflicting_cli_modes_fail_without_creating_a_report() -> TestResult {
    let dir = fixture()?;
    let result = plans(dir.path(), &["--check", "--write"])?;
    assert_eq!(result.status.code(), Some(1));
    assert_eq!(result.stdout, b"");
    assert!(String::from_utf8(result.stderr)?.contains("conflicting flag"));
    assert!(!dir.path().join(REPORT).exists());
    Ok(())
}
