use super::server::{Server, TestResult};
use std::{path::Path, time::Duration};

pub async fn report(bytes: &[u8]) -> TestResult<String> {
    let server = Server::new().await?;
    let work = tempfile::tempdir()?;
    tokio::fs::write(work.path().join("probe.catalog"), bytes).await?;
    let mut command = command(&server, work.path())?;
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
    tokio::time::timeout(Duration::from_secs(10), async {
        tokio::try_join!(probe, server.serve(4))
    })
    .await??;
    Ok(tokio::fs::read_to_string(work.path().join("probe.report")).await?)
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
