use super::builder;
use super::graph::{DagGraph, MAX_EDGES, MAX_LABEL_CHARS, MAX_NODES, NodeKind, truncate_label};
use super::layout::DagLayout;

#[cfg(test)]
#[path = "projection_tests.rs"]
mod projection_tests;

#[test]
fn labels_are_capped() {
    let long = "a".repeat(MAX_LABEL_CHARS.saturating_add(8));
    let capped = truncate_label(&long);
    assert_eq!(capped.chars().count(), MAX_LABEL_CHARS);
    assert!(capped.ends_with('…'));
    assert_eq!(truncate_label("short"), "short");
}

#[test]
fn duplicate_nodes_and_edges_collapse() {
    let mut graph = DagGraph::default();
    assert!(graph.is_empty());
    let first = graph.add_node(NodeKind::Ec2, "id", "label");
    let second = graph.add_node(NodeKind::S3, "id", "other");
    assert_eq!(first, second);
    assert!(!graph.is_empty());
    assert_eq!(graph.len(), 1);
    // Self-loops are now allowed (UI-03).
    assert!(graph.add_edge(0, 0));
    assert_eq!(graph.edges().len(), 1);
    // But duplicate self-loops are collapsed.
    assert!(!graph.add_edge(0, 0));
    assert_eq!(graph.edges().len(), 1);
    let mut graph = chain();
    assert!(!graph.add_edge(0, 1));
    assert_eq!(graph.edges().len(), 2);
}

#[test]
fn node_cap_truncates_explicitly() {
    let mut graph = DagGraph::default();
    for index in 0..MAX_NODES + 5 {
        graph.add_node(NodeKind::Other, format!("cap{index}"), "cap");
    }
    assert_eq!(graph.len(), MAX_NODES);
    assert_eq!(graph.dropped_nodes(), 5);
}

#[test]
fn edge_cap_drops_exact_overflow_and_layout_stays_acyclic() {
    // Given a dense DAG (all forward pairs over 100 nodes = 4950 edges) that
    // exceeds MAX_EDGES.
    let mut graph = DagGraph::default();
    for index in 0..100 {
        graph.add_node(NodeKind::Other, format!("e{index}"), "e");
    }
    for from in 0..100 {
        for to in from + 1..100 {
            graph.add_overlay_edge(from, to, "rel".to_owned(), String::new());
        }
    }
    // Then admission stops exactly at the cap and the overflow is counted.
    assert_eq!(graph.edges().len(), MAX_EDGES);
    assert_eq!(graph.dropped_edges(), 4950 - MAX_EDGES);
    // And layout consumes only admitted edges, with no cycle-induced back edges.
    let layout = DagLayout::compute(&graph);
    assert_eq!(layout.edges().len(), MAX_EDGES);
    assert_eq!(layout.dropped_edges(), 0);
    assert_eq!(layout.back_edges(), 0);
}

fn chain() -> DagGraph {
    let mut graph = DagGraph::default();
    for index in 0..3 {
        graph.add_node(NodeKind::Ec2, format!("n{index}"), format!("node {index}"));
    }
    graph.add_edge(0, 1);
    graph.add_edge(1, 2);
    graph
}

fn projected_row(account: &str, region: &str, kind: &str, id: &str, key: &str) -> crate::data::Row {
    crate::data::Row {
        account: account.to_owned(),
        region: region.to_owned(),
        kind: kind.to_owned(),
        id: id.to_owned(),
        name: id.to_owned(),
        key: key.to_owned(),
        record_bytes: Some(1),
        content_hash: Some(String::new()),
        tags: Vec::new(),
        tags_truncated: false,
        tag_observation: None,
    }
}

#[test]
fn live_neighbor_projection_uses_full_resource_keys_and_hierarchy() {
    let mut data = crate::data::Data::default();
    let source_row = projected_row(
        "111111111111",
        "us-east-1",
        "AWS::EC2::VPC",
        "vpc-a",
        "key-vpc",
    );
    let target_row = projected_row(
        "111111111111",
        "us-east-1",
        "AWS::EC2::Subnet",
        "subnet-a",
        "key-subnet",
    );
    data.replace_page(vec![source_row.clone(), target_row.clone()], 2);
    data.set_graph(
        crate::data::Graph {
            nodes: vec![source_row, target_row],
            edges: vec![crate::data::Edge {
                from: "key-vpc".to_owned(),
                to: "key-subnet".to_owned(),
                label: "CONTAINS".to_owned(),
                provenance: "collector".to_owned(),
            }],
            truncated: false,
            ..crate::data::Graph::default()
        },
        None,
    );
    let graph = builder::from_data_graph(&data.visual_graph());
    assert_eq!(graph.len(), 6);
    assert_eq!(graph.edges().len(), 6);
}
