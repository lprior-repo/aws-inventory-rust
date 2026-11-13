#![forbid(unsafe_code)]
#![deny(unused_must_use)]

mod analysis;
mod architecture;
mod catalog;
mod cli;
mod coverage;
mod delivery;
mod demo;
mod jobs;
mod plan;
mod report;
#[cfg(feature = "aws")]
mod scan;
mod targets;

use anyhow::{Context, Result};
use clap::Parser;
use cli::{Cli, Command};
use futures_util::future::BoxFuture;
use inventory_engine::{graph, store::Store};
use serde_json::Value;
use std::io::Write;
use std::process::ExitCode;

fn main() -> ExitCode {
    match execute() {
        Ok(value) => output(value),
        Err(error) => {
            eprintln!("aws-inventory: {error:#}");
            ExitCode::FAILURE
        }
    }
}

fn execute() -> Result<Value> {
    let cli = Cli::parse();
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "warn".into()),
        )
        .try_init()
        .map_err(|error| anyhow::anyhow!("logging initialization: {error}"))?;
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    runtime.block_on(dispatch(cli.command))
}

fn output(value: Value) -> ExitCode {
    let mut stdout = std::io::stdout().lock();
    let result = serde_json::to_writer_pretty(&mut stdout, &value)
        .map_err(std::io::Error::other)
        .and_then(|()| stdout.write_all(b"\n"));
    if let Err(error) = result {
        eprintln!("output error: {error}");
        return ExitCode::FAILURE;
    }
    match value.get("status").and_then(Value::as_str) {
        Some("partial" | "cancelled") => ExitCode::from(2),
        _ => ExitCode::SUCCESS,
    }
}

fn dispatch(command: Command) -> BoxFuture<'static, Result<Value>> {
    match command {
        Command::Catalog => Box::pin(std::future::ready(plan::full_catalog())),
        Command::Coverage {
            service,
            type_name,
            unselected,
            limit,
        } => Box::pin(std::future::ready(coverage::report(
            service.as_deref(),
            type_name.as_deref(),
            unselected,
            limit,
        ))),
        Command::Plan(options) => Box::pin(std::future::ready(plan::create(options))),
        Command::SdkCatalog { service } => {
            Box::pin(std::future::ready(catalog::sdk_catalog(service)))
        }
        Command::Demo { db } => Box::pin(demo::create(db)),
        Command::Ingest { db, input, scan_id } => Box::pin(demo::ingest(db, input, scan_id)),
        Command::Export(options) => Box::pin(delivery::export_command(options)),
        Command::Deliver(options) => Box::pin(delivery::command(options)),
        Command::Query { db, sql, limit } => Box::pin(query(db, sql, limit)),
        Command::Report { db, scan_id } => Box::pin(report::report(db, scan_id)),
        Command::Architecture { db, scan_id, limit } => {
            Box::pin(architecture::report(db, scan_id, limit))
        }
        Command::Diff {
            db,
            before,
            after,
            limit,
            max_items,
        } => Box::pin(analysis::diff(db, before, after, limit, max_items)),
        Command::Neighbors {
            db,
            scan_id,
            resource_id,
            traversal,
        } => Box::pin(analysis::neighborhood(
            db,
            scan_id,
            resource_id,
            traversal.into(),
        )),
        Command::Path {
            db,
            scan_id,
            from,
            to,
            traversal,
        } => Box::pin(analysis::path(db, scan_id, from, to, traversal.into())),
        Command::Findings { db, scan_id, limit } => {
            Box::pin(analysis::findings(db, scan_id, limit))
        }
        Command::Discover(options) => Box::pin(run_discovery(options)),
        Command::Job { command } => Box::pin(jobs::execute(command)),
        Command::GraphExport {
            db,
            scan_id,
            graph,
            output,
            allow_partial,
        } => Box::pin(export_graph(db, scan_id, graph, output, allow_partial)),
        Command::GraphPush {
            input,
            checkpoint,
            url_env,
            attempts,
        } => Box::pin(push_graph(input, checkpoint, url_env, attempts)),
        Command::Scan(options) => Box::pin(run_scan(options)),
    }
}

#[tracing::instrument(skip_all)]
async fn query(db: std::path::PathBuf, sql: String, limit: usize) -> Result<Value> {
    require_database(&db)?;
    let store = Store::open(db).await?;
    let result = store.query(&sql, limit).await;
    store.close().await;
    Ok(result?)
}

