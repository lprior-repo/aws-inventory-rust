mod args;
mod credentials;
mod load;
mod receipt;

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
    if let Some(when) = &args.compared_at {
        let probe = super::Probe {
            catalog: &catalog,
            credentials: &credentials,
            client: &client,
            executor: &executor,
            region: &region,
            endpoint: &args.endpoint,
            services: &args.services,
        };
        let (report, overlays) = probe.compare(&args.pairs, when).await?;
        return receipt::publish(args.report, report, Some(overlays)).await;
    }
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
    receipt::publish(args.report, text, None).await
}

fn executor() -> Result<ApiExecutor, ProbeError> {
    Ok(ApiExecutor::new(ExecutorConfig {
        global_concurrency: 8,
        per_bucket_concurrency: 8,
        max_attempts: 1,
        ..ExecutorConfig::default()
    })?)
}
