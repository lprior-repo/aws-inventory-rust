#[path = "scenario/arguments.rs"]
mod arguments;
#[path = "scenario/interruption.rs"]
mod interruption;
#[path = "scenario/projection.rs"]
mod projection;
#[path = "scenario/sql_reader.rs"]
mod sql_reader;
#[path = "scenario/sql_writing.rs"]
mod sql_writing;
use super::denial_peer::AccessDeniedPeer;
use super::guard::Environment;
use super::process::{cancel_cli, run_cli, run_cli_degraded, run_cli_graph_pending, spawn_cli};
use super::stack::Stack;
use super::state::{Database, InventorySnapshot, assert_graph, database_snapshot};
use anyhow::{Context, Result, ensure};
use arguments::*;
use aws_wire::sign::Credentials;
use inventory_core::catalog::runtime::RuntimeCatalog;
use inventory_harness::inventory_seed::{SeedConfig, TypedSeedOracle, run_seed};
use std::io::Read;
use std::net::TcpListener;
use std::path::Path;

pub(super) async fn run_live() -> Result<()> {
    let stack = Stack::start().await?;
    let oracle = seed_live(stack.environment(), stack.credentials()).await?;
    run_pipeline(stack.environment(), &oracle).await
}

pub(super) async fn run_denied() -> Result<()> {
    let stack = Stack::start().await?;
    let oracle = seed_live(stack.environment(), stack.credentials()).await?;
    run_denied_pipeline(stack.environment(), &oracle).await
}

async fn seed_live(
    environment: &Environment,
    credentials: &Credentials,
) -> Result<TypedSeedOracle> {
    let bytes = read_catalog(&environment.catalog)?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    Ok(run_seed(
        &catalog,
        SeedConfig::new(
            &environment.endpoint,
            &environment.ca_bundle,
            &environment.region,
        )?,
        credentials,
    )
    .await?)
}
fn read_catalog(path: &Path) -> Result<Vec<u8>> {
    let limit = inventory_core::catalog::MAX_CATALOG_BYTES;
    let mut bytes = Vec::new();
    std::fs::File::open(path)?
        .take(u64::try_from(limit)?.saturating_add(1))
        .read_to_end(&mut bytes)?;
    ensure!(
        bytes.len() <= limit,
        "compiled catalog exceeds its {limit} byte bound"
    );
    Ok(bytes)
}

async fn run_pipeline(environment: &Environment, oracle: &TypedSeedOracle) -> Result<()> {
    let directory = tempfile::tempdir()?;
    let database = Database::create(environment, directory.path().join("schema14.sqlite")).await?;
    let mut graph_environment = environment.clone();
    graph_environment.graph_name = format!("{}-{}", environment.graph_name, uuid::Uuid::new_v4());
    let (initial, pending) = offline_baseline(&graph_environment, &database, oracle).await?;
    project_offline_then_online(
        &database,
        &graph_environment,
        &graph_environment.graph_name,
        oracle,
        pending,
    )
    .await?;
    unchanged_rerun(&graph_environment, &database, oracle, &initial).await?;
    partial_preserves_seed(&graph_environment, &database, oracle, &initial).await?;
    if matches!(database, Database::Postgres(_)) {
        sql_writing::run(&graph_environment, oracle, &initial).await?;
    }
    interruption::run(&graph_environment, oracle, &initial).await
}

async fn offline_baseline(
    environment: &Environment,
    database: &Database,
    oracle: &TypedSeedOracle,
) -> Result<(InventorySnapshot, u64)> {
    let mut offline = environment.clone();
    offline.graph_url = closed_loopback_url()?;
    let full =
        run_cli_graph_pending(&offline, &sweep_args(&offline, database, 100, false)?).await?;
    let initial = database_snapshot(database, oracle).await?;
    let pending = verify_initial(&full, &initial)?;
    Ok((initial, pending))
}

