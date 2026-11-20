//! Proptest-based property tests for DagLayout::compute.
//!
//! Generates random graphs and checks layout invariants. All property tests use
//! ProptestConfig with a fixed case count (128) and no failure persistence, so
//! runs are deterministic and leave no regression files on disk.
//!
//! Properties checked:
//!   1. Node accounting: every placed node has a rect; dropped count is correct.
//!   2. Non-overlapping: no two placed rects overlap (touching allowed).
//!   3. Bounds: every placed rect lies within the layout's width/height.
//!   4. Determinism: compute is pure (same graph gives same layout).
//!   5. Edge consistency: from != to, back edges go backward or same layer.
//!   6. Layer bounds: layer index is < MAX_LAYERS for every placed node.
//!   7. Empty graph: layout handles zero nodes without panicking.
//!   8. Cap boundary: a graph filled to MAX_LAYOUT_NODES places every node.

use proptest::prelude::*;

use super::{DagLayout, LayoutRect, MAX_LAYERS, MAX_LAYOUT_NODES};
use crate::dag::graph::{DagGraph, NodeKind};

/// Fixed test strategy: run 128 cases, never save regression files.
fn config() -> ProptestConfig {
    ProptestConfig {
        cases: 128,
        max_shrink_iters: 1000,
        failure_persistence: None,
        ..ProptestConfig::default()
    }
}

/// Build a DagGraph from node count and edge pairs.
/// Node kinds cycle through Ec2, S3, Lambda.
fn build_graph(node_count: usize, edges: Vec<(usize, usize)>) -> DagGraph {
    let mut graph = DagGraph::default();
    for index in 0..node_count {
        let kind = match index % 3 {
            0 => NodeKind::Ec2,
            1 => NodeKind::S3,
            _ => NodeKind::Lambda,
        };
        graph.add_node(kind, format!("n{index}"), format!("node {index}"));
    }
    for (from, to) in edges {
        if from < node_count && to < node_count {
            let _ = graph.add_edge(from, to);
        }
    }
    graph
}

#[derive(Debug, Clone)]
struct GraphSpec {
    node_count: usize,
    edges: Vec<(usize, usize)>,
}

/// Strategy that generates a graph description: (node_count, edges).
fn graph_strategy(max_nodes: usize, max_edges: usize) -> impl Strategy<Value = GraphSpec> {
    (0..=max_nodes).prop_flat_map(move |node_count| {
        let edge_limit = max_edges.min(node_count.saturating_mul(2));
        let edge_pairs = (0..node_count)
            .prop_flat_map(move |from| (0..node_count).prop_map(move |to| (from, to)));
        prop::collection::vec(edge_pairs, 0..=edge_limit)
            .prop_map(move |edges| GraphSpec { node_count, edges })
    })
}

