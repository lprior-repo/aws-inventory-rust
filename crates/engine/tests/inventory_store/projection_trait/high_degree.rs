use super::edge_fixture::{seed_edges, sort_edges};
use super::{Call, RecordingBackend, TestResult, pending_resource};
use inventory_engine::inventory::{BinaryStore, ProjectionReport, run_projection};
use inventory_graph::{EdgeRow, Provenance};

async fn chunks(graph: &RecordingBackend) -> Vec<usize> {
    graph
        .calls()
        .await
        .into_iter()
        .filter_map(|call| match call {
            Call::Chunk(_, edges) => Some(edges.len()),
            _ => None,
        })
        .collect()
}

async fn complete_degree(count: usize, expected_chunks: Vec<usize>) -> TestResult {
    // Given: one real pending key with a complete, high-degree SQLite incident set.
    let (directory, store, owner) = pending_resource().await?;
    let mut expected = seed_edges(
        &directory.path().join("inventory.sqlite"),
        &owner.key,
        count,
    )
    .await?;
    sort_edges(&mut expected);
    let graph = RecordingBackend::new(false);
    // When: store pages are streamed through the paged replacement port.
    let report = run_projection(&store, &graph, 1_000).await?;
    // Then: bounded chunks cover exactly the set before the revision is acknowledged.
    assert_eq!(chunks(&graph).await, expected_chunks);
    assert_eq!(graph.edges().await, expected);
    assert_eq!(
        report,
        ProjectionReport {
            acknowledged: 1,
            pending: 0,
            offline: false,
            rebuild_incomplete: false
        }
    );
    assert_eq!(store.drain_projection(1).await?, Vec::new());
    assert_eq!(
        graph
            .calls()
            .await
            .iter()
            .filter(|call| matches!(call, Call::Replace(_, _)))
            .count(),
        1
    );
    Ok(())
}

#[tokio::test]
async fn given_2000_incident_edges_when_projected_then_one_complete_chunk_is_acknowledged()
-> TestResult {
    complete_degree(2_000, vec![2_000]).await
}

#[tokio::test]
async fn given_2001_incident_edges_when_projected_then_both_chunks_are_acknowledged() -> TestResult
{
    complete_degree(2_001, vec![2_000, 1]).await
}

#[tokio::test]
async fn given_5000_incident_edges_when_projected_then_three_chunks_are_acknowledged() -> TestResult
{
    complete_degree(5_000, vec![2_000, 2_000, 1_000]).await
}

#[tokio::test]
async fn given_second_chunk_failure_when_replayed_then_no_ack_until_exact_complete_replacement()
-> TestResult {
    // Given: stale graph data and 5,000 authoritative incident edges in the store.
    let (directory, store, owner) = pending_resource().await?;
    let mut expected = seed_edges(
        &directory.path().join("inventory.sqlite"),
        &owner.key,
        5_000,
    )
    .await?;
    sort_edges(&mut expected);
    let pending = store.drain_projection(1).await?;
    let graph = RecordingBackend::new(false);
    let mut stale = expected.first().ok_or("missing expected edge")?.clone();
    stale.provenance = Provenance::parse("stale")?;
    graph.seed_edges(vec![stale]).await;
    graph
        .fail_second_chunk_once(inventory_graph::GraphError::Transport(
            "second edge chunk failed".into(),
        ))
        .await;
    // When: the second chunk fails after the first has changed the graph.
    let failed = run_projection(&store, &graph, 1_000).await?;
    // Then: no durable revision is acknowledged; readers see only the successful prefix.
    assert_eq!(
        failed,
        ProjectionReport {
            acknowledged: 0,
            pending: 1,
            offline: true,
            rebuild_incomplete: false
        }
    );
    assert_eq!(store.drain_projection(1).await?, pending);
    assert_eq!(chunks(&graph).await, vec![2_000, 2_000]);
    assert_eq!(
        graph.edges().await,
        expected.get(..2_000).ok_or("missing first chunk")?
    );
    assert_complete_replay(&store, &graph, expected).await
}

