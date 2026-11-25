mod classes_support;

use catalog_tool::{
    ReportMode,
    classes::{build_report_with_policy, run_with_policy},
};
use classes_support::{REPORT, fixture, policy, write};
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

#[test]
fn explicit_policy_print_preserves_exact_report_without_creating_an_artifact() -> TestResult {
    let dir = fixture()?;
    assert_eq!(build_report_with_policy(dir.path(), &policy()?)?, REPORT);
    run_with_policy(dir.path(), dir.path(), ReportMode::Print, &policy()?)?;
    assert!(!dir.path().join(COMMITTED).exists());
    Ok(())
}

#[test]
fn explicit_policy_check_preserves_the_exact_artifact() -> TestResult {
    let dir = fixture()?;
    write(dir.path(), COMMITTED, REPORT)?;
    run_with_policy(dir.path(), dir.path(), ReportMode::Check, &policy()?)?;
    assert_eq!(std::fs::read_to_string(dir.path().join(COMMITTED))?, REPORT);
    Ok(())
}

#[test]
fn explicit_policy_write_replaces_stale_bytes() -> TestResult {
    let dir = fixture()?;
    write(dir.path(), COMMITTED, "stale\n")?;
    run_with_policy(dir.path(), dir.path(), ReportMode::Write, &policy()?)?;
    assert_eq!(std::fs::read_to_string(dir.path().join(COMMITTED))?, REPORT);
    Ok(())
}

#[test]
fn cli_builtin_policy_failure_never_prints_or_replaces_partial_reports() -> TestResult {
    let dir = fixture()?;
    write(dir.path(), COMMITTED, "preserve\n")?;
    [vec![], vec!["--write"]]
        .into_iter()
        .try_for_each(|flags| -> TestResult {
            let output = classes(dir.path(), &flags)?;
            assert_eq!(output.status.code(), Some(1));
            assert_eq!(output.stdout, b"");
            assert!(String::from_utf8(output.stderr)?.contains("matches no operation"));
            assert_eq!(
                std::fs::read_to_string(dir.path().join(COMMITTED))?,
                "preserve\n"
            );
            Ok(())
        })?;
    Ok(())
}

#[test]
fn cli_conflicting_modes_fail_without_creating_an_artifact() -> TestResult {
    let dir = fixture()?;
    let output = classes(dir.path(), &["--check", "--write"])?;
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, b"");
    assert!(String::from_utf8(output.stderr)?.contains("conflicting flag"));
    assert!(!dir.path().join(COMMITTED).exists());
    Ok(())
}

#[test]
fn cli_missing_models_fail_without_creating_an_artifact() -> TestResult {
    let dir = tempfile::tempdir()?;
    let output = classes(dir.path(), &["--write"])?;
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, b"");
    assert!(!output.stderr.is_empty());
    assert!(!dir.path().join(COMMITTED).exists());
    Ok(())
}
