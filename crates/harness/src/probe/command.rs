mod args;
mod credentials;
mod load;

use super::ProbeError;
use args::Args;
use credentials::EnvironmentCredentials;
use inventory_core::{catalog::runtime::RuntimeCatalog, inventory::RegionId};
use inventory_engine::limiter::{ApiExecutor, ExecutorConfig};
use std::{process::ExitCode, time::Instant};

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
    let started = Instant::now();
    let (text, root_wall) = super::Probe {
        catalog: &catalog,
        credentials: &credentials,
        client: &client,
        executor: &executor,
        region: &region,
        endpoint: &args.endpoint,
        services: &args.services,
        child_samples: args.child_samples,
    }
    .run()
    .await?;
    tokio::fs::write(args.report, text).await?;
    eprintln!(
        "probe wall time: roots {:.3}s, full {:.3}s",
        root_wall.as_secs_f64(),
        started.elapsed().as_secs_f64(),
    );
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