#[tracing::instrument(skip_all)]
async fn export_graph(
    db: std::path::PathBuf,
    scan_id: String,
    name: String,
    output: std::path::PathBuf,
    allow_partial: bool,
) -> Result<Value> {
    require_database(&db)?;
    let store = Store::open(db).await?;
    let result = export_open_store(&store, &scan_id, &name, &output, allow_partial).await;
    store.close().await;
    result
}

#[tracing::instrument(skip_all)]
async fn export_open_store(
    store: &Store,
    scan_id: &str,
    name: &str,
    output: &std::path::Path,
    allow_partial: bool,
) -> Result<Value> {
    let status = store.scan_status(scan_id).await?;
    anyhow::ensure!(
        allow_partial || status == inventory_core::ScanStatus::Complete,
        "scan is not complete; --allow-partial exports it with its actual status"
    );
    Ok(serde_json::to_value(
        graph::export_snapshot(store, scan_id, name, output).await?,
    )?)
}

#[tracing::instrument(skip_all)]
async fn push_graph(
    input: std::path::PathBuf,
    checkpoint: std::path::PathBuf,
    url_env: String,
    attempts: u32,
) -> Result<Value> {
    let url = std::env::var(&url_env)
        .with_context(|| format!("set {url_env} to the FalkorDB Redis URL"))?;
    let token = tokio_util::sync::CancellationToken::new();
    let work = graph::push_file_with_options(
        input,
        checkpoint,
        &url,
        graph::PushOptions {
            max_attempts: attempts,
            ..graph::PushOptions::default()
        },
        &token,
    );
    let summary = Box::pin(delivery::until_signal(work, &token)).await?;
    let mut result = serde_json::to_value(summary)?;
    result
        .as_object_mut()
        .context("graph push summary must be an object")?
        .insert("status".into(), serde_json::json!("projected"));
    Ok(result)
}

#[cfg(feature = "aws")]
#[tracing::instrument(skip(options))]
async fn run_scan(options: cli::ScanArgs) -> Result<Value> {
    scan::run(options).await
}

#[cfg(not(feature = "aws"))]
#[tracing::instrument(skip(_options))]
fn run_scan(_options: cli::ScanArgs) -> std::future::Ready<Result<Value>> {
    std::future::ready(Err(anyhow::anyhow!(
        "AWS collectors are a compile-time feature: build with cargo build --features aws"
    )))
}

fn require_database(path: &std::path::Path) -> Result<()> {
    anyhow::ensure!(
        path.is_file(),
        "database does not exist: {}",
        path.display()
    );
    Ok(())
}

#[cfg(feature = "aws")]
#[tracing::instrument(skip_all)]
async fn run_discovery(options: cli::DiscoverArgs) -> Result<Value> {
    use inventory_aws::discovery::{DiscoveryOptions, discover};
    use inventory_engine::limiter::{ApiExecutor, ExecutorConfig};
    let source_default =
        !(options.organization || options.config_aggregators || options.resource_explorer);
    let request = DiscoveryOptions {
        profile: options.profile,
        region: options.region,
        organization: options.organization,
        hierarchy: options.hierarchy,
        config_aggregators: options.config_aggregators || source_default,
        resource_explorer: options.resource_explorer || source_default,
        max_pages: options.max_pages,
        page_size: options.page_size,
    };
    let executor = std::sync::Arc::new(ApiExecutor::new(ExecutorConfig::default())?);
    let work = discover(request, std::sync::Arc::clone(&executor));
    tokio::pin!(work);
    let (report, cancelled) = tokio::select! {
        result=&mut work=>(result?,false),
        signal=tokio::signal::ctrl_c()=>{
            executor.cancel();signal?;
            match work.await {
                Ok(report)=>(report,true),
                Err(error)=>return Ok(serde_json::json!({"status":"cancelled","complete":false,"error":error.to_string(),"api":executor.stats()})),
            }
        }
    };
    let complete = report.complete;
    let mut result = serde_json::to_value(report)?;
    let object = result
        .as_object_mut()
        .context("discovery report must be an object")?;
    object.insert(
        "status".into(),
        serde_json::json!(if cancelled {
            "cancelled"
        } else if complete {
            "complete"
        } else {
            "partial"
        }),
    );
    object.insert("api".into(), serde_json::to_value(executor.stats())?);
    Ok(result)
}

#[cfg(not(feature = "aws"))]
fn run_discovery(_options: cli::DiscoverArgs) -> std::future::Ready<Result<Value>> {
    std::future::ready(Err(anyhow::anyhow!(
        "discovery requires an AWS-enabled build"
    )))
}
