use std::process::Command;
type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn given_no_postgres_url_when_run_then_usage_error_names_both_server_options() -> TestResult {
    let work = tempfile::tempdir()?;
    let output = Command::new(env!("CARGO_BIN_EXE_inventory-harness"))
        .args(["e2e", "run", "--work"])
        .arg(work.path())
        .env_remove("INVENTORY_POSTGRES_URL")
        .output()?;
    assert_eq!(output.status.code(), Some(2));
    let message = String::from_utf8(output.stderr)?;
    ["INVENTORY_POSTGRES_URL", "postgres up", "native server"]
        .iter()
        .for_each(|part| assert!(message.contains(part)));
    assert!(std::fs::read_dir(work.path())?.next().is_none());
    Ok(())
}

#[test]
fn given_nonempty_work_when_run_then_no_runtime_starts_or_secret_is_printed() -> TestResult {
    let work = tempfile::tempdir()?;
    std::fs::write(work.path().join("owned"), "preserve")?;
    let output = Command::new(env!("CARGO_BIN_EXE_inventory-harness"))
        .args([
            "e2e",
            "run",
            "--postgres-url",
            "postgres://admin:never-print-this@127.0.0.1:1/postgres",
            "--work",
        ])
        .arg(work.path())
        .output()?;
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8(output.stderr)?;
    assert!(stderr.contains("empty directory"));
    assert!(!stderr.contains("never-print-this"));
    assert!(output.stdout.is_empty());
    assert_eq!(
        std::fs::read_to_string(work.path().join("owned"))?,
        "preserve"
    );
    Ok(())
}
