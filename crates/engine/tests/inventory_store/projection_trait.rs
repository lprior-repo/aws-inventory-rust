#[path = "projection_trait/backend.rs"]
mod backend;
#[path = "projection_trait/fault.rs"]
mod fault;
#[path = "projection_trait/high_degree.rs"]
mod high_degree;
#[path = "projection_trait/paging.rs"]
mod paging;

#[path = "../projection_degree/support.rs"]
mod edge_fixture;

use super::support::{TestResult, fixture, observation, page, work};
use backend::{Call, RecordingBackend};
use inventory_engine::inventory::{
    BinaryStore, InventoryStore, ProjectionReport, SweepId, run_projection,
};
use inventory_graph::NodeRow;

async fn pending_resource() -> TestResult<(tempfile::TempDir, BinaryStore, NodeRow)> {
    let (directory, store, _) = fixture().await?;
    let observation = observation("one", "value")?;
    let expected = NodeRow {
        key: observation.key().clone(),
        name: None,
        content_hash: *observation.content_hash().as_bytes(),
    };
    store
        .seed_work(work("trait_contract", "us-east-1")?)
        .await?;
    let lease = store
        .claim(&SweepId::new("trait_contract".into())?, 1, 100)
        .await?
        .ok_or("missing lease")?;
    store
        .commit_page(page(lease, vec![observation], true, None, 2))
        .await?;
    assert_eq!(store.projection_state().await?.pending, 1);
    Ok((directory, store, expected))
}

#[tokio::test]
async fn given_pending_resource_when_projected_then_one_upsert_and_incident_replacement_are_acknowledged()
-> TestResult {
    // Given: a committed resource and its real durable outbox revision.
    let (directory, store, expected) = pending_resource().await?;
    let mut edges =
        edge_fixture::seed_edges(&directory.path().join("inventory.sqlite"), &expected.key, 2)
            .await?;
    edge_fixture::sort_edges(&mut edges);
    let graph = RecordingBackend::new(false);
    // When: projection runs through the backend port.
    let result = run_projection(&store, &graph, 1).await?;
    // Then: exactly one node and one incident replacement precede acknowledgement.
    assert_eq!(
        graph.calls().await,
        vec![
            Call::Claim,
            Call::Upsert(expected.clone()),
            Call::ClearTags(expected.key.clone()),
            Call::Replace(expected.key.clone(), edges.clone()),
            Call::Chunk(expected.key, edges.clone()),
        ]
    );
    assert_eq!(
        result,
        ProjectionReport {
            acknowledged: 1,
            pending: 0,
            offline: false,
            rebuild_incomplete: false
        }
    );
    assert_eq!(store.drain_projection(1).await?, Vec::new());
    assert_eq!(graph.edges().await, edges);
    let identity = store
        .read_projection_identity()
        .await?
        .ok_or("missing target identity")?;
    assert_eq!(identity.endpoint, "recording://projection");
    assert_eq!(identity.graph, "trait_contract");
    Ok(())
}

#[tokio::test]
async fn given_deleted_key_when_projected_then_only_one_delete_is_acknowledged() -> TestResult {
    // Given: a tombstone in the real durable outbox.
    let (directory, store, expected) = pending_resource().await?;
    let pool = sqlx::SqlitePool::connect_with(
        sqlx::sqlite::SqliteConnectOptions::new()
            .filename(directory.path().join("inventory.sqlite")),
    )
    .await?;
    sqlx::query("DELETE FROM resources").execute(&pool).await?;
    sqlx::query("UPDATE projection_outbox SET deleted=1")
        .execute(&pool)
        .await?;
    pool.close().await;
    let graph = RecordingBackend::new(false);
    // When: projection drains the tombstone through the port.
    let result = run_projection(&store, &graph, 1).await?;
    // Then: deletion, not upsert/replacement, is acknowledged exactly once.
    assert_eq!(
        graph.calls().await,
        vec![Call::Claim, Call::Delete(expected.key)]
    );
    assert_eq!(
        result,
        ProjectionReport {
            acknowledged: 1,
            pending: 0,
            offline: false,
            rebuild_incomplete: false
        }
    );
    assert_eq!(store.drain_projection(1).await?, Vec::new());
    Ok(())
}

#[tokio::test]
async fn given_offline_backend_when_delivery_fails_then_exact_outbox_revision_survives()
-> TestResult {
    // Given: pending durable work and an unavailable node-write backend.
    let (_directory, store, expected) = pending_resource().await?;
    let pending = store.drain_projection(1).await?;
    let graph = RecordingBackend::new(true);
    // When: the claimed session fails its first graph write.
    let result = run_projection(&store, &graph, 1).await?;
    // Then: no acknowledgement or target identity is persisted.
    assert_eq!(
        graph.calls().await,
        vec![Call::Claim, Call::Upsert(expected)]
    );
    assert_eq!(
        result,
        ProjectionReport {
            acknowledged: 0,
            pending: 1,
            offline: true,
            rebuild_incomplete: false
        }
    );
    assert_eq!(store.drain_projection(1).await?, pending);
    assert_eq!(store.read_projection_identity().await?, None);
    Ok(())
}

async fn changed_identity_requires_rebuild(
    identity: inventory_engine::inventory::ProjectionIdentity,
) -> TestResult {
    let (_directory, store, _) = pending_resource().await?;
    let graph = RecordingBackend::new(false);
    assert_eq!(run_projection(&store, &graph, 1).await?.acknowledged, 1);
    store.write_projection_identity(&identity).await?;
    let calls = graph.calls().await;
    let report = run_projection(&store, &graph, 1).await?;
    assert_eq!(
        report,
        ProjectionReport {
            acknowledged: 0,
            pending: 0,
            offline: false,
            rebuild_incomplete: true
        }
    );
    let state = store.projection_state().await?;
    assert_eq!((state.pending, state.rebuild_required), (0, true));
    assert_eq!(store.drain_projection(1).await?, Vec::new());
    assert_eq!(graph.calls().await, calls);
    assert_eq!(store.read_projection_identity().await?, Some(identity));
    Ok(())
}

#[tokio::test]
async fn changed_endpoint_requests_a_durable_rebuild_without_graph_writes() -> TestResult {
    changed_identity_requires_rebuild(inventory_engine::inventory::ProjectionIdentity {
        endpoint: "recording://old-endpoint".into(),
        graph: "trait_contract".into(),
    })
    .await
}

#[tokio::test]
async fn changed_graph_name_requests_a_durable_rebuild_without_graph_writes() -> TestResult {
    changed_identity_requires_rebuild(inventory_engine::inventory::ProjectionIdentity {
        endpoint: "recording://projection".into(),
        graph: "old-graph".into(),
    })
    .await
}

#[tokio::test]
async fn unchanged_identity_with_empty_outbox_does_not_request_a_rebuild() -> TestResult {
    let (_directory, store, _) = pending_resource().await?;
    let graph = RecordingBackend::new(false);
    assert_eq!(run_projection(&store, &graph, 1).await?.acknowledged, 1);
    let calls = graph.calls().await;
    assert_eq!(
        run_projection(&store, &graph, 1).await?,
        ProjectionReport {
            acknowledged: 0,
            pending: 0,
            offline: false,
            rebuild_incomplete: false,
        }
    );
    let state = store.projection_state().await?;
    assert_eq!((state.pending, state.rebuild_required), (0, false));
    assert_eq!(graph.calls().await, calls);
    Ok(())
}
