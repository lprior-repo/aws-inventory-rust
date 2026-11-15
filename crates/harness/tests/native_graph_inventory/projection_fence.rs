#[path = "projection_fence/proxy.rs"]
mod proxy;
#[path = "tag_commit.rs"]
mod tag_commit;

use super::{TestResult, compare, native::NativeGraph, recovery};
use cmdb_view::GraphConnection;
use inventory_core::inventory::{OperationId, ResourceKey, ScopeKey, ScopeOutcome, ServiceId};
use inventory_engine::inventory::{BinaryStore, ProjectionError, ProjectionReport, SweepId, WorkSpec, run_projection};
use sqlx::{Connection, Row, SqliteConnection, sqlite::SqliteConnectOptions};
use std::{collections::BTreeSet, path::Path, time::Duration};

const OWNER: &str = "projection-fence-owner";

pub(super) async fn assert_overlapping_delivery(database: &Path, native: &NativeGraph, target: &ResourceKey) -> TestResult {
    tokio::time::timeout(Duration::from_secs(120), async {
        tag_replacement(database, native, target).await?;
        cancelled_tag_replacement(database, native, target).await?;
        cancelled_claim(database, native, target).await?;
        cancelled_resurrection(database, native, target).await?;
        stale_rebuild(database, native).await
    }).await?
}

async fn tag_replacement(database: &Path, native: &NativeGraph, target: &ResourceKey) -> TestResult {
    // Given: P1 retains alice and pauses a real native tag transaction.
    let (_directory, path, store) = isolated(database, native, "native_fence_tags").await?;
    let second = BinaryStore::open(&path).await?;
    tag_commit::replace(&store, target, vec![(OWNER, "alice")], "fence-alice", 4_000_000_000_000).await?;
    let old = store.drain_projection(1000).await?;
    let mut proxy = proxy::Proxy::start(native.endpoint(), "MERGE (t:Tag").await.map_err(|error| error.to_string())?;
    let endpoint = proxy.endpoint.clone();
    let mut first = Box::pin(run_projection(&store, &endpoint, "native_fence_tags", 1000));
    tokio::select! { result = &mut first => return Err(format!("P1 finished before barrier: {result:?}").into()), paused = proxy.paused() => paused.map_err(|error| error.to_string())? }
    // The same handle's single reader is held by P1's snapshot. Admission must
    // check the independent cached sidecar path, not wait for that reader.
    let same_handle = tokio::time::timeout(Duration::from_secs(2),
        run_projection(&store, native.endpoint(), "native_fence_tags", 1000)).await?;
    assert!(matches!(same_handle, Err(ProjectionError::DeliveryBusy)));
    // When: a separate writer commits bob, P2 cannot overtake P1.
    tag_commit::replace(&second, target, vec![(OWNER, "bob")], "fence-bob", 4_000_000_000_001).await?;
    let latest = second.drain_projection(1000).await?;
    assert_eq!(latest.len(), 1);
    assert!(latest[0].revision > old[0].revision);
    assert!(matches!(run_projection(&second, native.endpoint(), "native_fence_tags", 1000).await, Err(ProjectionError::DeliveryBusy)));
    let (_, running) = proxy.resume().await.map_err(|error| error.to_string())?;
    let delivered = first.await?;
    running.close().await.map_err(|error| error.to_string())?;
    // Then: R removed no row, S remains exact, and final graph contains only bob.
    assert_eq!(delivered.acknowledged, 0);
    assert_eq!(delivered.pending, 1);
    assert_eq!(second.drain_projection(1000).await?, latest);
    let delivered = project(&second, native, "native_fence_tags").await?;
    assert_eq!(delivered.acknowledged, 1);
    assert_eq!(delivered.pending, 0);
    assert_target(native, "native_fence_tags", target, "bob").await?;
    assert_sets(&path, &second, native, "native_fence_tags").await
}

