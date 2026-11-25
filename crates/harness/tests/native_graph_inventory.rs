#[path = "native_graph_inventory/collisions.rs"]
mod collisions;
#[path = "native_graph_inventory/compare.rs"]
mod compare;
#[path = "seed_inventory/compute.rs"]
mod compute;
#[path = "native_graph_inventory/evidence.rs"]
mod evidence;
#[path = "native_graph_inventory/fault.rs"]
mod fault;
#[path = "native_graph_inventory/fixtures.rs"]
mod fixtures;
#[path = "seed_inventory/global.rs"]
mod global;
#[path = "native_graph_inventory/listed_resources.rs"]
mod listed_resources;
#[path = "seed_inventory/messaging.rs"]
mod messaging;
#[path = "native_graph_inventory/native.rs"]
mod native;
#[path = "seed_inventory/network.rs"]
mod network;
#[path = "native_graph_inventory/projection_fence.rs"]
mod projection_fence;
#[path = "native_graph_inventory/rebuild_bound.rs"]
mod rebuild_bound;
#[path = "native_graph_inventory/recovery.rs"]
mod recovery;
#[path = "native_graph_inventory/removal.rs"]
mod removal;
#[path = "seed_inventory/runtime.rs"]
mod runtime;
#[path = "native_graph_inventory/safe_fields.rs"]
mod safe_fields;
#[path = "native_graph_inventory/safe_shape.rs"]
mod safe_shape;
#[path = "seed_inventory/storage.rs"]
mod storage;
#[path = "native_graph_inventory/tags.rs"]
mod tags;

use inventory_core::inventory::ResourceKey;
use inventory_engine::inventory::BinaryStore;
use native::{FixtureCollector, NativeGraph};
use runtime::NativeRuntime;
use std::path::Path;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const ACCOUNT: &str = "000000000000";
const PARTITION: &str = "aws";
const REGION: &str = "us-east-1";

#[derive(Debug, thiserror::Error)]
#[error("{phase}: {source}")]
struct PhaseError {
    phase: &'static str,
    source: Box<dyn std::error::Error>,
}

