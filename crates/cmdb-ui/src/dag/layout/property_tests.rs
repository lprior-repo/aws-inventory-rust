use proptest::prelude::*;
use proptest::test_runner::{Config, RngSeed};

use super::{DagLayout, LayoutRect, MAX_LAYERS, MAX_LAYOUT_NODES};
use crate::dag::graph::{DagGraph, NodeKind};

fn config() -> Config {
    Config {
        cases: 128,
        max_shrink_iters: 1000,
        rng_seed: RngSeed::Fixed(0x4C41_594F_5554_5453),
        ..Config::default()
    }
}

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

fn graph_strategy(max_nodes: usize, max_edges: usize) -> impl Strategy<Value = GraphSpec> {
    (0..=max_nodes).prop_flat_map(move |node_count| {
        let edge_limit = max_edges.min(node_count.saturating_mul(2));
        let edge_pairs = (0..node_count)
            .prop_flat_map(move |from| (0..node_count).prop_map(move |to| (from, to)));
        prop::collection::vec(edge_pairs, 0..=edge_limit)
            .prop_map(move |edges| GraphSpec { node_count, edges })
    })
}

fn dag_strategy(max_nodes: usize, max_edges: usize) -> impl Strategy<Value = GraphSpec> {
    (0..=max_nodes).prop_flat_map(move |node_count| {
        let edge_limit = max_edges.min(node_count.saturating_mul(2));
        let edge_pairs = (0..node_count)
            .prop_flat_map(move |a| (0..node_count).prop_map(move |b| (a, b)))
            .prop_filter_map("a != b", |(a, b)| if a == b { None } else { Some((a, b)) })
            .prop_map(|(a, b)| if a < b { (a, b) } else { (b, a) });
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

        let node_count_actual = graph.nodes().len().min(MAX_LAYOUT_NODES);
        let mut max_right = 0.0_f32;

        for index in 0..node_count_actual {
            if let Some(rect) = layout.rect(index) {
                prop_assert!(rect.x >= super::PADDING,
                    "node {}: rect.x {} less than PADDING", index, rect.x);
                prop_assert!(rect.y >= super::PADDING,
                    "node {}: rect.y {} less than PADDING", index, rect.y);
                prop_assert!(rect.width > 0.0,
                    "node {}: rect.width not positive", index);
                prop_assert!(rect.height > 0.0,
                    "node {}: rect.height not positive", index);
                prop_assert!(rect.x + rect.width <= layout.width() + 0.001,
                    "node {}: rect extends past width", index);
                prop_assert!(rect.y + rect.height <= layout.height() + 0.001,
                    "node {}: rect extends past height", index);
                if rect.right() > max_right {
                    max_right = rect.right();
                }
            }
        }

        prop_assert!(
            layout.width() >= max_right + super::PADDING - 0.001
                && layout.width() <= max_right + super::PADDING + 0.001,
            "width mismatch: layout {} vs max_right {} + PADDING",
            layout.width(), max_right
        );
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
        let mut back_count = 0_usize;

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
                    let clamped = to_layer == MAX_LAYERS.saturating_sub(1);
                    prop_assert!(from_layer < to_layer || clamped,
                        "forward edge from {} (layer {}) to {} (layer {})",
                        edge.from, from_layer, edge.to, to_layer);
                }
            }

            if edge.back {
                back_count = back_count.saturating_add(1);
            }
        }

        prop_assert_eq!(layout.back_edges(), back_count,
            "back_edges() mismatch: layout says {}, counted {}",
            layout.back_edges(), back_count);
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
                    "node {} at layer {}, exceeds MAX_LAYERS={}",
                    index, layer, MAX_LAYERS);
            }
        }
    }

    #[test]
    fn dag_only_graphs_have_no_back_edges(spec in dag_strategy(100, 200)) {
        let graph = build_graph(spec.node_count, spec.edges);
        let layout = DagLayout::compute(&graph);
        prop_assert_eq!(layout.back_edges(), 0,
            "DAG graph should have no back edges, got {}", layout.back_edges());
    }

    #[test]
    fn edge_count_matches_graph_edges(spec in graph_strategy(100, 200)) {
        let graph = build_graph(spec.node_count, spec.edges);
        let layout = DagLayout::compute(&graph);

        let node_count_actual = graph.nodes().len().min(MAX_LAYOUT_NODES);
        let dropped = graph.edges().iter().filter(|e| {
            e.from >= node_count_actual || e.to >= node_count_actual
        }).count();

        prop_assert_eq!(layout.edges().len().saturating_add(layout.dropped_edges()),
            graph.edges().len(),
            "edge count mismatch: layout {} + dropped {} != graph {}",
            layout.edges().len(), layout.dropped_edges(), graph.edges().len());
        prop_assert_eq!(layout.dropped_edges(), dropped,
            "dropped_edges() mismatch: layout {} vs computed {}",
            layout.dropped_edges(), dropped);
    }
}

#[test]
fn empty_graph_layout_is_valid() {
    let graph = DagGraph::default();
    let layout = DagLayout::compute(&graph);

    assert_eq!(layout.node_count(), 0);
    assert_eq!(layout.dropped_nodes(), 0);
    assert_eq!(layout.dropped_edges(), 0);
    assert_eq!(layout.edges().len(), 0);
    assert_eq!(layout.back_edges(), 0);
    assert!(layout.width() >= 0.0);
    assert!(layout.height() >= 0.0);
}

#[test]
fn graph_at_node_cap_places_every_node() {
    let edges: Vec<(usize, usize)> = (0..99).map(|i| (i, i + 1)).collect();
    let graph = build_graph(MAX_LAYOUT_NODES, edges);
    let layout = DagLayout::compute(&graph);

    assert_eq!(graph.nodes().len(), MAX_LAYOUT_NODES);
    assert_eq!(layout.dropped_nodes(), 0);
    for index in 0..MAX_LAYOUT_NODES {
        assert!(layout.rect(index).is_some(), "node {} has no rect", index);
    }
}
