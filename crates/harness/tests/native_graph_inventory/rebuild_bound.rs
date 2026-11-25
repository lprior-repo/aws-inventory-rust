use super::{TestResult, collisions, native, recovery, runtime};
use inventory_core::inventory::{AccountId, Kind, Region, ResourceId, ResourceKey};
use inventory_engine::inventory::{BinaryStore, SweepId, run_projection};

async fn insert_resources(
    store: &BinaryStore,
    sweep: &str,
    first: usize,
    count: usize,
) -> TestResult {
    let end = first.checked_add(count).ok_or("invalid fixture range")?;
    let account = AccountId::parse("111111111111")?;
    let region = Region::parse("us-east-1")?;
    let kind = Kind::parse("AWS::EC2::VPC")?;
    let keys = (first..end).map(|index| {
        Ok(ResourceKey {
            account: account.clone(),
            region: region.clone(),
            kind: kind.clone(),
            id: ResourceId::parse(&format!("rebuild{index:05}"))?,
        })
    });
    collisions::insert(store, SweepId::new(sweep.to_owned())?, keys).await
}

#[tokio::test]
async fn given_rebuild_fixture_ranges_when_committed_then_all_resource_ids_are_retained()
-> TestResult {
    use sqlx::{Connection, SqliteConnection, sqlite::SqliteConnectOptions};

    let directory = tempfile::tempdir()?;
    let database = directory.path().join("inventory.sqlite");
    let store = collisions::create_store(&database).await?;
    insert_resources(&store, "boundary_initial", 0, 10_000).await?;
    insert_resources(&store, "boundary_extra", 10_000, 1).await?;

    let mut reader = SqliteConnection::connect_with(
        &SqliteConnectOptions::new()
            .filename(database)
            .read_only(true),
    )
    .await?;
    let ids: Vec<String> = sqlx::query_scalar("SELECT id FROM resources ORDER BY id")
        .fetch_all(&mut reader)
        .await?;
    assert_eq!(ids.len(), 10_001);
    ids.into_iter().enumerate().for_each(|(index, id)| {
        assert_eq!(id, format!("rebuild{index:05}"));
    });
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn given_exact_rebuild_page_budget_when_native_projection_runs_then_eof_completes_but_one_more_resource_stays_pending()
-> TestResult {
    let _permit = runtime::NATIVE_SLOT.acquire().await?;
    let prepared = native::PreparedGraph::install()?;
    let emulator = runtime::NativeRuntime::start()?;
    let graph = prepared.start(&emulator).await?;
    let directory = tempfile::tempdir()?;
    let database = directory.path().join("inventory.sqlite");
    let store = collisions::create_store(&database).await?;
    let expected = (0..10_000)
        .map(|index| vec![format!("rebuild{index:05}")])
        .collect::<Vec<_>>();

    insert_resources(&store, "boundary_initial", 0, 10_000).await?;
    native::project(&store, &graph).await?;
    recovery::request_rebuild(&database).await?;
    let backend = inventory_graph::falkor::FalkorDb::connect(graph.endpoint(), native::GRAPH)?;
    let exact = run_projection(&store, &backend, 1).await?;
    assert!(!exact.offline);
    assert!(
        !exact.rebuild_incomplete,
        "exactly 10000 full pages must acknowledge EOF"
    );
    assert_eq!(exact.pending, 0);
    assert_eq!(exact.acknowledged, 1);
    assert!(!store.projection_state().await?.rebuild_required);
    assert_eq!(resource_ids(&graph).await?, expected);

    insert_resources(&store, "boundary_extra", 10_000, 1).await?;
    native::project(&store, &graph).await?;
    recovery::request_rebuild(&database).await?;
    let exceeded = run_projection(&store, &backend, 1).await?;
    assert!(!exceeded.offline);
    assert!(
        exceeded.rebuild_incomplete,
        "a terminal probe must not project beyond the page budget"
    );
    assert_eq!(exceeded.pending, 1);
    assert_eq!(exceeded.acknowledged, 0);
    assert!(store.projection_state().await?.rebuild_required);
    assert_eq!(resource_ids(&graph).await?, expected);
    graph.stop().await?;
    emulator.stop()
}

async fn resource_ids(graph: &native::NativeGraph) -> TestResult<Vec<Vec<String>>> {
    graph
        .rows(inventory_graph::cypher::diagnostics::SORTED_RESOURCE_IDS)
        .await
}
