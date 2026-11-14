use makepad_widgets::{Rect, dvec2};

use super::builder;
use super::graph::{DagGraph, MAX_LABEL_CHARS, MAX_NODES, NodeKind, truncate_label};
use super::layout::DagLayout;
use super::view::{MAX_SCALE, MIN_SCALE, ViewTransform};
use crate::data::Data;

fn chain() -> DagGraph {
    let mut graph = DagGraph::default();
    for index in 0..3 {
        graph.add_node(NodeKind::Ec2, format!("n{index}"), format!("node {index}"));
    }
    graph.add_edge(0, 1);
    graph.add_edge(1, 2);
    graph
}

fn cycle() -> DagGraph {
    let mut graph = DagGraph::default();
    for index in 0..3 {
        graph.add_node(NodeKind::S3, format!("c{index}"), format!("cycle {index}"));
    }
    graph.add_edge(0, 1);
    graph.add_edge(1, 2);
    graph.add_edge(2, 0);
    graph
}

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
    let first = graph.add_node(NodeKind::Ec2, "id", "label");
    let second = graph.add_node(NodeKind::S3, "id", "other");
    assert_eq!(first, second);
    assert_eq!(graph.len(), 1);
    assert!(!graph.add_edge(0, 0));
    assert_eq!(graph.edges().len(), 0);
    let mut graph = chain();
    assert!(!graph.add_edge(0, 1));
    assert_eq!(graph.edges().len(), 2);
}

#[test]
fn longest_path_layers() {
    let layout = DagLayout::compute(&chain());
    assert_eq!(layout.layer(0), Some(0));
    assert_eq!(layout.layer(1), Some(1));
    assert_eq!(layout.layer(2), Some(2));
    assert_eq!(layout.back_edges(), 0);
    let first = layout.rect(0).map(|rect| rect.x).unwrap_or(0.0);
    let last = layout.rect(2).map(|rect| rect.x).unwrap_or(0.0);
    assert!(last > first);
}

#[test]
fn cycle_break_is_deterministic() {
    let first = DagLayout::compute(&cycle());
    let second = DagLayout::compute(&cycle());
    assert_eq!(first, second);
    assert_eq!(first.back_edges(), 1);
    assert_eq!(first.edges().len(), 3);
    assert_eq!(first.layer(0), Some(0));
    assert_eq!(first.layer(1), Some(1));
    assert_eq!(first.layer(2), Some(2));
}

#[test]
fn disconnected_nodes_and_components_are_placed() {
    let mut graph = DagGraph::default();
    for index in 0..5 {
        graph.add_node(NodeKind::Lambda, format!("d{index}"), format!("d{index}"));
    }
    graph.add_edge(0, 1);
    graph.add_edge(2, 3);
    let layout = DagLayout::compute(&graph);
    assert_eq!(layout.node_count(), 5);
    assert!(layout.width() > 0.0);
    assert!(layout.height() > 0.0);
    for index in 0..5 {
        assert!(layout.rect(index).is_some(), "missing rect");
        if let Some(rect) = layout.rect(index) {
            assert!(rect.x >= 0.0 && rect.y >= 0.0);
            assert!(rect.right() <= layout.width());
            assert!(rect.y + rect.height <= layout.height());
        }
    }
}

#[test]
fn rectangles_do_not_overlap_and_are_bounded() {
    let layout = DagLayout::compute(&chain());
    for a in 0..3 {
        assert!(layout.rect(a).is_some(), "missing rect");
        if let Some(rect_a) = layout.rect(a) {
            for b in a.saturating_add(1)..3 {
                assert!(layout.rect(b).is_some(), "missing rect");
                if let Some(rect_b) = layout.rect(b) {
                    assert!(!rect_a.intersects(rect_b));
                }
            }
        }
    }
}

#[test]
fn node_cap_truncates_explicitly() {
    let mut graph = DagGraph::default();
    for index in 0..MAX_NODES.saturating_add(5) {
        graph.add_node(NodeKind::Other, format!("cap{index}"), "cap");
    }
    assert_eq!(graph.len(), MAX_NODES);
    assert_eq!(graph.dropped_nodes(), 5);
    let layout = DagLayout::compute(&graph);
    assert_eq!(layout.node_count(), MAX_NODES);
}

#[test]
fn fixture_builder_matches_the_projection() {
    let data = Data::synthetic();
    let source = data.graph();
    let graph = builder::from_data_graph(&source);

    assert_eq!(graph.len(), 56);

    assert_eq!(graph.edges().len(), 69);
    assert_eq!(graph.dropped_nodes(), 0);
    assert_eq!(graph.dropped_edges(), 0);
    let mut ids = std::collections::BTreeSet::new();
    for index in 0..graph.len() {
        assert!(graph.node(index).is_some(), "missing node");
        if let Some(node) = graph.node(index) {
            assert!(ids.insert(node.id().to_owned()));
            assert!(node.label().chars().count() <= MAX_LABEL_CHARS);
        }
    }
    let layout = DagLayout::compute(&graph);
    assert_eq!(layout.node_count(), 56);
    assert_eq!(layout.back_edges(), 0);
    let account = graph.index_of("account:prod");
    let region = graph.index_of("region:prod/us-east-1");
    let kind = graph.index_of("kind:prod/us-east-1/ec2");
    let resource = graph.index_of("i-0a1b2c3d4e5f60001");
    assert_eq!(account.and_then(|i| layout.layer(i)), Some(0));
    assert_eq!(region.and_then(|i| layout.layer(i)), Some(1));
    assert_eq!(kind.and_then(|i| layout.layer(i)), Some(2));
    assert_eq!(resource.and_then(|i| layout.layer(i)), Some(3));
    for row in &source.nodes {
        let layer = graph.index_of(row.id).and_then(|i| layout.layer(i));
        assert_eq!(layer, Some(3));
    }
}

#[test]
fn filtered_projection_keeps_only_visible_endpoints() {
    let mut data = Data::synthetic();
    data.set_account(Some("prod"));
    let graph = builder::from_data_graph(&data.graph());

    assert_eq!(graph.len(), 20);

    assert_eq!(graph.edges().len(), 26);
    assert_eq!(DagLayout::compute(&graph).node_count(), 20);
}

#[test]
fn fit_centres_and_zoom_keeps_the_anchor() {
    let rect = Rect {
        pos: dvec2(10.0, 20.0),
        size: dvec2(800.0, 600.0),
    };
    let fitted = ViewTransform::fit(rect, 400.0, 300.0);
    assert!(fitted.scale >= MIN_SCALE && fitted.scale <= MAX_SCALE);
    assert!(fitted.offset.x > 0.0 && fitted.offset.y > 0.0);

    let anchor = dvec2(300.0, 250.0);
    let before = fitted.to_world(rect, anchor);
    let zoomed = fitted.zoomed(2.0, anchor, rect);
    let after = zoomed.to_world(rect, anchor);
    assert!((before.0 - after.0).abs() < 0.5);
    assert!((before.1 - after.1).abs() < 0.5);

    let panned = zoomed.panned(dvec2(15.0, -10.0));
    assert!((panned.offset.x - zoomed.offset.x - 15.0).abs() < f64::EPSILON);
}
