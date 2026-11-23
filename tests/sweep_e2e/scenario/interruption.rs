use super::super::process::{crash_cli, run_cli_lease_blocked};
use super::super::{barrier_proxy::BarrierProxy, state};
use super::*;

#[derive(Clone, Copy)]
enum Stop {
    Cancel,
    Crash,
}

pub(super) async fn run(
    environment: &Environment,
    oracle: &TypedSeedOracle,
    expected: &InventorySnapshot,
) -> Result<()> {
    let crashed = isolated_graph(environment, "crash");
    crash_preserves_checkpoint(&crashed).await?;
    let cancelled = isolated_graph(environment, "cancel");
    cancelled_resume_matches_full(&cancelled, oracle, expected).await
}

fn isolated_graph(environment: &Environment, scenario: &str) -> Environment {
    let mut isolated = environment.clone();
    let suffix = uuid::Uuid::new_v4();
    isolated.graph_name = format!("{}-{scenario}-{suffix}", environment.graph_name);
    isolated
}

async fn cancelled_resume_matches_full(
    environment: &Environment,
    oracle: &TypedSeedOracle,
    expected: &InventorySnapshot,
) -> Result<()> {
    let directory = tempfile::tempdir()?;
    let database =
        Database::create(environment, directory.path().join("interrupted.sqlite")).await?;
    let mut proxy = BarrierProxy::start(&environment.endpoint, &environment.ca_bundle).await?;
    let checkpoint = interrupt(&mut proxy, environment, &database, Stop::Cancel).await?;
    let resumed = run_cli(environment, &sweep_args(environment, &database, 100, true)?).await?;
    verify_resumed(&resumed, expected)?;
    ensure!(
        &database_snapshot(&database, oracle).await? == expected,
        "resumed checkpoint differs from uninterrupted canonical inventory"
    );
    assert_graph(
        &database,
        &environment.graph_url,
        &environment.graph_name,
        oracle,
    )
    .await?;
    state::assert_sweep_finished(&database, &checkpoint.sweep_id).await
}

async fn crash_preserves_checkpoint(environment: &Environment) -> Result<()> {
    let directory = tempfile::tempdir()?;
    let database = Database::create(environment, directory.path().join("crashed.sqlite")).await?;
    let mut proxy = BarrierProxy::start(&environment.endpoint, &environment.ca_bundle).await?;
    let checkpoint = interrupt(&mut proxy, environment, &database, Stop::Crash).await?;
    let before = state::checkpoint_snapshot(&database).await?;
    run_cli_lease_blocked(environment, &sweep_args(environment, &database, 100, true)?).await?;
    ensure!(
        state::active_checkpoint(&database).await? == checkpoint,
        "rejected crash takeover changed the committed checkpoint"
    );
    ensure!(
        state::checkpoint_snapshot(&database).await? == before,
        "rejected crash takeover changed resources, edges, or missing counters"
    );
    Ok(())
}

async fn interrupt(
    proxy: &mut BarrierProxy,
    environment: &Environment,
    database: &Database,
    stop: Stop,
) -> Result<state::ActiveCheckpoint> {
    let stopped = stop_at_checkpoint(proxy, environment, database, stop).await;
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
    stop: Stop,
) -> Result<state::ActiveCheckpoint> {
    let endpoint_url = proxy.endpoint();
    let endpoint = EndpointOverride {
        url: &endpoint_url,
        ca_bundle: proxy.ca_bundle(),
    };
    let mut arguments = sweep_args_for(environment, database, 100, false, Some(endpoint))?;
    set_concurrency(&mut arguments, 1)?;
    let process = spawn_cli(environment, &arguments).await?;
    let active = observe_checkpoint(proxy, database).await;
    match stop {
        Stop::Cancel => cancel_cli(process).await?,
        Stop::Crash => crash_cli(process).await?,
    }
    active
}

async fn observe_checkpoint(
    proxy: &mut BarrierProxy,
    database: &Database,
) -> Result<state::ActiveCheckpoint> {
    let observed = proxy.wait_for_continuation().await?;
    ensure!(
        (observed.action == "DescribeVpcs" || observed.action == "DescribeSubnets")
            && !observed.token.is_empty(),
        "barrier must observe a native AWS EC2 continuation token"
    );
    let active = state::active_checkpoint(database).await?;
    ensure!(
        active.resources_observed > 0 && active.work_pending > 0,
        "first native page was not committed before interruption"
    );
    Ok(active)
}
