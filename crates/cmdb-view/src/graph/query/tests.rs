use super::*;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn node(id: &str) -> GraphNode {
    GraphNode {
        account: "111111111111".to_owned(),
        region: "us-east-1".to_owned(),
        kind: "AWS::S3::Bucket".to_owned(),
        id: id.to_owned(),
    }
}

fn edge(from: &str, to: &str) -> GraphEdge {
    GraphEdge {
        from: node(from),
        to: node(to),
        relation: "depends_on".to_owned(),
        provenance: "model".to_owned(),
    }
}

fn traversal(nodes: usize, edges: usize) -> Result<Traversal, ViewError> {
    Ok(Traversal::new(
        key_digest(&node("a"))?,
        GraphLimits {
            depth: 3,
            nodes,
            edges,
        },
    ))
}

#[test]
fn given_chain_when_each_bounded_frontier_advances_then_exact_edge_budget_loses_no_layer()
-> TestResult {
    let mut state = traversal(3, 2)?;
    state.add_layer(vec![edge("a", "b")])?;
    state.add_layer(vec![edge("b", "c")])?;
    state.add_layer(Vec::new())?;
    assert_eq!(state.edges, vec![edge("a", "b"), edge("b", "c")]);
    assert!(!state.truncated);
    assert!(state.frontier.is_empty());
    Ok(())
}

#[test]
fn given_parallel_provenances_when_frontier_has_budget_then_relationship_identity_is_preserved()
-> TestResult {
    let mut state = traversal(2, 2)?;
    let mut parallel = edge("a", "b");
    parallel.provenance = "reviewed".to_owned();
    state.add_layer(vec![edge("a", "b"), parallel.clone()])?;
    assert_eq!(state.edges, vec![edge("a", "b"), parallel]);
    assert_eq!(state.visited.len(), 2);
    assert_eq!(state.frontier.len(), 1);
    assert!(!state.truncated);
    Ok(())
}

#[test]
fn given_lookahead_when_edge_budget_is_exhausted_then_result_is_truncated() -> TestResult {
    let mut state = traversal(4, 2)?;
    state.add_layer(vec![edge("a", "b"), edge("a", "c"), edge("a", "d")])?;
    assert_eq!(state.edges.len(), 2);
    assert_eq!(state.visited.len(), 3);
    assert!(state.truncated);
    Ok(())
}

#[test]
fn given_high_fanout_when_node_budget_is_one_then_no_neighbor_is_admitted() -> TestResult {
    let mut state = traversal(1, 2000)?;
    state.add_layer(vec![edge("a", "b"), edge("a", "c")])?;
    assert!(state.edges.is_empty());
    assert_eq!(state.visited.len(), 1);
    assert!(state.frontier.is_empty());
    assert!(state.truncated);
    Ok(())
}

#[test]
fn given_self_loop_when_node_budget_is_one_then_root_is_counted_once() -> TestResult {
    let mut state = traversal(1, 1)?;
    state.add_layer(vec![edge("a", "a")])?;
    assert_eq!(state.edges, vec![edge("a", "a")]);
    assert_eq!(state.visited.len(), 1);
    assert!(state.frontier.is_empty());
    assert!(!state.truncated);
    Ok(())
}

#[test]
fn given_unrelated_edge_when_decoding_layer_then_response_is_rejected() -> TestResult {
    let mut state = traversal(3, 2)?;
    assert!(matches!(
        state.add_layer(vec![edge("b", "c")]),
        Err(ViewError::GraphResponse)
    ));
    Ok(())
}

fn reply(rows: Vec<redis::Value>) -> redis::Value {
    redis::Value::Array(vec![
        redis::Value::Array(Vec::new()),
        redis::Value::Array(rows),
        redis::Value::Array(Vec::new()),
    ])
}

fn row() -> redis::Value {
    redis::Value::Array(
        [
            "111111111111",
            "us-east-1",
            "AWS::S3::Bucket",
            "a",
            "111111111111",
            "us-east-1",
            "AWS::S3::Bucket",
            "b",
            "depends_on",
            "model",
        ]
        .into_iter()
        .map(|text| redis::Value::BulkString(text.as_bytes().to_vec()))
        .collect(),
    )
}

#[test]
fn given_malformed_or_overbudget_rows_when_parsing_then_errors_are_explicit() {
    assert!(matches!(
        parse_rows(redis::Value::Nil, 1),
        Err(ViewError::GraphResponse)
    ));
    assert!(matches!(
        parse_rows(reply(vec![redis::Value::Nil]), 1),
        Err(ViewError::GraphResponse)
    ));
    assert!(matches!(
        parse_rows(reply(vec![row(), row()]), 1),
        Err(ViewError::GraphResponse)
    ));
    assert!(matches!(
        parse_rows(reply(vec![redis::Value::Array(vec![])]), 1),
        Err(ViewError::GraphResponse)
    ));
}

#[test]
fn given_valid_row_when_parsing_then_full_identity_and_provenance_survive() -> TestResult {
    assert_eq!(parse_rows(reply(vec![row()]), 1)?, vec![edge("a", "b")]);
    Ok(())
}

#[test]
fn given_frontier_when_compiling_query_then_limit_precedes_any_combinatorial_work() -> TestResult {
    let root = key_digest(&node("a"))?;
    let previous = key_digest(&node("b"))?;
    let query = traversal_query(&[root.clone()], &[previous.clone()], 2001);
    assert!(query.starts_with(&format!("UNWIND ['{root}'] AS frontier_key MATCH")));
    assert!(query.contains(&format!("NOT n.key IN ['{previous}']")));
    assert!(query.ends_with("LIMIT 2001"));
    [
        "[*",
        "UNWIND relationships",
        "DISTINCT",
        "ORDER BY",
        "min(",
        "WITH ",
    ]
    .iter()
    .for_each(|operation| assert!(!query.contains(operation), "{query}"));
    Ok(())
}

// UI-04: Directed blast radius test - verify the query follows incoming
// edges (n)-[r]->(s) to find upstream dependents, not outgoing edges.
// Graph: A->B->C. Blast radius of C should find A and B.
#[test]
fn directed_blast_radius_finds_upstream_dependents() -> TestResult {
    let a = node("a");
    let b = node("b");
    let c = node("c");
    let root = key_digest(&c)?;
    let expanded = vec![key_digest(&c)?];
    let query = blast_radius_query(&[root], &expanded, 10);

    // The query should match incoming edges: (n)-[r]->(s) where s is the root
    assert!(
        query.contains("(n:Resource)-[r]->(s:Resource {key:frontier_key})"),
        "blast radius query must follow incoming edges"
    );
    assert!(!query.contains("(s:Resource {key:frontier_key})-[r]-(n:Resource)"),
        "blast radius query must not follow undirected edges");
    Ok(())
}

// UI-04: Verify the neighborhood query is undirected (contrasts with blast radius)
#[test]
fn neighborhood_query_is_undirected() -> TestResult {
    let root = key_digest(&node("a"))?;
    let query = traversal_query(&[root], &[], 10);
    assert!(
        query.contains("(s:Resource {key:frontier_key})-[r]-(n:Resource)"),
        "neighborhood query must be undirected"
    );
    Ok(())
}
