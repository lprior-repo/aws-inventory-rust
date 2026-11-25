mod args;
mod credentials;
mod load;
mod receipt;

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
    let probe = super::Probe {
        catalog: &catalog,
        credentials: &credentials,
        client: &client,
        executor: &executor,
        region: &region,
        endpoint: &args.endpoint,
        services: &args.services,
    };
    if let Some(when) = &args.compared_at {
        let (report, overlays) = probe.compare(&args.pairs, when).await?;
        return receipt::publish(args.report, report, Some(overlays)).await;
    }
    let started = Instant::now();
    let report = probe.run(args.child_samples).await?;
    receipt::publish(args.report, report.text, None).await?;
    eprintln!(
        "probe wall time: roots {:.3}s, full {:.3}s",
        report.root_wall.as_secs_f64(),
        started.elapsed().as_secs_f64(),
    );
    if let Some(error) = report.child_error {
        return Err(ProbeError::ChildSchedule(error));
    }
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
