#[path = "../../../engine/tests/inventory_store/support.rs"]
mod support;

use super::{TestResult, native, recovery, runtime};
use inventory_engine::inventory::{BinaryStore, SweepId, run_projection};

async fn insert_resources(
    store: &BinaryStore,
    sweep: &str,
    first: usize,
    count: usize,
) -> TestResult {
    store.seed_work(support::work(sweep, "us-east-1")?).await?;
    let lease = store.claim(&SweepId::new(sweep.to_owned())?, 1, 60_000)
        .await?.ok_or("boundary fixture work was not claimable")?;
    let observations = (first..first + count)
        .map(|index| support::observation(&format!("rebuild{index:05}"), "retained"))
        .collect::<Result<Vec<_>, _>>()?;
    store.commit_page(support::page(lease, observations, true, None, 2)).await?;
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn given_exact_rebuild_page_budget_when_native_projection_runs_then_eof_completes_but_one_more_resource_stays_pending() -> TestResult {
    let _permit = runtime::NATIVE_SLOT.acquire().await?;
    let prepared = native::PreparedGraph::install()?;
    let emulator = runtime::NativeRuntime::start()?;
    let graph = prepared.start(&emulator).await?;
    let (directory, store, _) = support::fixture().await?;
    let database = directory.path().join("inventory.sqlite");
    let expected = (0..10_000).map(|index| vec![format!("rebuild{index:05}")])
        .collect::<Vec<_>>();

    insert_resources(&store, "boundary_initial", 0, 10_000).await?;
    native::project(&store, &graph).await?;
    recovery::request_rebuild(&database).await?;
    let exact = run_projection(&store, graph.endpoint(), native::GRAPH, 1).await?;
    assert!(!exact.offline);
    assert!(!exact.rebuild_incomplete, "exactly 10000 full pages must acknowledge EOF");
    assert_eq!(exact.pending, 0);
    assert_eq!(exact.acknowledged, 1);
    assert!(!store.projection_state().await?.rebuild_required);
    assert_eq!(graph.rows("MATCH (r:Resource) RETURN r.id ORDER BY r.id").await?, expected);

    insert_resources(&store, "boundary_extra", 10_000, 1).await?;
    native::project(&store, &graph).await?;
    recovery::request_rebuild(&database).await?;
    let exceeded = run_projection(&store, graph.endpoint(), native::GRAPH, 1).await?;
    assert!(!exceeded.offline);
    assert!(exceeded.rebuild_incomplete, "a terminal probe must not project beyond the page budget");
    assert_eq!(exceeded.pending, 1);
    assert_eq!(exceeded.acknowledged, 0);
    assert!(store.projection_state().await?.rebuild_required);
    assert_eq!(graph.rows("MATCH (r:Resource) RETURN r.id ORDER BY r.id").await?, expected);
    graph.stop().await?;
    emulator.stop()
}