async fn cancelled_claim(database: &Path, native: &NativeGraph, target: &ResourceKey) -> TestResult {
    // Given: A's atomic compare/SET claim has not yet reached native Redis.
    let (_directory, path, store) = isolated(database, native, "native_fence_claim").await?;
    tag_commit::replace(&store, target, vec![(OWNER, "alice")], "fence-claim-alice", 4_000_000_000_000).await?;
    let mut old_proxy = proxy::Proxy::start_claim(native.endpoint()).await.map_err(|error| error.to_string())?;
    let old_endpoint = old_proxy.endpoint.clone();
    let mut old = Box::pin(run_projection(&store, &old_endpoint, "native_fence_claim", 1000));
    tokio::select! { result = &mut old => return Err(format!("A finished before claim barrier: {result:?}").into()), paused = old_proxy.paused() => paused.map_err(|error| error.to_string())? }
    drop(old);
    wait_for_release(&path).await?;
    // When: successor B owns delivery and pauses inside its real tag transaction.
    let second = BinaryStore::open(&path).await?;
    tag_commit::replace(&second, target, vec![(OWNER, "bob")], "fence-claim-bob", 4_000_000_000_001).await?;
    let mut successor_proxy = proxy::Proxy::start(native.endpoint(), "MERGE (t:Tag").await.map_err(|error| error.to_string())?;
    let successor_endpoint = successor_proxy.endpoint.clone();
    let mut successor = Box::pin(run_projection(&second, &successor_endpoint, "native_fence_claim", 1000));
    tokio::select! { result = &mut successor => return Err(format!("B finished before tag barrier: {result:?}").into()), paused = successor_proxy.paused() => paused.map_err(|error| error.to_string())? }
    let (response, old_running) = old_proxy.resume().await.map_err(|error| error.to_string())?;
    assert_eq!(response, b":0\r\n", "native compare/SET must reject A's delayed older generation without changing B's owner key");
    old_running.close().await.map_err(|error| error.to_string())?;
    // Then: B's already-watched EXEC still succeeds: a stale claim cannot abort it.
    let (_, running) = successor_proxy.resume().await.map_err(|error| error.to_string())?;
    let delivered = successor.await?;
    running.close().await.map_err(|error| error.to_string())?;
    assert_eq!(delivered.acknowledged, 1);
    assert_eq!(delivered.pending, 0);
    assert_target(native, "native_fence_claim", target, "bob").await?;
    assert_sets(&path, &second, native, "native_fence_claim").await
}

async fn cancelled_tag_replacement(database: &Path, native: &NativeGraph, target: &ResourceKey) -> TestResult {
    // Given: alice's actual tag command is queued in MULTI, with EXEC delayed.
    let (_directory, path, store) = isolated(database, native, "native_fence_cancel_tags").await?;
    tag_commit::replace(&store, target, vec![(OWNER, "alice")], "fence-cancel-alice", 4_000_000_000_000).await?;
    let old = store.drain_projection(1000).await?;
    let mut proxy = proxy::Proxy::start(native.endpoint(), "MERGE (t:Tag").await.map_err(|error| error.to_string())?;
    let endpoint = proxy.endpoint.clone();
    let mut first = Box::pin(run_projection(&store, &endpoint, "native_fence_cancel_tags", 1000));
    tokio::select! { result = &mut first => return Err(format!("P1 finished before tag barrier: {result:?}").into()), paused = proxy.paused() => paused.map_err(|error| error.to_string())? }
    // When: cancelling P1 releases local ownership; P2 delivers bob first.
    drop(first);
    wait_for_release(&path).await?;
    let second = BinaryStore::open(&path).await?;
    tag_commit::replace(&second, target, vec![(OWNER, "bob")], "fence-cancel-bob", 4_000_000_000_001).await?;
    let latest = second.drain_projection(1000).await?;
    assert_eq!(old.len(), 1);
    assert_eq!(latest.len(), 1);
    assert!(latest[0].revision > old[0].revision);
    let delivered = project(&second, native, "native_fence_cancel_tags").await?;
    assert_eq!(delivered.acknowledged, 1);
    assert_eq!(delivered.pending, 0);
    assert_target(native, "native_fence_cancel_tags", target, "bob").await?;
    // Then: real Redis rejects alice's late tag EXEC, not merely node upserts.
    let (response, running) = proxy.resume().await.map_err(|error| error.to_string())?;
    assert_eq!(response, b"*-1\r\n", "WATCH must reject a cancelled owner's late native tag transaction");
    running.close().await.map_err(|error| error.to_string())?;
    assert_eq!(second.projection_state().await?.pending, 0);
    assert_target(native, "native_fence_cancel_tags", target, "bob").await?;
    assert_sets(&path, &second, native, "native_fence_cancel_tags").await
}

