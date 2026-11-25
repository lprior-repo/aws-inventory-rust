use std::{
    error::Error,
    path::Path,
    process::{Command, Output},
};
type TestResult = Result<(), Box<dyn Error>>;
fn compare(before: &Path, after: &Path, flags: &[&str]) -> Result<Output, std::io::Error> {
    Command::new(env!("CARGO_BIN_EXE_cmdb-bench"))
        .arg("compare")
        .arg(before)
        .arg(after)
        .args(flags)
        .output()
}
fn table(profile: &str, value: u64) -> String {
    format!(
        "# profile\t{profile}\n# server.fsync\ton\nmeasurement\tmin\tmedian\tmax\nP1.cold_wall_ms\t{value}\t{value}\t{value}\n"
    )
}
#[test]
fn given_real_cli_when_medians_regress_then_exit_and_verdict_follow_band() -> TestResult {
    let work = tempfile::tempdir()?;
    let before = work.path().join("before.tsv");
    let after = work.path().join("after.tsv");
    std::fs::write(&before, table("thin", 100))?;
    std::fs::write(&after, table("thin", 105))?;
    assert!(compare(&before, &after, &[])?.status.success());
    std::fs::write(&after, table("thin", 106))?;
    let output = compare(&before, &after, &[])?;
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8(output.stdout)?.contains("regressed"));
    assert!(compare(&before, &after, &["--band", "6"])?.status.success());
    Ok(())
}
#[test]
fn given_different_settings_when_override_used_then_warning_is_explicit() -> TestResult {
    let work = tempfile::tempdir()?;
    let before = work.path().join("before.tsv");
    let after = work.path().join("after.tsv");
    std::fs::write(&before, table("thin", 100))?;
    std::fs::write(&after, table("full", 100))?;
    assert_eq!(compare(&before, &after, &[])?.status.code(), Some(2));
    let output = compare(&before, &after, &["--allow-different-server"])?;
    assert!(output.status.success());
    assert!(String::from_utf8(output.stdout)?.contains("not comparable"));
    std::fs::write(
        &after,
        "measurement\tmin\tmedian\tmax\nP2.warm_wall_ms\t1\t1\t1\n",
    )?;
    let output = compare(&before, &after, &["--allow-different-server"])?;
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8(output.stderr)?.contains("missing"));
    Ok(())
}
#[test]
fn given_zero_runs_when_sweep_requested_then_cli_refuses_before_launching() -> TestResult {
    let output = Command::new(env!("CARGO_BIN_EXE_cmdb-bench"))
        .args([
            "sweep",
            "--collector",
            "absent",
            "--profile",
            "thin",
            "--runs",
            "0",
        ])
        .output()?;
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8(output.stderr)?.contains("--runs must be in 1..=100"));
    Ok(())
}
#[test]
fn given_invalid_compare_arguments_then_refused_is_distinct_from_regression() -> TestResult {
    let output = Command::new(env!("CARGO_BIN_EXE_cmdb-bench"))
        .args(["compare", "--unknown"])
        .output()?;
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8(output.stderr)?.contains("refused"));
    Ok(())
}
