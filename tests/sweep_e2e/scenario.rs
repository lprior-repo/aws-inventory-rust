#[path = "scenario/arguments.rs"]
mod arguments;
#[path = "scenario/projection.rs"]
mod projection;
use super::barrier_proxy::BarrierProxy;
use super::denial_peer::AccessDeniedPeer;
use super::guard::Environment;
use super::process::{run_cli, run_cli_degraded, spawn_cli, terminate_cli};
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
    interrupted_resume_matches_full(&graph_environment, oracle, &initial).await
}

async fn offline_baseline(
    environment: &Environment,
    database: &Database,
    oracle: &TypedSeedOracle,
) -> Result<(InventorySnapshot, u64)> {
    let mut offline = environment.clone();
    offline.graph_url = closed_loopback_url()?;
    let full = run_cli(&offline, &sweep_args(&offline, database, 100, false)?).await?;
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

async fn interrupted_resume_matches_full(
    environment: &Environment,
    oracle: &TypedSeedOracle,
    expected: &InventorySnapshot,
) -> Result<()> {
    let directory = tempfile::tempdir()?;
    let database =
        Database::create(environment, directory.path().join("interrupted.sqlite")).await?;
    let mut proxy = BarrierProxy::start(&environment.endpoint, &environment.ca_bundle).await?;
    let checkpoint = interrupt(&mut proxy, environment, &database).await?;
    let resumed = run_cli(environment, &sweep_args(environment, &database, 100, true)?).await?;
    verify_resumed(&resumed, expected)?;
    ensure!(
        &database_snapshot(&database, oracle).await? == expected,
        "resumed checkpoint differs from uninterrupted canonical inventory"
    );
    super::state::assert_sweep_finished(&database, &checkpoint.sweep_id).await
}

async fn interrupt(
    proxy: &mut BarrierProxy,
    environment: &Environment,
    database: &Database,
) -> Result<super::state::ActiveCheckpoint> {
    let stopped = stop_at_checkpoint(proxy, environment, database).await;
    let shutdown = proxy.shutdown().await;
    let checkpoint = stopped?;
    shutdown?;
    Ok(checkpoint)
}

fn verify_resumed(summary: &str, expected: &InventorySnapshot) -> Result<()> {
    ensure!(
        summary_value(summary, "graph_pending")? == 0,
        "resumed CLI did not project through configured Falkor endpoint"
    );
    assert_no_failures(summary)?;
    ensure!(
        summary_value(summary, "rows")? == u64::try_from(expected.resources.len())?,
        "resumed summary rows differ from uninterrupted SQL counts"
    );
    Ok(())
}

async fn stop_at_checkpoint(
    proxy: &mut BarrierProxy,
    environment: &Environment,
    database: &Database,
) -> Result<super::state::ActiveCheckpoint> {
    let endpoint_url = proxy.endpoint();
    let endpoint = EndpointOverride {
        url: &endpoint_url,
        ca_bundle: proxy.ca_bundle(),
    };
    let mut arguments = sweep_args_for(environment, database, 100, false, Some(endpoint))?;
    set_concurrency(&mut arguments, 1)?;
    let process = spawn_cli(environment, &arguments).await?;
    let active = observe_checkpoint(proxy, database).await;
    terminate_cli(process).await?;
    active
}

async fn observe_checkpoint(
    proxy: &mut BarrierProxy,
    database: &Database,
) -> Result<super::state::ActiveCheckpoint> {
    let observed = proxy.wait_for_continuation().await?;
    ensure!(
        (observed.action == "DescribeVpcs" || observed.action == "DescribeSubnets")
            && !observed.token.is_empty(),
        "barrier must observe a native AWS EC2 continuation token"
    );
    let active = super::state::active_checkpoint(database).await?;
    ensure!(
        active.resources_observed > 0 && active.work_pending > 0,
        "first native page was not committed before interruption"
    );
    Ok(active)
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