fn verify_initial(summary: &str, initial: &InventorySnapshot) -> Result<u64> {
    ensure!(
        summary_value(summary, "resource_writes")? >= 2,
        "initial sweep must persist seeded resources"
    );
    assert_no_failures(summary)?;
    assert_baseline_budget(summary)?;
    ensure!(
        summary_value(summary, "rows")? == u64::try_from(initial.resources.len())?,
        "CLI row summary must match SQL-backed rows"
    );
    let pending = summary_value(summary, "graph_pending")?;
    ensure!(
        pending > 0,
        "CLI offline projection must retain durable graph work"
    );
    Ok(pending)
}

async fn project_offline_then_online(
    database: &Database,
    environment: &Environment,
    graph: &str,
    oracle: &TypedSeedOracle,
    expected_pending: u64,
) -> Result<()> {
    projection::run(database, environment, graph, expected_pending).await?;
    assert_graph(database, &environment.graph_url, graph, oracle).await?;
    Ok(())
}

async fn unchanged_rerun(
    environment: &Environment,
    database: &Database,
    oracle: &TypedSeedOracle,
    initial: &InventorySnapshot,
) -> Result<()> {
    let unchanged = run_cli(environment, &sweep_args(environment, database, 100, false)?).await?;
    ensure!(
        summary_value(&unchanged, "resource_writes")? == 0,
        "unchanged rerun must write zero resource rows"
    );
    assert_no_failures(&unchanged)?;
    assert_baseline_budget(&unchanged)?;
    ensure!(
        summary_value(&unchanged, "graph_pending")? == 0,
        "unchanged CLI projection must leave no graph work"
    );
    ensure!(
        &database_snapshot(database, oracle).await? == initial,
        "unchanged rerun changed the persisted inventory digest"
    );
    Ok(())
}

async fn partial_preserves_seed(
    environment: &Environment,
    database: &Database,
    oracle: &TypedSeedOracle,
    initial: &InventorySnapshot,
) -> Result<()> {
    for _ in 0..2 {
        let partial =
            run_cli_degraded(environment, &sweep_args(environment, database, 1, false)?).await?;
        ensure!(
            summary_value(&partial, "partial")? > 0,
            "one-page run must encounter an incomplete scope"
        );
        assert_no_failures(&partial)?;
        ensure!(
            &database_snapshot(database, oracle).await? == initial,
            "repeated partial runs changed resources, edges, or missing counters"
        );
    }
    Ok(())
}

async fn run_denied_pipeline(environment: &Environment, oracle: &TypedSeedOracle) -> Result<()> {
    let directory = tempfile::tempdir()?;
    let database = Database::create(environment, directory.path().join("denied.sqlite")).await?;
    let mut graph_environment = environment.clone();
    graph_environment.graph_name = format!("{}-{}", environment.graph_name, uuid::Uuid::new_v4());
    let before = denied_baseline(&graph_environment, &database, oracle).await?;
    let peer = AccessDeniedPeer::start(oracle.account_id.as_str(), oracle.region.as_str()).await?;
    let denied =
        denied_preserves_inventory(&graph_environment, &database, oracle, &before, &peer).await;
    let stopped = peer.shutdown().await;
    denied?;
    stopped?;
    Ok(())
}

async fn denied_baseline(
    environment: &Environment,
    database: &Database,
    oracle: &TypedSeedOracle,
) -> Result<InventorySnapshot> {
    let full = run_cli(environment, &sweep_args(environment, database, 100, false)?).await?;
    ensure!(
        summary_value(&full, "resource_writes")? > 0 && summary_value(&full, "graph_pending")? == 0,
        "denial regression requires a full emulator baseline projected by the CLI"
    );
    assert_no_failures(&full)?;
    assert_baseline_budget(&full)?;
    let before = database_snapshot(database, oracle).await?;
    assert_graph(
        database,
        &environment.graph_url,
        &environment.graph_name,
        oracle,
    )
    .await?;
    Ok(before)
}
