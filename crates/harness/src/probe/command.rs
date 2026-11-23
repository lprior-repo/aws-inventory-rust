mod args;
mod credentials;
mod load;

use super::ProbeError;
use args::Args;
use credentials::EnvironmentCredentials;
use inventory_core::{catalog::runtime::RuntimeCatalog, inventory::RegionId};
use inventory_engine::limiter::{ApiExecutor, ExecutorConfig};
use std::process::ExitCode;

pub async fn command(arguments: &[String]) -> ExitCode {
    match execute(arguments).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

async fn execute(arguments: &[String]) -> Result<(), ProbeError> {
    let args = Args::parse(arguments)?;
    let credentials = EnvironmentCredentials::load(args.region.clone())?;
    let region = RegionId::parse(&args.region)?;
    let (bytes, client) =
        tokio::task::spawn_blocking(move || load::load(&args.catalog, &args.ca_bundle)).await??;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let executor = executor()?;
    let text = super::run(
        &catalog,
        &credentials,
        &client,
        &executor,
        &region,
        &args.endpoint,
        &args.services,
    )
    .await?;
    tokio::fs::write(args.report, text).await?;
    Ok(())
}

fn executor() -> Result<ApiExecutor, ProbeError> {
    Ok(ApiExecutor::new(ExecutorConfig {
        global_concurrency: 8,
        per_bucket_concurrency: 8,
        max_attempts: 1,
        ..ExecutorConfig::default()
    })?)
}
