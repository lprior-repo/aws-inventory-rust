use super::stack::Stack;
use crate::HarnessError;
use std::{path::Path, process::Stdio, time::Duration};
use tokio::process::Command;

#[derive(Clone, Copy, Debug)]
pub struct SweepTarget<'a> {
    pub endpoint: &'a str,
    pub timeout: Duration,
}

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
    let target = SweepTarget {
        endpoint,
        timeout: Duration::from_secs(300),
    };
    sweep_at_with_timeout(stack, collector, catalog, database, services, log, target).await
}

pub async fn sweep_at_with_timeout(
    stack: &Stack,
    collector: &Path,
    catalog: &Path,
    database: &str,
    services: &[String],
    log: &Path,
    target: SweepTarget<'_>,
) -> Result<String, HarnessError> {
    let mut command = Command::new(collector);
    command
        .arg("sweep")
        .args(["--database", database])
        .arg("--catalog")
        .arg(catalog);
    configure_request(stack, &mut command, target.endpoint);
    services.iter().for_each(|service| {
        command.args(["--service", service]);
    });
    stack.configure(&mut command);
    execute_with_timeout(command, log, target.timeout).await
}

fn configure_request(stack: &Stack, command: &mut Command, endpoint: &str) {
    command.args(["--profile", "default", "--region", "us-east-1"]);
    command.args(["--endpoint", endpoint]);
    command.arg("--ca-bundle").arg(stack.emulator().ca_bundle());
    command.args(["--graph-url", stack.graph().url()]);
    command.args(["--graph-name", "aws_inventory_e2e"]);
    command.args(["--max-pages", "100000", "--concurrency", "4"]);
}

pub async fn execute(command: Command, log: &Path) -> Result<String, HarnessError> {
    execute_with_timeout(command, log, Duration::from_secs(300)).await
}

pub async fn execute_with_timeout(
    mut command: Command,
    log: &Path,
    timeout: Duration,
) -> Result<String, HarnessError> {
    let file = std::fs::File::create(log)?;
    command
        .stdin(Stdio::null())
        .stdout(Stdio::from(file.try_clone()?))
        .stderr(Stdio::from(file))
        .kill_on_drop(true);
    let mut child = command.spawn()?;
    let status = wait(&mut child, timeout).await?;
    let summary = super::verify::read_text(log)?;
    let outcome = super::verify::outcome(&summary);
    if !outcome.is_empty() {
        println!("collector {outcome}");
    }
    super::outcome::accept(status.code(), &summary)?;
    Ok(summary)
}

async fn wait(
    child: &mut tokio::process::Child,
    timeout: Duration,
) -> Result<std::process::ExitStatus, HarnessError> {
    match tokio::time::timeout(timeout, child.wait()).await {
        Ok(status) => Ok(status?),
        Err(_) => {
            child.kill().await?;
            Err(HarnessError::invalid(format!(
                "collector exceeded {timeout:?}; child stopped"
            )))
        }
    }
}

#[cfg(test)]
mod tests;