async fn cancelled_resurrection(database: &Path, native: &NativeGraph, target: &ResourceKey) -> TestResult {
    // Given: a native upsert is queued in MULTI, but its EXEC is still in transit.
    let (_directory, path, store) = isolated(database, native, "native_fence_delete").await?;
    tag_commit::replace(&store, target, vec![(OWNER, "alice")], "fence-delete-alice", 4_000_000_000_000).await?;
    let mut proxy = proxy::Proxy::start(native.endpoint(), "MERGE (n:Resource").await.map_err(|error| error.to_string())?;
    let endpoint = proxy.endpoint.clone();
    let mut first = Box::pin(run_projection(&store, &endpoint, "native_fence_delete", 1000));
    tokio::select! { result = &mut first => return Err(format!("P1 finished before barrier: {result:?}").into()), paused = proxy.paused() => paused.map_err(|error| error.to_string())? }
    // When: public complete sweeps delete while the old native request is in flight.
    delete_after_two_misses(&store, target).await?;
    drop(first);
    wait_for_release(&path).await?;
    let second = BinaryStore::open(&path).await?;
    let pending = second.drain_projection(1000).await?;
    assert!(pending.iter().any(|entry| entry.deleted && entry.key.as_ref() == Some(target)));
    let delivered = project(&second, native, "native_fence_delete").await?;
    assert_eq!(delivered.acknowledged, u64::try_from(pending.len())?);
    assert_eq!(delivered.pending, 0);
    // Then: a late old EXEC is rejected by real FalkorDB/Redis, not the proxy.
    let (response, running) = proxy.resume().await.map_err(|error| error.to_string())?;
    assert_eq!(response, b"*-1\r\n", "WATCH must reject the old native transaction after successor delivery");
    running.close().await.map_err(|error| error.to_string())?;
    assert_eq!(second.projection_state().await?.pending, 0);
    assert!(second.projection_resource(target).await?.is_none());
    assert_sets(&path, &second, native, "native_fence_delete").await
}

async fn stale_rebuild(database: &Path, native: &NativeGraph) -> TestResult {
    // Given: rebuild and incremental delivery use the same durable owner.
    let (_directory, path, store) = isolated(database, native, "native_fence_rebuild").await?;
    recovery::request_rebuild(&path).await?;
    let mut proxy = proxy::Proxy::start(native.endpoint(), "MERGE (n:Resource").await.map_err(|error| error.to_string())?;
    let endpoint = proxy.endpoint.clone();
    let mut first = Box::pin(run_projection(&store, &endpoint, "native_fence_rebuild", 1000));
    tokio::select! { result = &mut first => return Err(format!("rebuild finished before barrier: {result:?}").into()), paused = proxy.paused() => paused.map_err(|error| error.to_string())? }
    // When: only the persisted rebuild control revision advances mid-snapshot.
    // No public request-rebuild API exists; this is transparent control injection.
    advance_marker(&path).await?;
    let second = BinaryStore::open(&path).await?;
    let latest = second.drain_projection(1000).await?;
    assert!(matches!(run_projection(&second, native.endpoint(), "native_fence_rebuild", 1000).await, Err(ProjectionError::DeliveryBusy)));
    let (_, running) = proxy.resume().await.map_err(|error| error.to_string())?;
    let delivered = first.await?;
    running.close().await.map_err(|error| error.to_string())?;
    // Then: stale marker attempts count zero and the next rebuild counts one.
    assert_eq!(delivered.acknowledged, 0);
    assert_eq!(delivered.pending, 1);
    assert_eq!(second.drain_projection(1000).await?, latest);
    let delivered = project(&second, native, "native_fence_rebuild").await?;
    assert_eq!(delivered.acknowledged, 1);
    assert_eq!(delivered.pending, 0);
    assert_sets(&path, &second, native, "native_fence_rebuild").await
}

