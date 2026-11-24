use super::*;
use crate::Provenance;
use inventory_core::inventory::{AccountId, Kind, Region, Relation, ResourceId};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn node(id: &str) -> TestResult<NodeKey> {
    Ok(NodeKey {
        account: AccountId::parse("111111111111")?,
        region: Region::parse("us-east-1")?,
        kind: Kind::parse("AWS::S3::Bucket")?,
        id: ResourceId::parse(id)?,
    })
}

fn edge(from: &str, to: &str) -> TestResult<EdgeRow> {
    Ok(EdgeRow {
        from: node(from)?,
        to: node(to)?,
        relation: Relation::parse("depends_on")?,
        provenance: Provenance::parse("model")?,
    })
}

fn traversal(nodes: usize, edges: usize) -> TestResult<Traversal> {
    Ok(Traversal::new(
        &node("a")?,
        GraphLimits::new(3, nodes, edges)?,
    )?)
}

#[test]
fn given_chain_when_each_bounded_frontier_advances_then_exact_edge_budget_loses_no_layer()
-> TestResult {
    let mut state = traversal(3, 2)?;
    state.add_layer(vec![edge("a", "b")?])?;
    state.add_layer(vec![edge("b", "c")?])?;
    state.add_layer(Vec::new())?;
    assert_eq!(state.edges, vec![edge("a", "b")?, edge("b", "c")?]);
    assert_eq!(state.truncation, Truncation::Complete);
    assert_eq!(state.frontier, Vec::new());
    Ok(())
}

#[test]
fn given_parallel_provenances_when_frontier_has_budget_then_relationship_identity_is_preserved()
-> TestResult {
    let mut state = traversal(2, 2)?;
    let mut parallel = edge("a", "b")?;
    parallel.provenance = Provenance::parse("reviewed")?;
    state.add_layer(vec![edge("a", "b")?, parallel.clone()])?;
    assert_eq!(state.edges, vec![edge("a", "b")?, parallel]);
    assert_eq!(state.visited, vec![node("a")?, node("b")?]);
    assert_eq!(state.frontier, vec![node("b")?]);
    assert_eq!(state.truncation, Truncation::Complete);
    Ok(())
}

#[test]
fn given_lookahead_when_edge_budget_is_exhausted_then_result_is_truncated() -> TestResult {
    let mut state = traversal(4, 2)?;
    state.add_layer(vec![edge("a", "b")?, edge("a", "c")?, edge("a", "d")?])?;
    assert_eq!(state.edges, vec![edge("a", "b")?, edge("a", "c")?]);
    assert_eq!(state.visited, vec![node("a")?, node("b")?, node("c")?]);
    assert_eq!(state.truncation, Truncation::Budget);
    Ok(())
}

#[test]
fn given_high_fanout_when_node_budget_is_one_then_no_neighbor_is_admitted() -> TestResult {
    let mut state = traversal(1, 2000)?;
    state.add_layer(vec![edge("a", "b")?, edge("a", "c")?])?;
    assert_eq!(state.edges, Vec::new());
    assert_eq!(state.visited, vec![node("a")?]);
    assert_eq!(state.frontier, Vec::new());
    assert_eq!(state.truncation, Truncation::Budget);
    Ok(())
}

#[test]
fn given_self_loop_when_node_budget_is_one_then_root_is_counted_once() -> TestResult {
    let mut state = traversal(1, 1)?;
    state.add_layer(vec![edge("a", "a")?])?;
    assert_eq!(state.edges, vec![edge("a", "a")?]);
    assert_eq!(state.visited, vec![node("a")?]);
    assert_eq!(state.frontier, Vec::new());
    assert_eq!(state.truncation, Truncation::Complete);
    Ok(())
}

