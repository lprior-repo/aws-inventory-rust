use super::{
    TestResult,
    compare::{self, Expected},
    fault::DeniedRoute,
    native::{self, NativeGraph},
    runtime::NativeRuntime,
    tags::{self, TagSets},
};
use cmdb_view::{Database, GraphConnection, GraphLimits};
use inventory_core::{inventory::ResourceKey, record::Value};
use inventory_engine::inventory::{BinaryStore, StoreError};
use inventory_harness::inventory_seed::{
    FieldPath, SeedInput, SeedInputValue, SeedScenario, SeedStep, StepId,
};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::path::Path;

pub(super) async fn retire(
    native: &NativeRuntime,
    database: &Path,
    store: &BinaryStore,
    graph: &NativeGraph,
    expected: &mut Expected,
    tags: &mut TagSets,
    target: &ResourceKey,
) -> TestResult {
    // Given a genuinely tagged removable subnet; this is not DeleteTags.
    let before = Database::open(database.to_str().ok_or("non-UTF-8 database path")?)
        .await?
        .detail(&compare::node(target))
        .await?
        .ok_or("deletion target missing")?;
    assert!(!before.tags.is_empty());
    let memberships = misses(database, target).await?;
    assert!(!memberships.is_empty() && memberships.iter().all(|(_, missing)| *missing == 0));
    delete_subnet(native, target).await?;

    // One distinct complete miss retains the resource and every incident edge.
    native::collect(native, database, "miss-one").await?;
    native::project(store, graph).await?;
    let once = misses(database, target).await?;
    assert_eq!(
        once,
        memberships
            .iter()
            .map(|(scope, _)| (scope.clone(), 1))
            .collect::<Vec<_>>()
    );
    compare::all(database, store, graph, expected).await?;
    tags::assert_sets(database, graph, expected, tags).await?;

    // A real CLI request is denied for DescribeSubnets only. Bootstrap and all
    // other operations still reach the owned emulator; no shared service stops.
    denied_miss(native, database, graph, store, target, &once).await?;
    compare::all(database, store, graph, expected).await?;
    tags::assert_sets(database, graph, expected, tags).await?;

    // The second distinct complete miss retires the full key, associations and
    // incident dependencies. The independent oracle changes at this boundary.
    native::collect(native, database, "miss-two").await?;
    assert!(expected.resources.remove(target));
    assert!(expected.safe_records.remove(target).is_some());
    let fields = compare::key_fields(target);
    expected.edges.retain(|edge| {
        edge.get(..4) != Some(fields.as_slice()) && edge.get(4..8) != Some(fields.as_slice())
    });
    assert!(tags.remove(target).is_some());
    native::project(store, graph).await?;
    compare::all(database, store, graph, expected).await?;
    tags::assert_sets(database, graph, expected, tags).await?;
    assert_absent(database, graph, target).await?;
    reject_late_snapshot(store, database, graph, target, expected, tags).await
}

async fn delete_subnet(native: &NativeRuntime, target: &ResourceKey) -> TestResult {
    assert_eq!(target.kind.as_str(), "AWS::EC2::Subnet");
    let step = SeedStep::auxiliary(
        StepId::new(0),
        "ec2".into(),
        "DeleteSubnet".into(),
        vec![SeedInput::new(
            FieldPath::new(&["SubnetId"])?,
            SeedInputValue::Literal(Value::Text(target.id.as_str().into())),
        )],
    );
    let oracle = native.seed(SeedScenario::new(vec![step])?).await?;
    assert!(oracle.resources.is_empty() && oracle.edges.is_empty());
    Ok(())
}