async fn isolated(database: &Path, native: &NativeGraph, name: &str) -> TestResult<(tempfile::TempDir, std::path::PathBuf, BinaryStore)> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("projection.sqlite");
    let mut connection = connect(database).await?;
    sqlx::query("VACUUM INTO ?1").bind(path.to_str().ok_or("temporary database path is not UTF-8")?).execute(&mut connection).await?;
    connection.close().await?;
    recovery::request_rebuild(&path).await?;
    let store = BinaryStore::open(&path).await?;
    let delivered = project(&store, native, name).await?;
    assert_eq!(delivered.pending, 0);
    assert_eq!(delivered.acknowledged, 1, "real native GRAPH.QUERY in MULTI must execute before acknowledgement");
    Ok((directory, path, store))
}

async fn project(store: &BinaryStore, native: &NativeGraph, name: &str) -> TestResult<ProjectionReport> {
    let report = run_projection(store, native.endpoint(), name, 1000).await?;
    assert!(!report.offline && !report.rebuild_incomplete, "native fence delivery incomplete: {report:?}");
    Ok(report)
}

async fn connect(path: &Path) -> TestResult<SqliteConnection> {
    Ok(SqliteConnection::connect_with(&SqliteConnectOptions::new().filename(path).busy_timeout(Duration::from_secs(5))).await?)
}

async fn wait_for_release(path: &Path) -> TestResult {
    let mut name = path.as_os_str().to_os_string();
    name.push(".projection-lock.sqlite");
    let mut connection = connect(Path::new(&name)).await?;
    sqlx::query("BEGIN IMMEDIATE").execute(&mut connection).await?;
    connection.close().await?;
    Ok(())
}

async fn advance_marker(path: &Path) -> TestResult {
    let mut connection = connect(path).await?;
    let mut transaction = connection.begin().await?;
    let revision: i64 = sqlx::query_scalar("UPDATE metadata SET value=CAST(value AS INTEGER)+1 WHERE key='projection_revision' RETURNING value").fetch_one(&mut *transaction).await?;
    let changed = sqlx::query("UPDATE projection_outbox SET revision=?1 WHERE full_rebuild=1").bind(revision).execute(&mut *transaction).await?;
    assert_eq!(changed.rows_affected(), 1);
    transaction.commit().await?;
    connection.close().await?;
    Ok(())
}

async fn delete_after_two_misses(store: &BinaryStore, target: &ResourceKey) -> TestResult {
    let scope = ScopeKey::new(target.account.clone(), target.region.clone(), ServiceId::parse("ec2")?, OperationId::parse("DescribeVpcs")?, None)?;
    for sweep in ["fence-missing-one", "fence-missing-two"] {
        let sweep = SweepId::new(sweep.into())?;
        let bindings = inventory_core::record::Record::encode(&inventory_core::record::Value::Map(Default::default()))?;
        store.seed_work(WorkSpec::new(sweep.clone(), scope.clone(), None, bindings.as_bytes().to_vec())?).await?;
        let lease = store.claim(&sweep, 10, 100).await?.ok_or("missing deletion lease")?;
        store.finish_scope(lease, ScopeOutcome::Complete, 11).await?;
        assert_eq!(store.projection_resource(target).await?.is_some(), sweep.as_str() == "fence-missing-one");
    }
    Ok(())
}

async fn assert_target(native: &NativeGraph, name: &str, target: &ResourceKey, owner: &str) -> TestResult {
    let reader = GraphConnection::connect(native.endpoint(), name)?;
    let affiliations = reader.tag_affiliations(&compare::node(target), 1000).await?;
    assert!(!affiliations.truncated);
    assert_eq!(affiliations.affiliations.len(), 1);
    let affiliation = &affiliations.affiliations[0];
    assert_eq!(affiliation.provenance, "inventory.tags");
    assert_eq!(affiliation.tag.account, target.account.as_str());
    assert_eq!(affiliation.tag.region, target.region.as_str());
    assert_eq!(affiliation.tag.key, OWNER);
    assert_eq!(affiliation.tag.value, owner);
    Ok(())
}

