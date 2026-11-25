use std::error::Error;
use std::fs::{self, File};
use std::io::Read;
use std::path::PathBuf;

use inventory_harness::mutate;
use tempfile::TempDir;

type TestResult = Result<(), Box<dyn Error>>;

fn create_temp_dir(prefix: &str) -> Result<TempDir, Box<dyn Error>> {
    Ok(tempfile::Builder::new().prefix(prefix).tempdir()?)
}

#[test]
fn test_drain_budget_default() -> TestResult {
    let budget = mutate::command_drain_budget()?;
    assert_eq!(budget, 35);
    Ok(())
}

#[test]
fn test_receipt_json_serialization() -> TestResult {
    let receipt = mutate::Receipt {
        argv: vec![String::from("test")],
        workspace: PathBuf::from("/tmp"),
        deadline_seconds: 60,
        status: String::from("completed"),
        log: PathBuf::from("/tmp/test.log"),
        pid: Some(123),
        process_group: Some(123),
        cleanup_budget_seconds: Some(35),
        child_cleanup_budget_seconds: Some(8),
        cleanup: None,
        child_exit_code: Some(0),
        exit_code: Some(0),
        elapsed_seconds: 1.5,
        fixture_health_checks: 3,
        interrupt_signal: None,
        fixture_error_type: None,
        fixture_error: None,
        supervisor_error: None,
        log_sha256: Some("abc123".to_string()),
    };
    let json = receipt.to_json();
    assert_eq!(
        json.get("status").and_then(|v| v.as_str()),
        Some("completed")
    );
    assert_eq!(json.get("exit_code").and_then(|v| v.as_i64()), Some(0));
    assert_eq!(
        json.get("argv")
            .and_then(|v| v.get(0))
            .and_then(|v| v.as_str()),
        Some("test")
    );
    Ok(())
}

#[test]
fn test_receipt_new_defaults() -> TestResult {
    let temp = create_temp_dir("mutate-test-")?;
    let output = temp.path().to_path_buf();
    let receipt = mutate::Receipt::new(
        vec![String::from("echo")],
        PathBuf::from("/tmp"),
        60,
        &output,
        "test-label",
    );
    assert_eq!(receipt.status, "running");
    assert_eq!(receipt.exit_code, None);
    assert_eq!(receipt.pid, None);
    assert_eq!(receipt.log, output.join("test-label.log"));
    drop(temp);
    Ok(())
}

#[tokio::test]
async fn test_execute_command_simple() -> TestResult {
    let temp = create_temp_dir("mutate-execute-")?;
    let output = temp.path().to_path_buf();
    let receipt = mutate::execute_command(mutate::CommandRequest {
        arguments: vec![String::from("echo"), String::from("hello")],
        workspace: PathBuf::from("/tmp"),
        output: output.clone(),
        label: "simple".to_string(),
        scan_id: None,
        port: 0,
        deadline_seconds: 30,
        health_check: Some(Box::new(|| Ok(()))),
    })
    .await?;

    assert_eq!(receipt.status, "completed");
    assert_eq!(receipt.exit_code, Some(0));
    assert!(receipt.elapsed_seconds >= 0.0);

    let log_path = output.join("simple.log");
    assert!(log_path.exists());
    let mut log = File::open(&log_path)?;
    let mut content = String::new();
    log.read_to_string(&mut content)?;
    assert!(content.contains("hello"));

    let receipt_path = output.join("simple.json");
    assert!(receipt_path.exists());

    drop(temp);
    Ok(())
}

#[tokio::test]
async fn test_execute_command_nonzero_exit() -> TestResult {
    let temp = create_temp_dir("mutate-nonzero-")?;
    let output = temp.path().to_path_buf();
    let receipt = mutate::execute_command(mutate::CommandRequest {
        arguments: vec![
            String::from("sh"),
            String::from("-c"),
            String::from("exit 42"),
        ],
        workspace: PathBuf::from("/tmp"),
        output: output.clone(),
        label: "nonzero".to_string(),
        scan_id: None,
        port: 0,
        deadline_seconds: 30,
        health_check: Some(Box::new(|| Ok(()))),
    })
    .await?;

    assert_eq!(receipt.status, "completed");
    assert_eq!(receipt.exit_code, Some(42));

    drop(temp);
    Ok(())
}

#[tokio::test]
async fn test_execute_command_timeout() -> TestResult {
    let temp = create_temp_dir("mutate-timeout-")?;
    let output = temp.path().to_path_buf();
    let receipt = mutate::execute_command(mutate::CommandRequest {
        arguments: vec![
            String::from("sh"),
            String::from("-c"),
            String::from("sleep 60"),
        ],
        workspace: PathBuf::from("/tmp"),
        output: output.clone(),
        label: "timeout".to_string(),
        scan_id: None,
        port: 0,
        deadline_seconds: 2,
        health_check: Some(Box::new(|| Ok(()))),
    })
    .await?;

    assert_eq!(receipt.status, "timeout");
    assert_eq!(receipt.exit_code, Some(124));
    assert!(receipt.elapsed_seconds >= 1.0);
    assert!(receipt.elapsed_seconds <= 4.0);

    drop(temp);
    Ok(())
}

#[tokio::test]
async fn test_execute_command_empty_args() {
    let result = mutate::execute_command(mutate::CommandRequest {
        arguments: vec![],
        workspace: PathBuf::from("/tmp"),
        output: PathBuf::from("/tmp"),
        label: "test".to_string(),
        scan_id: None,
        port: 0,
        deadline_seconds: 60,
        health_check: Some(Box::new(|| Ok(()))),
    })
    .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_execute_command_too_many_args() {
    let mut args = vec![];
    for i in 0..129 {
        args.push(format!("arg{i}"));
    }
    let result = mutate::execute_command(mutate::CommandRequest {
        arguments: args,
        workspace: PathBuf::from("/tmp"),
        output: PathBuf::from("/tmp"),
        label: "test".to_string(),
        scan_id: None,
        port: 0,
        deadline_seconds: 60,
        health_check: Some(Box::new(|| Ok(()))),
    })
    .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_execute_command_spawn_failure() -> TestResult {
    let temp = create_temp_dir("mutate-spawn-")?;
    let output = temp.path().to_path_buf();
    let receipt = mutate::execute_command(mutate::CommandRequest {
        arguments: vec![String::from("/nonexistent/binary")],
        workspace: PathBuf::from("/tmp"),
        output: output.clone(),
        label: "spawn".to_string(),
        scan_id: None,
        port: 0,
        deadline_seconds: 30,
        health_check: Some(Box::new(|| Ok(()))),
    })
    .await?;

    assert_eq!(receipt.status, "supervisor_error");
    assert!(receipt.supervisor_error.is_some());

    drop(temp);
    Ok(())
}

#[test]
fn test_write_receipt_file() -> TestResult {
    let temp = create_temp_dir("mutate-receipt-")?;
    let output = temp.path().to_path_buf();
    let receipt = mutate::Receipt::new(
        vec![String::from("test")],
        PathBuf::from("/tmp"),
        60,
        &output,
        "test-label",
    );

    mutate::write_receipt(&receipt, &output, "test-label")?;

    let receipt_path = output.join("test-label.json");
    assert!(receipt_path.exists());

    let content = fs::read_to_string(&receipt_path)?;
    let parsed: serde_json::Value = serde_json::from_str(&content)?;
    assert_eq!(
        parsed.get("status").and_then(|v| v.as_str()),
        Some("running")
    );

    drop(temp);
    Ok(())
}
