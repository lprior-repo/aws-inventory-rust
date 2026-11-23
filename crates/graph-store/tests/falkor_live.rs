mod common;

use common::{TestResult, edge, key, row};
use inventory_graph::falkor::FalkorDb;
use inventory_graph::{
    EdgeRow, GraphBackend, GraphError, GraphLimits, NodeKey, PathResult, Truncation,
};

#[tokio::test]
#[ignore = "requires INVENTORY_GRAPH_URL pointing to FalkorDB"]
async fn graph_backend_conformance() -> TestResult {
    let endpoint = std::env::var("INVENTORY_GRAPH_URL")?;
    let graph = FalkorDb::connect(
        &endpoint,
        &format!("inventory-graph-test-{}", std::process::id()),
    )?;
    graph.clear().await?;
    let outcome = conformance(&graph).await;
    let cleanup = graph.clear().await;
    match (outcome, cleanup) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(error), Ok(())) => Err(error),
        (Ok(()), Err(error)) => Err(error.into()),
        (Err(error), Err(cleanup)) => Err(format!("{error}; cleanup failed: {cleanup}").into()),
    }
}

async fn conformance<G: GraphBackend>(graph: &G) -> TestResult {
    let [a, b, c, foreign, alone] = given_nodes(graph).await?;
    let [ab, parallel, bc, fc] = given_edges(graph, &a, &b, &c, &foreign).await?;
    let nodes = vec![
        a.clone(),
        b.clone(),
        c.clone(),
        foreign.clone(),
        alone.clone(),
    ];
    check_node_pages(graph, nodes).await?;
    let edges = [ab.clone(), parallel.clone(), bc.clone(), fc.clone()];
    check_edge_pages(graph, &edges).await?;
    check_traversals(graph, &a, &b, &c, &alone).await?;
    check_budgets(graph, &b, &alone).await?;
    graph
        .replace_edges(&b, &[ab.clone(), parallel.clone(), bc])
        .await?;
    assert_eq!(graph.edge_keys(None, 20).await?.len(), 4);
    check_replacement(graph, &a, &b, &c, &foreign, &fc).await?;
    Ok(())
}

async fn given_nodes<G: GraphBackend>(graph: &G) -> TestResult<[NodeKey; 5]> {
    let a = key("123456789012", "a")?;
    let b = key("123456789012", "b")?;
    let c = key("123456789012", "c")?;
    let foreign = key("000000000000", "a")?;
    let alone = key("123456789012", "alone")?;
    seed_nodes(graph, [&a, &b, &c, &foreign, &alone]).await?;
    Ok([a, b, c, foreign, alone])
}

async fn given_edges<G: GraphBackend>(
    graph: &G,
    a: &NodeKey,
    b: &NodeKey,
    c: &NodeKey,
    foreign: &NodeKey,
) -> TestResult<[EdgeRow; 4]> {
    let ab = edge(a, b, "model")?;
    let parallel = edge(a, b, "reviewed-rule")?;
    let bc = edge(b, c, "model")?;
    let fc = edge(foreign, c, "model")?;
    graph
        .replace_edges(b, &[ab.clone(), parallel.clone(), bc.clone()])
        .await?;
    graph.replace_edges(c, &[bc.clone(), fc.clone()]).await?;
    Ok([ab, parallel, bc, fc])
}

async fn seed_nodes<G: GraphBackend>(graph: &G, keys: [&NodeKey; 5]) -> TestResult {
    upsert(graph, keys.first().ok_or("missing key")?).await?;
    upsert(graph, keys.get(1).ok_or("missing key")?).await?;
    upsert(graph, keys.get(2).ok_or("missing key")?).await?;
    upsert(graph, keys.get(3).ok_or("missing key")?).await?;
    upsert(graph, keys.get(4).ok_or("missing key")?).await?;
    upsert(graph, keys.first().ok_or("missing key")?).await?;
    Ok(())
}

async fn upsert<G: GraphBackend>(graph: &G, key: &NodeKey) -> TestResult {
    graph.upsert_node(&row(key)).await?;
    Ok(())
}