async fn assert_sets(path: &Path, store: &BinaryStore, native: &NativeGraph, name: &str) -> TestResult {
    let resources = store.projection_resources(None, 1000).await?;
    assert!(resources.len() < 1000);
    let expected: BTreeSet<_> = resources.iter().map(|resource| compare::key_fields(&resource.key)).collect();
    let actual = rows(native, name, "MATCH (r:Resource) RETURN r.account,r.region,r.kind,r.id").await?;
    assert_eq!(actual.len(), expected.len(), "late stale upserts must not resurrect resources");
    assert_eq!(actual.into_iter().collect::<BTreeSet<_>>(), expected);
    let mut connection = connect(path).await?;
    let tags = sqlx::query("SELECT account,region,kind,id,tag_key,tag_value FROM tags").fetch_all(&mut connection).await?;
    let expected = tags.into_iter().map(|row| Ok(vec![row.try_get::<String,_>(0)?,row.try_get(1)?,row.try_get(2)?,row.try_get(3)?,row.try_get(4)?,row.try_get(5)?,"inventory.tags".into()])).collect::<Result<BTreeSet<_>, sqlx::Error>>()?;
    let vertices: BTreeSet<_> = expected.iter().map(|row| vec![row[0].clone(), row[1].clone(), row[4].clone(), row[5].clone()]).collect();
    let actual = rows(native, name, "MATCH (r:Resource)-[a:HAS_TAG]->(t:Tag) RETURN r.account,r.region,r.kind,r.id,t.tag_key,t.tag_value,a.provenance").await?;
    assert_eq!(actual.len(), expected.len());
    assert_eq!(actual.into_iter().collect::<BTreeSet<_>>(), expected);
    let actual = rows(native, name, "MATCH (t:Tag) RETURN t.account,t.region,t.tag_key,t.tag_value").await?;
    assert_eq!(actual.len(), vertices.len(), "no orphan or duplicate native tag vertices");
    assert_eq!(actual.into_iter().collect::<BTreeSet<_>>(), vertices);
    let dependencies = store.projection_edges(None, 1000).await?;
    assert_eq!(dependencies.next, None);
    let expected_edges: BTreeSet<_> = dependencies.edges.iter().map(|edge| {
        compare::key_fields(&edge.from).into_iter().chain(compare::key_fields(&edge.to))
            .chain([edge.relation.as_str().to_owned(), edge.source.as_str().to_owned()]).collect::<Vec<_>>()
    }).collect();
    assert_eq!(expected_edges.len(), dependencies.edges.len());
    let actual = rows(native, name, "MATCH (s:Resource)-[r]->(t:Resource) RETURN s.account,s.region,s.kind,s.id,t.account,t.region,t.kind,t.id,r.relation,r.provenance").await?;
    assert_eq!(actual.len(), expected_edges.len());
    assert_eq!(actual.into_iter().collect::<BTreeSet<_>>(), expected_edges);
    let nodes = rows(native, name, "MATCH (n) RETURN labels(n)[0]").await?;
    assert_eq!(nodes.len(), resources.len() + vertices.len());
    assert!(nodes.iter().all(|row| matches!(row.as_slice(), [label] if label == "Resource" || label == "Tag")));
    let relationships = rows(native, name, "MATCH ()-[r]->() RETURN type(r)").await?;
    assert_eq!(relationships.len(), expected.len() + dependencies.edges.len());
    connection.close().await?;
    Ok(())
}

async fn rows(native: &NativeGraph, name: &str, query: &str) -> TestResult<Vec<Vec<String>>> {
    let client = redis::Client::open(native.endpoint())?;
    let mut connection = client.get_multiplexed_async_connection().await?;
    let value: redis::Value = redis::cmd("GRAPH.RO_QUERY").arg(name).arg(format!("{query} LIMIT 1001")).query_async(&mut connection).await?;
    let redis::Value::Array(parts) = value else { return Err("native response is not an array".into()); };
    let Some(redis::Value::Array(rows)) = parts.get(1) else { return Err("native rows missing".into()); };
    assert!(rows.len() <= 1000);
    rows.iter().map(|row| {
        let redis::Value::Array(fields) = row else { return Err("native row is not an array".into()); };
        fields.iter().map(|field| match field {
            redis::Value::BulkString(bytes) => Ok(String::from_utf8(bytes.clone())?),
            redis::Value::SimpleString(value) => Ok(value.clone()),
            _ => Err(format!("native field is not text: {field:?}").into()),
        }).collect()
    }).collect()
}
