use super::stack::Stack;
use crate::HarnessError;
use std::{path::Path, process::Stdio, time::Duration};
use tokio::process::Command;

pub async fn sweep(
    stack: &Stack,
    collector: &Path,
    catalog: &Path,
    database: &str,
    services: &[String],
    log: &Path,
) -> Result<String, HarnessError> {
    sweep_at(
        stack,
        collector,
        catalog,
        database,
        services,
        log,
        stack.emulator().endpoint(),
    )
    .await
}

pub async fn sweep_at(
    stack: &Stack,
    collector: &Path,
    catalog: &Path,
    database: &str,
    services: &[String],
    log: &Path,
    endpoint: &str,
) -> Result<String, HarnessError> {
    let mut command = Command::new(collector);
    command
        .arg("sweep")
        .args(["--database", database])
        .arg("--catalog")
        .arg(catalog);
    configure_request(stack, &mut command, endpoint);
    services.iter().for_each(|service| {
        command.args(["--service", service]);
    });
    stack.configure(&mut command);
    execute(command, log).await
}

fn configure_request(stack: &Stack, command: &mut Command, endpoint: &str) {
    command.args(["--profile", "default", "--region", "us-east-1"]);
    command.args(["--endpoint", endpoint]);
    command.arg("--ca-bundle").arg(stack.emulator().ca_bundle());
    command.args(["--graph-url", stack.graph().url()]);
    command.args(["--graph-name", "aws_inventory_e2e"]);
    command.args(["--max-pages", "100000", "--concurrency", "4"]);
}

pub async fn execute(mut command: Command, log: &Path) -> Result<String, HarnessError> {
    let file = std::fs::File::create(log)?;
    command
        .stdin(Stdio::null())
        .stdout(Stdio::from(file.try_clone()?))
        .stderr(Stdio::from(file))
        .kill_on_drop(true);
    let mut child = command.spawn()?;
    let status = wait(&mut child).await?;
    let summary = super::verify::read_text(log)?;
    let outcome = super::verify::outcome(&summary);
    if !outcome.is_empty() {
        println!("collector {outcome}");
    }
    if !status.success() {
        return Err(HarnessError::invalid("collector failed; see stage log"));
    }
    super::verify::zero(&summary, "failed")?;
    Ok(summary)
}

async fn wait(child: &mut tokio::process::Child) -> Result<std::process::ExitStatus, HarnessError> {
    match tokio::time::timeout(Duration::from_secs(300), child.wait()).await {
        Ok(status) => Ok(status?),
        Err(_) => {
            child.kill().await?;
            Err(HarnessError::invalid(
                "collector exceeded 300 seconds; child stopped",
            ))
        }
    }
}
