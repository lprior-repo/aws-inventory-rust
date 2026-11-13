//! Graph neighborhood depth, caps, cycles, dangling targets and determinism.

mod support;

use cmdb_view::{GraphLimits, MAX_GRAPH_EDGES, MAX_GRAPH_NODES, Snapshot, Subgraph, ViewError};

fn ids(graph: &Subgraph<'_>) -> Vec<String> {
    graph.nodes.iter().map(|node| node.id.to_owned()).collect()
}

fn edges(graph: &Subgraph<'_>) -> Vec<(String, String, String)> {
    graph
        .edges
        .iter()
        .map(|edge| {
            (
                edge.from_id.clone(),
                edge.to_id.clone(),
                edge.relation.clone(),
            )
        })
        .collect()
}

fn limits(depth: u8, max_nodes: usize, max_edges: usize) -> Result<GraphLimits, ViewError> {
    GraphLimits::new(depth, max_nodes, max_edges)
}

fn canonical() -> Result<Snapshot, Box<dyn std::error::Error>> {
    support::snapshot()
}

#[test]
fn depth_one_returns_root_neighbors_and_marks_depth_truncation()
-> Result<(), Box<dyn std::error::Error>> {
    let snapshot = canonical()?;
    let graph = snapshot.neighborhood(support::INSTANCE_ONE, limits(1, 100, 100)?);
    assert_eq!(
        ids(&graph),
        vec![
            support::BUCKET,
            support::INSTANCE_ONE,
            support::INSTANCE_TWO
        ]
    );
    assert_eq!(graph.root.id, support::INSTANCE_ONE);
    assert!(graph.root.is_present());
    assert_eq!(graph.edges.len(), 3);
    assert!(graph.truncated_depth);
    assert!(!graph.truncated_nodes);
    assert!(!graph.truncated_edges);
    Ok(())
}

#[test]
fn depth_two_includes_dangling_target_as_absent_node() -> Result<(), Box<dyn std::error::Error>> {
    let snapshot = canonical()?;
    let graph = snapshot.neighborhood(support::INSTANCE_ONE, limits(2, 100, 100)?);
    assert_eq!(
        ids(&graph),
        vec![
            support::BUCKET,
            support::INSTANCE_ONE,
            support::INSTANCE_TWO,
            support::DANGLING,
            support::ROLE
        ]
    );
    let dangling = graph
        .nodes
        .iter()
        .find(|node| node.id == support::DANGLING)
        .ok_or("dangling node")?;
    assert!(!dangling.is_present());
    assert!(dangling.resource.is_none());
    assert_eq!(graph.edges.len(), 5);
    assert!(edges(&graph).contains(&(
        support::BUCKET.to_owned(),
        support::DANGLING.to_owned(),
        "references".to_owned()
    )));
    assert!(!graph.truncated_depth);
    assert!(!graph.truncated_nodes);
    assert!(!graph.truncated_edges);
    Ok(())
}

#[test]
fn cycle_terminates_and_keeps_both_directions() -> Result<(), Box<dyn std::error::Error>> {
    let snapshot = canonical()?;
    let graph = snapshot.neighborhood(support::INSTANCE_TWO, limits(2, 100, 100)?);
    assert_eq!(graph.edges.len(), 4);
    let pairs: Vec<(&str, &str)> = graph
        .edges
        .iter()
        .map(|edge| (edge.from_id.as_str(), edge.to_id.as_str()))
        .collect();
    assert!(pairs.contains(&(support::INSTANCE_ONE, support::INSTANCE_TWO)));
    assert!(pairs.contains(&(support::INSTANCE_TWO, support::INSTANCE_ONE)));
    Ok(())
}

#[test]
fn node_cap_truncates_deterministically() -> Result<(), Box<dyn std::error::Error>> {
    let snapshot = canonical()?;
    let graph = snapshot.neighborhood(support::INSTANCE_ONE, limits(2, 3, 100)?);
    assert_eq!(graph.nodes.len(), 3);
    assert_eq!(
        ids(&graph),
        vec![
            support::BUCKET,
            support::INSTANCE_ONE,
            support::INSTANCE_TWO
        ]
    );
    assert_eq!(graph.edges.len(), 3);
    assert!(graph.truncated_nodes);
    assert!(!graph.truncated_edges);
    Ok(())
}

#[test]
fn edge_cap_flags_truncation_only_when_edges_are_omitted() -> Result<(), Box<dyn std::error::Error>>
{
    let snapshot = canonical()?;
    let capped = snapshot.neighborhood(support::INSTANCE_ONE, limits(2, 100, 2)?);
    assert_eq!(capped.edges.len(), 2);
    assert!(capped.truncated_edges);
    assert_eq!(ids(&capped).len(), 5);

    let exact = snapshot.neighborhood(support::INSTANCE_ONE, limits(2, 100, 5)?);
    assert_eq!(exact.edges.len(), 5);
    assert!(!exact.truncated_edges);
    Ok(())
}

#[test]
fn repeated_queries_are_identical() -> Result<(), Box<dyn std::error::Error>> {
    let snapshot = canonical()?;
    let first = snapshot.neighborhood(support::INSTANCE_TWO, limits(2, 100, 100)?);
    let second = snapshot.neighborhood(support::INSTANCE_TWO, limits(2, 100, 100)?);
    assert_eq!(ids(&first), ids(&second));
    assert_eq!(edges(&first), edges(&second));
    assert_eq!(first.truncated_nodes, second.truncated_nodes);
    assert_eq!(first.truncated_edges, second.truncated_edges);
    assert_eq!(first.truncated_depth, second.truncated_depth);
    Ok(())
}

#[test]
fn unknown_root_is_an_absent_node() -> Result<(), Box<dyn std::error::Error>> {
    let snapshot = canonical()?;
    let graph = snapshot.neighborhood("ghost", limits(2, 10, 10)?);
    assert_eq!(ids(&graph), vec!["ghost".to_owned()]);
    assert!(!graph.root.is_present());
    assert!(graph.edges.is_empty());
    assert!(!graph.truncated_nodes);
    assert!(!graph.truncated_edges);
    assert!(!graph.truncated_depth);
    Ok(())
}

#[test]
fn rejects_invalid_depth_and_caps() {
    assert!(matches!(
        GraphLimits::new(0, 1, 1),
        Err(ViewError::InvalidGraphDepth)
    ));
    assert!(matches!(
        GraphLimits::new(3, 1, 1),
        Err(ViewError::InvalidGraphDepth)
    ));
    assert!(matches!(
        GraphLimits::new(1, 0, 1),
        Err(ViewError::InvalidGraphLimit)
    ));
    assert!(matches!(
        GraphLimits::new(1, usize::MAX, 1),
        Err(ViewError::InvalidGraphLimit)
    ));
    assert!(matches!(
        GraphLimits::new(1, MAX_GRAPH_NODES, 0),
        Err(ViewError::InvalidGraphLimit)
    ));
    assert!(matches!(
        GraphLimits::new(1, 1, MAX_GRAPH_EDGES.saturating_add(1)),
        Err(ViewError::InvalidGraphLimit)
    ));
}