async fn denied_miss(
    native: &NativeRuntime,
    database: &Path,
    graph: &NativeGraph,
    store: &BinaryStore,
    target: &ResourceKey,
    expected_misses: &[(Vec<u8>, i64)],
) -> TestResult {
    let observation = Database::open(database.to_str().ok_or("non-UTF-8 database path")?)
        .await?
        .detail(&compare::node(target))
        .await?
        .ok_or("denied target missing")?
        .tag_observation;
    let started = tags::now_ms()?;
    let route = DeniedRoute::start(native).await?;
    let result = native::collect_at(
        native,
        database,
        "denied-between-misses",
        route.endpoint(),
        route.ca(),
    )
    .await;
    let drained = route.close().await;
    let output = result?;
    drained?;
    assert!(
        output.status.success(),
        "scope denial is a durable degraded sweep, not bootstrap failure: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new()
                .filename(database)
                .read_only(true),
        )
        .await?;
    for (scope, _) in expected_misses {
        let outcome: String = sqlx::query_scalar("SELECT outcome FROM scope_outcomes WHERE scope_key=?1 AND observed_at>=?2 ORDER BY observed_at DESC LIMIT 1")
            .bind(scope).bind(started).fetch_one(&pool).await?;
        assert_eq!(
            outcome, "inaccessible",
            "actual denied operation must persist its landed scope outcome"
        );
    }
    pool.close().await;
    assert_eq!(
        misses(database, target).await?,
        expected_misses,
        "denied scope cannot advance a complete-miss counter"
    );
    assert_eq!(
        Database::open(database.to_str().ok_or("non-UTF-8 database path")?)
            .await?
            .detail(&compare::node(target))
            .await?
            .ok_or("denied sweep deleted resource")?
            .tag_observation,
        observation,
        "whole-operation denial must retain existing per-resource evidence, not invent Complete-empty or Failed"
    );
    native::project(store, graph).await
}

async fn misses(database: &Path, target: &ResourceKey) -> TestResult<Vec<(Vec<u8>, i64)>> {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new()
                .filename(database)
                .read_only(true),
        )
        .await?;
    let values = sqlx::query_as("SELECT scope_key,missing_complete FROM resource_scopes WHERE account=?1 AND region=?2 AND kind=?3 AND id=?4 ORDER BY scope_key")
        .bind(target.account.as_str()).bind(target.region.as_str()).bind(target.kind.as_str()).bind(target.id.as_str()).fetch_all(&pool).await?;
    pool.close().await;
    Ok(values)
}

pub(super) async fn assert_absent(
    database: &Path,
    graph: &NativeGraph,
    target: &ResourceKey,
) -> TestResult {
    let node = compare::node(target);
    assert!(
        Database::open(database.to_str().ok_or("non-UTF-8 database path")?)
            .await?
            .detail(&node)
            .await?
            .is_none(),
        "deleted public detail must not resurrect"
    );
    let reader = GraphConnection::connect(graph.endpoint(), graph.name())?;
    let neighbors = reader
        .neighbors(
            &node,
            GraphLimits {
                depth: 3,
                nodes: 500,
                edges: 2000,
            },
        )
        .await?;
    assert!(!neighbors.truncated && neighbors.edges.is_empty());
    let tags = reader.tag_affiliations(&node, 1000).await?;
    assert!(!tags.truncated && tags.affiliations.is_empty());
    Ok(())
}

async fn reject_late_snapshot(
    store: &BinaryStore,
    database: &Path,
    graph: &NativeGraph,
    target: &ResourceKey,
    expected: &Expected,
    tags: &TagSets,
) -> TestResult {
    let result = tags::tag_commit::replace(
        store,
        target,
        vec![(tags::TAG_KEY, "late-must-not-resurrect")],
        "native-late-deleted-tags",
        tags::now_ms()?,
    )
    .await;
    let error = result
        .err()
        .ok_or("late tag snapshot unexpectedly resurrected a deleted resource")?;
    assert!(
        matches!(
            error.downcast_ref::<StoreError>(),
            Some(StoreError::Invalid(
                "tag observation has no persisted resource"
            ))
        ),
        "unexpected late snapshot error: {error}"
    );
    native::project(store, graph).await?;
    compare::all(database, store, graph, expected).await?;
    tags::assert_sets(database, graph, expected, tags).await?;
    assert_absent(database, graph, target).await
}
