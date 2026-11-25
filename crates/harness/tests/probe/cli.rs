use super::server::{Server, TestResult};
use std::path::Path;

pub async fn run(
    bytes: &[u8],
    options: &[&str],
    respond: impl Fn(&str) -> TestResult<(u16, &'static str)>,
) -> TestResult<(String, Vec<String>)> {
    let server = Server::new().await?;
    let work = tempfile::tempdir()?;
    tokio::fs::write(work.path().join("probe.catalog"), bytes).await?;
    let mut command = command(&server, work.path())?;
    command.args(options);
    let probe = async {
        let output = command.output().await?;
        assert_eq!(
            output.status.code(),
            Some(0),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        Ok::<_, Box<dyn std::error::Error>>(())
    };
    let ((), requests) = server.capture_with(probe, respond).await?;
    let report = tokio::fs::read_to_string(work.path().join("probe.report")).await?;
    Ok((report, requests))
}

fn command(server: &Server, work: &Path) -> TestResult<tokio::process::Command> {
    let mut command = tokio::process::Command::new(env!("CARGO_BIN_EXE_inventory-harness"));
    configure(&mut command, server, work)?;
    command
        .env("AWS_ACCESS_KEY_ID", "TEST")
        .env("AWS_SECRET_ACCESS_KEY", "TEST")
        .env_remove("AWS_SESSION_TOKEN")
        .env("PATH", work.join("no-python"))
        .kill_on_drop(true);
    Ok(command)
}

fn configure(command: &mut tokio::process::Command, server: &Server, work: &Path) -> TestResult {
    command
        .arg("probe")
        .arg("--catalog")
        .arg(work.join("probe.catalog"))
        .arg("--endpoint")
        .arg(server.endpoint()?)
        .arg("--ca-bundle")
        .arg(&server.ca)
        .arg("--region")
        .arg("us-east-1")
        .arg("--report")
        .arg(work.join("probe.report"));
    Ok(())
}

#[tokio::test]
async fn given_repeated_service_filters_when_cli_runs_then_only_selected_service_is_reported()
-> TestResult {
    let bytes = super::fixture::compiled(4, |_| {})?;
    let (report, requests) = run(
        &bytes,
        &["--service", "bindings", "--service", "bindings"],
        super::server::response,
    )
    .await?;
    assert_eq!(requests.len(), 5);
    assert!(
        report
            .lines()
            .any(|line| line == "bindings ListParents0 awsJson1_1 items 2")
    );
    assert_eq!(
        report
            .lines()
            .filter(|line| line.starts_with("bindings "))
            .count(),
        5
    );
    Ok(())
}

#[tokio::test]
async fn given_unknown_service_when_cli_runs_then_it_fails_without_writing_a_report() -> TestResult
{
    let bytes = super::fixture::compiled(1, |_| {})?;
    let server = Server::new().await?;
    let work = tempfile::tempdir()?;
    tokio::fs::write(work.path().join("probe.catalog"), bytes).await?;
    let output = command(&server, work.path())?
        .args(["--service", "missing"])
        .output()
        .await?;
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("unknown probe service"));
    assert!(!work.path().join("probe.report").exists());
    Ok(())
}