trait InPhase<T> {
    fn in_phase(self, phase: &'static str) -> TestResult<T>;
}

impl<T, E> InPhase<T> for Result<T, E>
where
    Box<dyn std::error::Error>: From<E>,
{
    fn in_phase(self, phase: &'static str) -> TestResult<T> {
        self.map_err(|source| {
            PhaseError {
                phase,
                source: source.into(),
            }
            .into()
        })
    }
}

#[tokio::test(flavor = "current_thread")]
async fn given_shared17_and_tags_when_native_pipeline_runs_then_exact_sql_graph_and_view_sets_survive_removal_and_replay()
-> TestResult {
    let _permit = runtime::NATIVE_SLOT
        .acquire()
        .await
        .in_phase("native slot")?;
    let prepared_graph = native::PreparedGraph::install().in_phase("install graph")?;
    let emulator = NativeRuntime::start().in_phase("start emulator")?;
    let scenarios = fixtures::scenarios().in_phase("build seed scenarios")?;
    let (oracle, safe_records) = fixtures::seed_all(&emulator, &scenarios)
        .await
        .in_phase("seed fixtures")?;
    let mut expected = compare::Expected::from_oracles(&oracle).in_phase("build seed oracle")?;
    expected.safe_records = safe_records;
    let retired = oracle
        .first()
        .ok_or("network fixture missing")
        .in_phase("select removal target")?
        .resources
        .iter()
        .find(|resource| resource.kind.as_str() == "AWS::EC2::Subnet")
        .ok_or("independently seeded removable network subnet missing")
        .in_phase("select removal target")?
        .key
        .clone();
    let tagged = tags::targets(&expected, &retired).in_phase("select tag targets")?;
    tags::create(&emulator, &tagged)
        .await
        .in_phase("create tags")?;
    expected
        .refresh_listed(&emulator)
        .await
        .in_phase("read seeded resource listings")?;
    let mut graph = prepared_graph
        .start(&emulator)
        .await
        .in_phase("start graph")?;
    let database = emulator.work().join("native-inventory.sqlite");
    let collector = FixtureCollector::new(&emulator, &database, &scenarios, &tagged)
        .in_phase("fixture scopes")?;

    assert_sweeps_and_rebuild(&collector, &mut graph, &mut expected, &tagged, &retired).await?;
    evidence::retain(&database, &emulator, &expected, &graph)
        .await
        .in_phase("retain evidence")?;
    collisions::assert_scoped_fixture(&mut graph, emulator.work())
        .await
        .in_phase("assert identity collisions")?;
    graph.stop().await.in_phase("stop graph")?;
    emulator.stop().in_phase("stop emulator")
}

async fn assert_sweeps_and_rebuild(
    collector: &FixtureCollector<'_>,
    graph: &mut NativeGraph,
    expected: &mut compare::Expected,
    tagged: &[ResourceKey],
    retired: &ResourceKey,
) -> TestResult {
    let store = assert_initial_sweep(collector, graph, expected, tagged)
        .await
        .in_phase("initial sweep")?;
    let mut latest_tags = assert_updated_sweep(collector, &store, graph, expected, tagged)
        .await
        .in_phase("updated sweep")?;
    removal::retire(
        collector,
        &store,
        graph,
        expected,
        &mut latest_tags,
        retired,
    )
    .await
    .in_phase("remove subnet across sweeps")?;
    assert_rebuild(
        collector.database,
        &store,
        graph,
        expected,
        &latest_tags,
        retired,
    )
    .await
    .in_phase("rebuild projection")
}

async fn assert_rebuild(
    database: &Path,
    store: &BinaryStore,
    graph: &mut NativeGraph,
    expected: &compare::Expected,
    tags: &tags::TagSets,
    retired: &ResourceKey,
) -> TestResult {
    graph
        .select_empty_graph("native_inventory_rebuild")
        .await
        .in_phase("select empty graph")?;
    recovery::request_rebuild(database)
        .await
        .in_phase("request rebuild")?;
    let state = store
        .projection_state()
        .await
        .in_phase("read rebuild state")?;
    assert!(
        state.rebuild_required,
        "request must mark the rebuild required"
    );
    native::project(store, graph)
        .await
        .in_phase("deliver rebuild")?;
    compare::all(database, store, graph, expected)
        .await
        .in_phase("compare rebuilt resource and edge sets")?;
    tags::assert_sets(database, graph, expected, tags)
        .await
        .in_phase("compare rebuilt tag sets")?;
    removal::assert_absent(database, graph, retired)
        .await
        .in_phase("check removed subnet after rebuild")
}

async fn assert_initial_sweep(
    collector: &FixtureCollector<'_>,
    graph: &NativeGraph,
    expected: &compare::Expected,
    tagged: &[ResourceKey],
) -> TestResult<BinaryStore> {
    safe_fields::assert_oracle_rejects_mutations(&expected.safe_records)
        .in_phase("check safe-record oracle mutations")?;
    let database = collector.database;
    let started = tags::now_ms().in_phase("record sweep start time")?;
    collector
        .collect("initial")
        .await
        .in_phase("collect inventory")?;
    let store = BinaryStore::open(database)
        .await
        .in_phase("open inventory store")?;
    let state = store
        .projection_state()
        .await
        .in_phase("read projection state")?;
    assert!(
        state.pending > 0,
        "collector must publish actual resource changes"
    );
    native::project(&store, graph)
        .await
        .in_phase("project inventory")?;
    tags::assert_collector_coverage(database, tagged, started)
        .await
        .in_phase("check collector tag coverage")?;
    let target = tagged
        .first()
        .ok_or("tag target missing")
        .in_phase("select fence target")?;
    projection_fence::assert_overlapping_delivery(database, graph, target)
        .await
        .in_phase("check projection delivery fences")?;
    compare::all(database, &store, graph, expected)
        .await
        .in_phase("compare resource and edge sets")?;
    tags::assert_all(database, graph, expected, tagged, true)
        .await
        .in_phase("compare tag sets")?;
    tags::replay(&store, graph, database, expected, tagged, true)
        .await
        .in_phase("replay tags")?;
    Ok(store)
}

async fn assert_updated_sweep(
    collector: &FixtureCollector<'_>,
    store: &BinaryStore,
    graph: &NativeGraph,
    expected: &mut compare::Expected,
    tagged: &[ResourceKey],
) -> TestResult<tags::TagSets> {
    tags::replace_and_remove(collector.native, tagged)
        .await
        .in_phase("replace and remove tags")?;
    expected
        .refresh_listed(collector.native)
        .await
        .in_phase("read updated resource listings")?;
    let database = collector.database;
    let started = tags::now_ms().in_phase("record sweep start time")?;
    collector
        .collect("updated")
        .await
        .in_phase("collect inventory")?;
    native::project(store, graph)
        .await
        .in_phase("project inventory")?;
    tags::assert_collector_coverage(database, tagged, started)
        .await
        .in_phase("check collector tag coverage")?;
    compare::all(database, store, graph, expected)
        .await
        .in_phase("compare resource and edge sets")?;
    tags::assert_all(database, graph, expected, tagged, false)
        .await
        .in_phase("compare tag sets")?;
    tags::replay(store, graph, database, expected, tagged, false)
        .await
        .in_phase("replay tags")?;
    tags::assert_tag_only_refresh(store, graph, database, expected, tagged)
        .await
        .in_phase("check tag-only refresh")
}