#[test]
fn given_unrelated_edge_when_decoding_layer_then_response_is_rejected() -> TestResult {
    let mut state = traversal(3, 2)?;
    let error = state
        .add_layer(vec![edge("b", "c")?])
        .err()
        .ok_or("expected an error")?;
    assert_eq!(
        error.to_string(),
        "graph response is malformed or exceeds its row bound"
    );
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
fn given_malformed_or_overbudget_rows_when_parsing_then_errors_are_explicit() -> TestResult {
    [
        redis::Value::Nil,
        reply(vec![redis::Value::Nil]),
        reply(vec![row(), row()]),
        reply(vec![redis::Value::Array(vec![])]),
    ]
    .into_iter()
    .try_for_each(|value| {
        let error = response::edges(&value, 1)
            .err()
            .ok_or("expected an error")?;
        assert_eq!(
            error.to_string(),
            "graph response is malformed or exceeds its row bound"
        );
        Ok::<_, Box<dyn std::error::Error>>(())
    })
}

#[test]
fn given_valid_row_when_parsing_then_full_identity_and_provenance_survive() -> TestResult {
    assert_eq!(
        response::edges(&reply(vec![row()]), 1)?,
        vec![edge("a", "b")?]
    );
    Ok(())
}

#[test]
fn given_cycle_and_shortcut_when_finding_path_then_shortest_edges_keep_stored_direction()
-> TestResult {
    let edges = vec![
        edge("a", "b")?,
        edge("b", "c")?,
        edge("c", "a")?,
        edge("d", "c")?,
    ];
    let result = path::shortest(
        &edges,
        &node("a")?,
        &node("d")?,
        GraphLimits::new(3, 200, 500)?,
    )?;
    assert_eq!(result, Some(vec![edge("c", "a")?, edge("d", "c")?]));
    Ok(())
}

#[test]
fn given_parallel_provenances_when_finding_path_then_nodes_are_visited_once() -> TestResult {
    let mut reviewed = edge("a", "b")?;
    reviewed.provenance = Provenance::parse("reviewed")?;
    let edges = vec![edge("a", "b")?, reviewed, edge("b", "c")?];
    assert_eq!(
        path::shortest(
            &edges,
            &node("a")?,
            &node("c")?,
            GraphLimits::new(2, 200, 500)?
        )?,
        Some(vec![edge("a", "b")?, edge("b", "c")?])
    );
    Ok(())
}

#[test]
fn given_depth_budget_when_target_is_farther_then_no_partial_path_is_returned() -> TestResult {
    let edges = vec![edge("a", "b")?, edge("b", "c")?];
    assert_eq!(
        path::shortest(
            &edges,
            &node("a")?,
            &node("c")?,
            GraphLimits::new(1, 200, 500)?
        )?,
        None
    );
    assert_eq!(
        path::shortest(
            &edges,
            &node("a")?,
            &node("missing")?,
            GraphLimits::new(3, 200, 500)?
        )?,
        None
    );
    assert_eq!(
        path::shortest(
            &edges,
            &node("a")?,
            &node("a")?,
            GraphLimits::new(3, 200, 500)?
        )?,
        Some(Vec::new())
    );
    Ok(())
}

#[test]
fn given_shared_id_when_target_has_another_account_then_identity_does_not_collapse() -> TestResult {
    let edges = vec![edge("a", "b")?];
    let mut other = node("b")?;
    other.account = AccountId::parse("222222222222")?;
    assert_eq!(
        path::shortest(&edges, &node("a")?, &other, GraphLimits::new(1, 200, 500)?)?,
        None
    );
    Ok(())
}

#[test]
fn given_frontier_when_compiling_query_then_limit_precedes_any_combinatorial_work() -> TestResult {
    let query = cypher::layer(&[node("a")?], &[node("b")?], 2001, Direction::Both)?;
    assert!(query.ends_with("LIMIT 2001"));
    let root = crate::encoding::digest(&node("a")?)?;
    let previous = crate::encoding::digest(&node("b")?)?;
    assert!(query.starts_with(&format!(
        "CYPHER frontier=['{root}'] excluded=['{previous}'] UNWIND $frontier AS frontier_key MATCH"
    )));
    assert!(query.contains("NOT n.key IN $excluded"));
    [
        "[*",
        "UNWIND relationships",
        "DISTINCT",
        "ORDER BY",
        "min(",
        "WITH ",
    ]
    .iter()
    .for_each(|operation| assert!(!query.contains(operation)));
    Ok(())
}

#[test]
fn directed_blast_radius_finds_upstream_dependents() -> TestResult {
    let query = cypher::layer(&[node("c")?], &[node("c")?], 10, Direction::Incoming)?;
    let root = crate::encoding::digest(&node("c")?)?;
    assert_eq!(
        query,
        format!(
            "CYPHER frontier=['{root}'] excluded=['{root}'] UNWIND $frontier AS frontier_key MATCH (n:Resource)-[r]->(s:Resource {{key:frontier_key}}) WHERE NOT n.key IN $excluded RETURN startNode(r).account,startNode(r).region,startNode(r).kind,startNode(r).id,endNode(r).account,endNode(r).region,endNode(r).kind,endNode(r).id,r.relation,r.provenance LIMIT 10"
        )
    );
    Ok(())
}

#[test]
fn neighborhood_query_is_undirected() -> TestResult {
    let query = cypher::layer(&[node("a")?], &[], 10, Direction::Both)?;
    let root = crate::encoding::digest(&node("a")?)?;
    assert_eq!(
        query,
        format!(
            "CYPHER frontier=['{root}'] excluded=[] UNWIND $frontier AS frontier_key MATCH (s:Resource {{key:frontier_key}})-[r]-(n:Resource) WHERE NOT n.key IN $excluded AND (NOT n.key IN $frontier OR startNode(r)=s) RETURN startNode(r).account,startNode(r).region,startNode(r).kind,startNode(r).id,endNode(r).account,endNode(r).region,endNode(r).kind,endNode(r).id,r.relation,r.provenance LIMIT 10"
        )
    );
    Ok(())
}

#[test]
fn given_path_plus_node_budget_when_searching_then_graph_reports_truncation() -> TestResult {
    let limits = GraphLimits::new(2, 1, 10)?;
    let mut state = Traversal::new(&node("a")?, limits)?;
    state.add_layer(vec![edge("a", "b")?])?;
    let neighborhood = GraphNeighborhood {
        edges: state.edges,
        truncation: state.truncation,
    };
    let error = shortest(&neighborhood, &node("a")?, &node("b")?, limits)
        .err()
        .ok_or("expected an error")?;
    assert_eq!(
        error.to_string(),
        "path search exceeded its traversal budget"
    );
    Ok(())
}
