use crate::{
    cli::SweepArgs,
    sweep_runtime::{self, SweepConfig, SweepSummary},
};
use anyhow::{Context, Result};
use inventory_aws::model_credentials::ProfileCredentials;
use inventory_core::catalog::runtime::RuntimeCatalog;
use inventory_engine::inventory::{BinaryStore, SchemaRegistry, run_projection};
use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};

pub(super) async fn run(options: SweepArgs) -> Result<std::process::ExitCode> {
    let (store, mut summary) = collect(&options).await?;
    if let Some(endpoint) = &options.graph_url {
        project(&store, endpoint, &options.graph_name).await?;
    }
    summary.graph_pending = store.projection_state().await?.pending;
    let code = exit_code(&summary.outcomes);
    print_summary(summary)?;
    Ok(code)
}

async fn collect(options: &SweepArgs) -> Result<(BinaryStore, SweepSummary)> {
    let bytes = read_catalog(&options.catalog)?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let store = open_store(options, &catalog).await?;
    let credentials =
        ProfileCredentials::load(options.profile.as_deref(), options.region.as_deref()).await?;
    let summary = sweep_runtime::run(&catalog, &store, config(options), &credentials).await?;
    Ok((store, summary))
}

fn exit_code(outcomes: &sweep_runtime::ScopeOutcomes) -> std::process::ExitCode {
    if outcomes.partial > 0
        || outcomes.inaccessible > 0
        || outcomes.unsupported > 0
        || outcomes.failed > 0
        || outcomes.unobserved > 0
    {
        std::process::ExitCode::from(2)
    } else {
        std::process::ExitCode::SUCCESS
    }
}

fn read_catalog(path: &Path) -> Result<Vec<u8>> {
    let file = File::open(path).with_context(|| format!("opening catalog {}", path.display()))?;
    let mut bytes = Vec::new();
    file.take(16_777_217).read_to_end(&mut bytes)?;
    anyhow::ensure!(bytes.len() <= 16_777_216, "runtime catalog exceeds 16 MiB");
    Ok(bytes)
}

async fn open_store(options: &SweepArgs, catalog: &RuntimeCatalog<'_>) -> Result<BinaryStore> {
    if std::fs::metadata(&options.database).is_ok_and(|metadata| metadata.len() > 0) {
        let store = BinaryStore::open(&options.database).await?;
        store.validate_catalog(catalog.raw_bytes()).await?;
        return Ok(store);
    }
    anyhow::ensure!(!options.resume, "cannot resume a missing database");
    let registry = SchemaRegistry::new(catalog.raw_bytes(), catalog.schemas().to_vec())?;
    Ok(BinaryStore::create(&options.database, registry).await?)
}

fn config(options: &SweepArgs) -> SweepConfig {
    SweepConfig {
        region: options.region.clone(),
        max_pages: options.max_pages,
        concurrency: options.concurrency,
        resume: options.resume,
        endpoint: options.endpoint.clone(),
        ca_bundle: options.ca_bundle.clone(),
    }
}

async fn project(store: &BinaryStore, endpoint: &str, graph: &str) -> Result<()> {
    loop {
        let report = run_projection(store, endpoint, graph, 1000).await?;
        if report.offline
            || report.rebuild_incomplete
            || report.pending == 0
            || report.acknowledged == 0
        {
            return Ok(());
        }
    }
}

fn print_summary(summary: SweepSummary) -> Result<()> {
    let outcomes = summary.outcomes;
    writeln!(
        std::io::stdout().lock(),
        "resource_writes_scope=invocation resource_writes={} rows={} scopes={} complete={} partial={} inaccessible={} unsupported={} failed={} unobserved={} graph_pending={}",
        summary.resource_writes,
        summary.rows,
        summary.scopes,
        outcomes.complete,
        outcomes.partial,
        outcomes.inaccessible,
        outcomes.unsupported,
        outcomes.failed,
        outcomes.unobserved,
        summary.graph_pending
    )?;
    Ok(())
}
