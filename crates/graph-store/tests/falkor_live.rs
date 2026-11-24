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
    let name = format!("inventory-graph-test-{}", std::process::id());
    let graph = FalkorDb::connect(&endpoint, &name)?;
    graph.claim_projection(1).await?;
    graph.claim_projection(2).await?;
    graph.clear().await?;
    let outcome = conformance(&graph).await;
    let cleanup = clear_graph_and_fence(&graph, &endpoint, &name).await;
    match (outcome, cleanup) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(error), Ok(())) => Err(error),
        (Ok(()), Err(error)) => Err(error),
        (Err(error), Err(cleanup)) => Err(format!("{error}; cleanup failed: {cleanup}").into()),
    }
}

async fn clear_graph_and_fence(graph: &FalkorDb, endpoint: &str, name: &str) -> TestResult {
    let cleared = graph.clear().await;
    let mut connection = redis::Client::open(endpoint)?
        .get_multiplexed_async_connection()
        .await?;
    let key = format!("aws-inventory:projection-owner:{name}");
    let removed: u64 = redis::cmd("DEL")
        .arg(&key)
        .query_async(&mut connection)
        .await?;
    assert_eq!(removed, 1);
    let remaining: u64 = redis::cmd("EXISTS")
        .arg(&key)
        .query_async(&mut connection)
        .await?;
    assert_eq!(remaining, 0);
    cleared?;
    Ok(())
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

#[tokio::test]
#[ignore = "requires INVENTORY_GRAPH_URL pointing to FalkorDB"]
async fn paged_replacement_2001_shrinks_replays_and_rejects_foreign_edges() -> TestResult {
    let endpoint = std::env::var("INVENTORY_GRAPH_URL")?;
    let name = format!("inventory-paged-test-{}", std::process::id());
    let graph = FalkorDb::connect(&endpoint, &name)?;
    graph.claim_projection(1).await?;
    graph.clear().await?;
    let result = paged_conformance(&graph).await;
    let cleanup = clear_graph_and_fence(&graph, &endpoint, &name).await;
    match (result, cleanup) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(error), Ok(())) => Err(error),
        (Ok(()), Err(error)) => Err(error),
        (Err(error), Err(cleanup)) => Err(format!("{error}; cleanup failed: {cleanup}").into()),
    }
}

fn sorted_edges(mut edges: Vec<EdgeRow>) -> Vec<EdgeRow> {
    edges.sort_by(|left, right| {
        (&left.from, &left.to, &left.relation, &left.provenance).cmp(&(
            &right.from,
            &right.to,
            &right.relation,
            &right.provenance,
        ))
    });
    edges
}

async fn paged_edges(graph: &FalkorDb) -> TestResult<Vec<EdgeRow>> {
    let mut first = graph.edge_keys(None, 2_000).await?;
    let tail = graph.edge_keys(first.last(), 2_000).await?;
    first.extend(tail);
    Ok(sorted_edges(first))
}

async fn seed_paged_graph(graph: &FalkorDb) -> TestResult<(NodeKey, EdgeRow, Vec<EdgeRow>)> {
    use futures_util::{TryStreamExt, stream};
    let owner = key("123456789012", "paged-owner")?;
    let foreign = key("000000000000", "foreign")?;
    upsert(graph, &owner).await?;
    upsert(graph, &foreign).await?;
    let expected = (0..2_001)
        .map(|index| {
            let mut row = edge(
                &owner,
                &key("123456789012", &format!("target-{index}"))?,
                "model",
            )?;
            if index == 1_000 {
                row.relation = inventory_core::inventory::Relation::parse("USES")?;
            }
            Ok(row)
        })
        .collect::<TestResult<Vec<_>>>()?;
    stream::iter(expected.iter().map(Ok::<_, Box<dyn std::error::Error>>))
        .try_for_each(|edge| upsert(graph, &edge.to))
        .await?;
    let foreign_edge = edge(&foreign, &foreign, "foreign")?;
    graph
        .replace_edges(&foreign, std::slice::from_ref(&foreign_edge))
        .await?;
    graph
        .replace_edges(&owner, &[edge(&owner, &owner, "stale")?])
        .await?;
    Ok((owner, foreign_edge, expected))
}

async fn paged_conformance(graph: &FalkorDb) -> TestResult {
    use futures_util::stream;
    let (owner, foreign_edge, expected) = seed_paged_graph(graph).await?;
    let pages = || {
        stream::iter(
            expected
                .chunks(2_000)
                .map(|rows| Ok::<_, GraphError>(rows.to_vec())),
        )
    };
    graph.replace_edges_paged(&owner, pages()).await?;
    let mut complete = expected.clone();
    complete.push(foreign_edge.clone());
    assert_eq!(paged_edges(graph).await?, sorted_edges(complete.clone()));
    graph.replace_edges_paged(&owner, pages()).await?;
    assert_eq!(paged_edges(graph).await?, sorted_edges(complete));
    let retained = expected.first().ok_or("missing expected edge")?.clone();
    graph
        .replace_edges_paged(
            &owner,
            stream::iter([Ok::<_, GraphError>(vec![retained.clone()])]),
        )
        .await?;
    assert_eq!(
        paged_edges(graph).await?,
        sorted_edges(vec![retained, foreign_edge.clone()])
    );
    let error = graph
        .replace_edges_paged(
            &owner,
            stream::iter([
                Ok::<_, GraphError>(expected.get(..2_000).ok_or("missing first page")?.to_vec()),
                Ok(vec![foreign_edge.clone()]),
            ]),
        )
        .await
        .err()
        .ok_or("expected an error")?;
    assert_eq!(
        error.to_string(),
        "replacement edge is not incident to its owner"
    );
    let mut partial = expected
        .get(..2_000)
        .ok_or("missing applied prefix")?
        .to_vec();
    partial.push(foreign_edge.clone());
    assert_eq!(paged_edges(graph).await?, sorted_edges(partial));
    graph.replace_edges_paged(&owner, pages()).await?;
    let mut replayed = expected;
    replayed.push(foreign_edge);
    assert_eq!(paged_edges(graph).await?, sorted_edges(replayed));
    check_paged_missing_owner(graph).await?;
    Ok(())
}

async fn check_paged_missing_owner(graph: &FalkorDb) -> TestResult {
    let missing = key("123456789012", "missing-owner")?;
    let error = graph
        .replace_edges(&missing, &[])
        .await
        .err()
        .ok_or("expected an error")?;
    assert_eq!(error.to_string(), "replacement owner does not exist");
    Ok(())
}