async fn assert_complete_replay(
    store: &BinaryStore,
    graph: &RecordingBackend,
    expected: Vec<EdgeRow>,
) -> TestResult {
    // When: the complete replacement is replayed from its beginning.
    let replayed = run_projection(store, graph, 1_000).await?;
    // Then: stale/prefix data is replaced, without missing or duplicate edges.
    assert_eq!(
        replayed,
        ProjectionReport {
            acknowledged: 1,
            pending: 0,
            offline: false,
            rebuild_incomplete: false
        }
    );
    assert_eq!(graph.edges().await, expected);
    assert_eq!(chunks(graph).await, vec![2_000, 2_000, 2_000, 2_000, 1_000]);
    assert_eq!(
        graph
            .calls()
            .await
            .iter()
            .filter(|call| matches!(call, Call::Replace(_, _)))
            .count(),
        2
    );
    assert_eq!(store.drain_projection(1).await?, Vec::new());
    Ok(())
}

#[tokio::test]
async fn given_foreign_later_page_when_produced_then_no_foreign_edge_or_ack_escapes() -> TestResult
{
    use super::fault::{Fault, FaultStore};
    use inventory_engine::inventory::InventoryStore;
    // Given: two valid pages precede a corrupt final store page.
    let (directory, store, owner) = pending_resource().await?;
    let mut expected = seed_edges(
        &directory.path().join("inventory.sqlite"),
        &owner.key,
        2_001,
    )
    .await?;
    sort_edges(&mut expected);
    let pending = store.drain_projection(1).await?;
    let store = FaultStore::new(store, Fault::ForeignEdges);
    let graph = RecordingBackend::new(false);
    // When: the producer encounters an edge excluding its owner.
    let error = run_projection(&store, &graph, 1_000)
        .await
        .err()
        .ok_or("expected invalid edge")?;
    // Then: the malformed page never reaches the backend, nor is work acknowledged.
    assert_eq!(
        error.to_string(),
        "projection query is invalid or exceeds its bound"
    );
    assert_eq!(store.drain_projection(1).await?, pending);
    assert_eq!(
        store
            .acknowledgements
            .load(std::sync::atomic::Ordering::SeqCst),
        0
    );
    assert_eq!(chunks(&graph).await, vec![2_000]);
    assert_eq!(
        graph.edges().await,
        expected.get(..2_000).ok_or("missing accepted prefix")?
    );
    // When: the authoritative store is replayed without the injected corruption.
    let replay = run_projection(&store.inner, &graph, 1_000).await?;
    // Then: exactly the new incident set is visible and the revision is acknowledged.
    assert_eq!(
        replay,
        ProjectionReport {
            acknowledged: 1,
            pending: 0,
            offline: false,
            rebuild_incomplete: false,
        }
    );
    assert_eq!(graph.edges().await, expected);
    assert_eq!(chunks(&graph).await, vec![2_000, 2_000, 1]);
    assert_eq!(store.drain_projection(1).await?, Vec::new());
    Ok(())
}

async fn rejected_second_chunk(error: inventory_graph::GraphError) -> TestResult {
    // Given: a complete replacement requiring two chunks.
    let (directory, store, owner) = pending_resource().await?;
    let mut expected = seed_edges(
        &directory.path().join("inventory.sqlite"),
        &owner.key,
        2_001,
    )
    .await?;
    sort_edges(&mut expected);
    let pending = store.drain_projection(1).await?;
    let graph = RecordingBackend::new(false);
    graph.fail_second_chunk_once(error).await;
    // When: the adapter rejects the later chunk.
    let error = run_projection(&store, &graph, 1_000)
        .await
        .err()
        .ok_or("expected rejected chunk")?;
    // Then: the exact revision remains pending, with only the first chunk visible.
    assert_eq!(
        error.to_string(),
        "projection query is invalid or exceeds its bound"
    );
    assert_eq!(store.drain_projection(1).await?, pending);
    assert_eq!(
        graph.edges().await,
        expected.get(..2_000).ok_or("missing first chunk")?
    );
    // When: the complete replacement is replayed.
    let replayed = run_projection(&store, &graph, 1_000).await?;
    // Then: acknowledgement follows convergence to exactly the authoritative set.
    assert_eq!(
        replayed,
        ProjectionReport {
            acknowledged: 1,
            pending: 0,
            offline: false,
            rebuild_incomplete: false,
        }
    );
    assert_eq!(graph.edges().await, expected);
    assert_eq!(store.drain_projection(1).await?, Vec::new());
    Ok(())
}

#[tokio::test]
async fn given_later_limits_rejection_when_replayed_then_no_ack_until_convergence() -> TestResult {
    rejected_second_chunk(inventory_graph::GraphError::Limits).await
}

#[tokio::test]
async fn given_later_unrelated_rejection_when_replayed_then_no_ack_until_convergence() -> TestResult
{
    rejected_second_chunk(inventory_graph::GraphError::UnrelatedEdge).await
}
