use super::*;
type TestResult = Result<(), Box<dyn std::error::Error>>;

#[tokio::test]
async fn given_expired_caller_deadline_when_waiting_then_collector_is_killed_and_reaped()
-> TestResult {
    let mut child = Command::new("sleep")
        .arg("3600")
        .kill_on_drop(true)
        .spawn()?;
    let result = wait(&mut child, Duration::ZERO).await;
    assert!(result.is_err());
    assert!(child.try_wait()?.is_some());
    if let Err(error) = result {
        assert!(error.to_string().contains("exceeded 0ns"));
    }
    Ok(())
}

#[tokio::test]
async fn given_longer_deadline_when_collector_finishes_with_outcomes_then_canonical_contract_applies()
-> TestResult {
    let root = tempfile::tempdir()?;
    let mut command = Command::new("sh");
    command.args(["-c", "printf 'resource_writes_scope=invocation resource_writes=1 rows=1 complete=1 partial=0 unsupported=1 failed=1 work_remaining=0 graph_pending=0\\n'; exit 2"]);
    let summary = execute_with_timeout(
        command,
        &root.path().join("collector.log"),
        Duration::from_secs(3600),
    )
    .await?;
    assert!(summary.contains("unsupported=1 failed=1"));
    Ok(())
}