async fn check_node_pages<G: GraphBackend>(graph: &G, mut expected: Vec<NodeKey>) -> TestResult {
    let first = graph.node_keys(None, 2).await?;
    let second = graph.node_keys(first.last(), 2).await?;
    let third = graph.node_keys(second.last(), 2).await?;
    assert!(graph.node_keys(third.last(), 2).await?.is_empty());
    let mut actual: Vec<_> = first.into_iter().chain(second).chain(third).collect();
    actual.sort();
    expected.sort();
    assert_eq!(actual, expected);
    Ok(())
}

async fn check_edge_pages<G: GraphBackend>(graph: &G, edges: &[EdgeRow]) -> TestResult {
    let first = graph.edge_keys(None, 1).await?;
    let second = graph.edge_keys(first.last(), 1).await?;
    let third = graph.edge_keys(second.last(), 1).await?;
    let fourth = graph.edge_keys(third.last(), 1).await?;
    assert!(graph.edge_keys(fourth.last(), 1).await?.is_empty());
    let actual: Vec<_> = first
        .into_iter()
        .chain(second)
        .chain(third)
        .chain(fourth)
        .collect();
    assert_eq!(actual.len(), edges.len());
    assert!(edges.iter().all(|edge| actual.contains(edge)));
    Ok(())
}

async fn check_traversals<G: GraphBackend>(
    graph: &G,
    a: &NodeKey,
    b: &NodeKey,
    c: &NodeKey,
    alone: &NodeKey,
) -> TestResult {
    let limits = GraphLimits::new(2, 20, 20)?;
    assert_eq!(graph.neighbors(b, limits).await?.edges.len(), 4);
    assert_eq!(graph.blast_radius(c, limits).await?.edges.len(), 4);
    assert!(graph.blast_radius(a, limits).await?.edges.is_empty());
    assert!(graph.neighbors(alone, limits).await?.edges.is_empty());
    assert_eq!(graph.path(a, a, limits).await?, PathResult::SameNode);
    assert_eq!(graph.path(a, alone, limits).await?, PathResult::NotFound);
    let PathResult::Path(path) = graph.path(a, c, limits).await? else {
        return Err("expected a two-hop path".into());
    };
    assert_eq!(path.len(), 2);
    assert_eq!(
        graph.path(a, c, GraphLimits::new(1, 20, 20)?).await?,
        PathResult::NotFound
    );
    Ok(())
}

async fn check_budgets<G: GraphBackend>(graph: &G, b: &NodeKey, alone: &NodeKey) -> TestResult {
    let limits = GraphLimits::new(2, 1, 20)?;
    assert_eq!(
        graph.neighbors(b, limits).await?.truncation,
        Truncation::Budget
    );
    assert!(matches!(
        graph.path(b, alone, limits).await,
        Err(GraphError::Truncated)
    ));
    Ok(())
}

async fn check_replacement<G: GraphBackend>(
    graph: &G,
    a: &NodeKey,
    b: &NodeKey,
    c: &NodeKey,
    foreign: &NodeKey,
    fc: &EdgeRow,
) -> TestResult {
    graph.replace_edges(b, &[]).await?;
    assert_eq!(graph.edge_keys(None, 20).await?, vec![fc.clone()]);
    let unrelated = edge(a, c, "model")?;
    assert!(matches!(
        graph.replace_edges(b, &[unrelated]).await,
        Err(GraphError::UnrelatedEdge)
    ));
    assert_eq!(graph.edge_keys(None, 20).await?, vec![fc.clone()]);
    check_missing_and_delete(graph, c, foreign, fc).await
}

async fn check_missing_and_delete<G: GraphBackend>(
    graph: &G,
    c: &NodeKey,
    foreign: &NodeKey,
    fc: &EdgeRow,
) -> TestResult {
    let missing = key("123456789012", "missing")?;
    graph
        .replace_edges(c, &[edge(c, &missing, "model")?, fc.clone()])
        .await?;
    assert_eq!(graph.edge_keys(None, 20).await?, vec![fc.clone()]);
    graph.delete_node(foreign).await?;
    graph.delete_node(foreign).await?;
    assert!(graph.edge_keys(None, 20).await?.is_empty());
    graph.clear().await?;
    assert!(graph.node_keys(None, 20).await?.is_empty());
    assert!(graph.edge_keys(None, 20).await?.is_empty());
    Ok(())
}
