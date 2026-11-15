#[path = "seed_inventory/compute.rs"]
mod compute;
#[path = "seed_inventory/global.rs"]
mod global;
#[path = "seed_inventory/messaging.rs"]
mod messaging;
#[path = "seed_inventory/network.rs"]
mod network;
#[path = "seed_inventory/runtime.rs"]
mod runtime;
#[path = "seed_inventory/storage.rs"]
mod storage;
#[path = "native_graph_inventory/fixtures.rs"]
mod fixtures;
#[path = "native_graph_inventory/native.rs"]
mod native;
#[path = "native_graph_inventory/compare.rs"]
mod compare;
#[path = "native_graph_inventory/tags.rs"]
mod tags;
#[path = "native_graph_inventory/evidence.rs"]
mod evidence;
#[path = "native_graph_inventory/recovery.rs"]
mod recovery;
#[path = "native_graph_inventory/safe_fields.rs"]
mod safe_fields;
#[path = "native_graph_inventory/rebuild_bound.rs"]
mod rebuild_bound;
#[path = "native_graph_inventory/fault.rs"]
mod fault;
#[path = "native_graph_inventory/removal.rs"]
mod removal;
#[path = "native_graph_inventory/collisions.rs"]
mod collisions;
#[path = "native_graph_inventory/safe_shape.rs"]
mod safe_shape;
#[path = "native_graph_inventory/projection_fence.rs"]
mod projection_fence;

use inventory_engine::inventory::BinaryStore;
use runtime::NativeRuntime;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const ACCOUNT: &str = "000000000000";
const PARTITION: &str = "aws";
const REGION: &str = "us-east-1";

#[tokio::test(flavor = "current_thread")]
async fn given_shared17_and_tags_when_native_pipeline_runs_then_exact_sql_graph_and_view_sets_survive_removal_and_replay() -> TestResult {
    // Given: a fresh owned emulator and independently checked typed seed recipes.
    let _permit = runtime::NATIVE_SLOT.acquire().await?;
    let prepared_graph = native::PreparedGraph::install()?;
    let emulator = NativeRuntime::start()?;
    let (oracle, safe_records) = fixtures::seed_all(&emulator).await?;
    let mut expected = compare::Expected::from_oracles(&oracle)?;
    expected.safe_records = safe_records;
    let retired = oracle.first().ok_or("network fixture missing")?.resources.iter()
        .find(|resource| resource.kind.as_str() == "AWS::EC2::Subnet")
        .ok_or("independently seeded removable network subnet missing")?.key.clone();
    let tagged = tags::targets(&expected, &retired)?;
    tags::create(&emulator, &tagged).await?;
    let mut graph = prepared_graph.start(&emulator).await?;
    let database = emulator.work().join("native-inventory.sqlite");

    // When: the actual collector writes SQLite and the public outbox consumer
    // delivers it to the pinned native FalkorDB module (not an echoed client).
    safe_fields::assert_oracle_rejects_mutations()?;
    let started = tags::now_ms()?;
    native::collect(&emulator, &database, "initial").await?;
    let store = BinaryStore::open(&database).await?;
    assert!(store.projection_state().await?.pending > 0, "collector must publish actual resource changes");
    native::project(&store, &graph).await?;
    tags::assert_collector_coverage(&database, &tagged, started).await?;
    projection_fence::assert_overlapping_delivery(&database, &graph, tagged.first().ok_or("tag target missing")?).await?;

    // Then: all full-key resource/edge sets and scoped tag associations agree
    // with the seed oracle, including isolated resources and provenance.
    compare::all(&database, &store, &graph, &expected).await?;
    tags::assert_all(&database, &graph, &expected, &tagged, true).await?;
    tags::replay(&store, &graph, &database, &expected, &tagged, true).await?;

    // When: the real API removes one resource's complete tag snapshot and
    // replaces another's value, a second collector sweep must remove stale links.
    tags::replace_and_remove(&emulator, &tagged).await?;
    let started = tags::now_ms()?;
    native::collect(&emulator, &database, "updated").await?;
    native::project(&store, &graph).await?;
    tags::assert_collector_coverage(&database, &tagged, started).await?;
    compare::all(&database, &store, &graph, &expected).await?;
    tags::assert_all(&database, &graph, &expected, &tagged, false).await?;
    tags::replay(&store, &graph, &database, &expected, &tagged, false).await?;
    let mut latest_tags = tags::assert_tag_only_refresh(&store, &graph, &database, &expected, &tagged).await?;
    removal::retire(&emulator, &database, &store, &graph, &mut expected, &mut latest_tags, &retired).await?;
    graph.select_empty_graph("native_inventory_rebuild").await?;
    recovery::request_rebuild(&database).await?;
    assert!(store.projection_state().await?.rebuild_required);
    native::project(&store, &graph).await?;
    compare::all(&database, &store, &graph, &expected).await?;
    tags::assert_sets(&database, &graph, &expected, &latest_tags).await?;
    removal::assert_absent(&database, &graph, &retired).await?;
    evidence::retain(&database, &emulator, &expected, &graph).await?;
    collisions::assert_scoped_fixture(&mut graph, emulator.work()).await?;
    graph.stop().await?;
    emulator.stop()
}