proptest! {
    #![proptest_config(config())]

    #[test]
    fn node_accounting_is_consistent(spec in graph_strategy(300, 600)) {
        let graph = build_graph(spec.node_count, spec.edges);
        let layout = DagLayout::compute(&graph);

        let node_count_actual = graph.nodes().len().min(MAX_LAYOUT_NODES);

        for index in 0..node_count_actual {
            prop_assert!(layout.rect(index).is_some(),
                "node {} not dropped but has no rect", index);
        }

        let expected_dropped = graph.nodes().len().saturating_sub(MAX_LAYOUT_NODES);
        prop_assert_eq!(layout.dropped_nodes(), expected_dropped,
            "dropped nodes mismatch: got {}, expected {}",
            layout.dropped_nodes(), expected_dropped);

        for index in node_count_actual..graph.nodes().len() {
            prop_assert!(layout.rect(index).is_none(),
                "dropped node {} has a rect", index);
        }
    }

    #[test]
    fn placed_rects_do_not_overlap(spec in graph_strategy(200, 400)) {
        let graph = build_graph(spec.node_count, spec.edges);
        let layout = DagLayout::compute(&graph);

        let placed: Vec<(usize, LayoutRect)> = (0..graph.nodes().len().min(MAX_LAYOUT_NODES))
            .filter_map(|i| layout.rect(i).map(|r| (i, r)))
            .collect();

        for (k, (i, ra)) in placed.iter().enumerate() {
            for (_, (j, rb)) in placed.iter().enumerate().skip(k.saturating_add(1)) {
                let overlap = ra.x < rb.x + rb.width && rb.x < ra.x + ra.width
                    && ra.y < rb.y + rb.height && rb.y < ra.y + ra.height;
                prop_assert!(!overlap,
                    "rects {}/{} overlap: {:?} vs {:?}", i, j, ra, rb);
            }
        }
    }

    #[test]
    fn rects_fit_within_layout_bounds(spec in graph_strategy(200, 400)) {
        let graph = build_graph(spec.node_count, spec.edges);
        let layout = DagLayout::compute(&graph);

        for index in 0..graph.nodes().len().min(MAX_LAYOUT_NODES) {
            if let Some(rect) = layout.rect(index) {
                prop_assert!(rect.x >= 0.0, "node {}: rect.x negative", index);
                prop_assert!(rect.y >= 0.0, "node {}: rect.y negative", index);
                prop_assert!(rect.width > 0.0, "node {}: rect.width not positive", index);
                prop_assert!(rect.height > 0.0, "node {}: rect.height not positive", index);
                prop_assert!(rect.x + rect.width <= layout.width() + 0.001,
                    "node {}: rect extends past width", index);
                prop_assert!(rect.y + rect.height <= layout.height() + 0.001,
                    "node {}: rect extends past height", index);
            }
        }
    }

    #[test]
    fn compute_is_deterministic(spec in graph_strategy(100, 200)) {
        let graph = build_graph(spec.node_count, spec.edges);

        let layout1 = DagLayout::compute(&graph);
        let layout2 = DagLayout::compute(&graph);

        prop_assert_eq!(layout1, layout2, "compute is not deterministic");
    }

    #[test]
    fn edge_consistency_holds(spec in graph_strategy(100, 200)) {
        let graph = build_graph(spec.node_count, spec.edges);
        let layout = DagLayout::compute(&graph);

        let node_count_actual = graph.nodes().len().min(MAX_LAYOUT_NODES);

        for edge in layout.edges() {
            prop_assert!(edge.from < node_count_actual,
                "edge.from {} out of range", edge.from);
            prop_assert!(edge.to < node_count_actual,
                "edge.to {} out of range", edge.to);

            if edge.from == edge.to {
                prop_assert!(edge.back, "self-loop is not marked back");
            }

            if let (Some(from_layer), Some(to_layer)) =
                (layout.layer(edge.from), layout.layer(edge.to))
            {
                if edge.back {
                    prop_assert!(from_layer >= to_layer,
                        "back edge from {} (layer {}) to {} (layer {})",
                        edge.from, from_layer, edge.to, to_layer);
                } else {
                    prop_assert!(from_layer <= to_layer,
                        "forward edge from {} (layer {}) to {} (layer {})",
                        edge.from, from_layer, edge.to, to_layer);
                }
            }
        }

        prop_assert!(layout.back_edges() <= layout.edges().len(),
            "back edge count exceeds edge count");
    }

    #[test]
    fn layer_indices_are_bounded(spec in graph_strategy(100, 200)) {
        let graph = build_graph(spec.node_count, spec.edges);
        let layout = DagLayout::compute(&graph);

        for index in 0..graph.nodes().len().min(MAX_LAYOUT_NODES) {
            if let Some(layer) = layout.layer(index) {
                prop_assert!(layer < MAX_LAYERS,
                    "node {} at layer {}, exceeds MAX_LAYERS={}", index, layer, MAX_LAYERS);
            }
        }
    }

    #[test]
    fn empty_graph_layout_is_valid(_dummy in Just(())) {
        let graph = DagGraph::default();
        let layout = DagLayout::compute(&graph);

        prop_assert_eq!(layout.node_count(), 0);
        prop_assert_eq!(layout.dropped_nodes(), 0);
        prop_assert_eq!(layout.dropped_edges(), 0);
        prop_assert_eq!(layout.edges().len(), 0);
        prop_assert_eq!(layout.back_edges(), 0);
        prop_assert!(layout.width() >= 0.0);
        prop_assert!(layout.height() >= 0.0);
    }

    #[test]
    fn graph_at_node_cap_places_every_node(_dummy in Just(())) {
        // DagGraph refuses nodes past MAX_NODES, which equals MAX_LAYOUT_NODES, so
        // the drop path is unreachable from a DagGraph. Check the boundary instead:
        // a graph filled to the cap places every node and drops none.
        let edges: Vec<(usize, usize)> = (0..99_usize).map(|i| (i, i.saturating_add(1))).collect();
        let graph = build_graph(MAX_LAYOUT_NODES, edges);
        let layout = DagLayout::compute(&graph);

        prop_assert_eq!(graph.nodes().len(), MAX_LAYOUT_NODES);
        prop_assert_eq!(layout.dropped_nodes(), 0);
        for index in 0..MAX_LAYOUT_NODES {
            prop_assert!(layout.rect(index).is_some(), "node {} has no rect", index);
        }
    }
}
