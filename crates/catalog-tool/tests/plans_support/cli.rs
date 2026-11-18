use super::{
    TestResult,
    report_fixture::{EXPECTED, fixture, policy},
    super_write,
};
use catalog_tool::{
    ReportMode,
    plans::{build_report_with_policy, run_with_policy},
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

#[test]
fn explicit_policy_print_preserves_exact_report_without_creating_committed_artifact() -> TestResult
{
    let dir = fixture()?;
    assert_eq!(build_report_with_policy(dir.path(), &policy()?)?, EXPECTED);
    run_with_policy(dir.path(), dir.path(), ReportMode::Print, &policy()?)?;
    assert!(!dir.path().join(REPORT).exists());
    Ok(())
}

#[test]
fn write_atomically_replaces_stale_bytes_and_is_idempotent() -> TestResult {
    let dir = fixture()?;
    super_write(dir.path(), REPORT, "stale\n")?;
    run_with_policy(dir.path(), dir.path(), ReportMode::Write, &policy()?)?;
    assert_eq!(std::fs::read_to_string(dir.path().join(REPORT))?, EXPECTED);
    run_with_policy(dir.path(), dir.path(), ReportMode::Write, &policy()?)?;
    assert_eq!(std::fs::read_to_string(dir.path().join(REPORT))?, EXPECTED);
    Ok(())
}

#[test]
fn check_accepts_exact_bytes_and_preserves_artifact() -> TestResult {
    let dir = fixture()?;
    super_write(dir.path(), REPORT, EXPECTED)?;
    run_with_policy(dir.path(), dir.path(), ReportMode::Check, &policy()?)?;
    assert_eq!(std::fs::read_to_string(dir.path().join(REPORT))?, EXPECTED);
    Ok(())
}

#[test]
fn check_rejects_single_byte_difference_without_rewriting_it() -> TestResult {
    let dir = fixture()?;
    let stale = EXPECTED.replacen("operations 17", "operations 18", 1);
    super_write(dir.path(), REPORT, &stale)?;
    let error = run_with_policy(dir.path(), dir.path(), ReportMode::Check, &policy()?)
        .err()
        .ok_or("changed report accepted")?;
    let error = error.to_string();
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

#[test]
fn cli_builtin_policy_failure_never_prints_or_replaces_a_partial_report() -> TestResult {
    let dir = fixture()?;
    super_write(dir.path(), REPORT, "preserve\n")?;
    [vec![], vec!["--write"]]
        .into_iter()
        .try_for_each(|flags| -> TestResult {
            let result = plans(dir.path(), &flags)?;
            assert_eq!(result.status.code(), Some(1));
            assert_eq!(result.stdout, b"");
            assert!(String::from_utf8(result.stderr)?.contains("matches no operation"));
            assert_eq!(
                std::fs::read_to_string(dir.path().join(REPORT))?,
                "preserve\n"
            );
            Ok(())
        })?;
    Ok(())
}

#[test]
fn cli_missing_models_fail_without_creating_a_report() -> TestResult {
    let dir = tempfile::tempdir()?;
    let result = plans(dir.path(), &["--write"])?;
    assert_eq!(result.status.code(), Some(1));
    assert_eq!(result.stdout, b"");
    assert!(!result.stderr.is_empty());
    assert!(!dir.path().join(REPORT).exists());
    Ok(())
}
