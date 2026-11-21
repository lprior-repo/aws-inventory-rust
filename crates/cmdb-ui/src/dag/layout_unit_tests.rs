use super::graph::{DagGraph, NodeKind};
use super::layout::{
    DagLayout, H_GAP, NODE_W, PADDING, MAX_LAYERS,
};

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
fn cycle_break_flags_correct_back_edge() {
    let layout = DagLayout::compute(&cycle());
    let back_edges: Vec<_> = layout.edges()
        .iter()
        .filter(|e| e.back)
        .collect();
    assert_eq!(back_edges.len(), 1);
    assert_eq!(back_edges[0].from, 2);
    assert_eq!(back_edges[0].to, 0);
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
            for b in a + 1..3 {
                assert!(layout.rect(b).is_some(), "missing rect");
                if let Some(rect_b) = layout.rect(b) {
                    assert!(!rect_a.intersects(rect_b));
                }
            }
        }
    }
}

#[test]
fn long_chain_clamps_layers() {
    let node_count = MAX_LAYERS + 5;
    let mut graph = DagGraph::default();
    for index in 0..node_count {
        graph.add_node(NodeKind::Ec2, format!("n{index}"), format!("node {index}"));
    }
    for index in 0..node_count - 1 {
        graph.add_edge(index, index + 1);
    }

    let layout = DagLayout::compute(&graph);

    for index in 0..node_count {
        let layer = layout.layer(index).expect("layer should be assigned");
        assert!(layer < MAX_LAYERS, "node {} at layer {} >= MAX_LAYERS", index, layer);
    }

    for index in 0..64 {
        assert_eq!(layout.layer(index), Some(index));
    }

    for index in 64..node_count {
        assert_eq!(layout.layer(index), Some(63),
            "node {} should be clamped to layer 63", index);
    }

    let expected_width = PADDING * 2.0
        + f32::from(MAX_LAYERS as i16) * (NODE_W + H_GAP)
        - H_GAP;
    assert!((layout.width() - expected_width).abs() < 0.001,
        "width mismatch: got {}, expected {}", layout.width(), expected_width);

    assert_eq!(layout.node_count(), node_count);

    let placed: Vec<_> = (0..node_count)
        .filter_map(|i| layout.rect(i).map(|r| (i, r)))
        .collect();
    for (k, (i, ra)) in placed.iter().enumerate() {
        for (_, (j, rb)) in placed.iter().enumerate().skip(k + 1) {
            let overlap = ra.x < rb.x + rb.width && rb.x < ra.x + ra.width
                && ra.y < rb.y + rb.height && rb.y < ra.y + ra.height;
            assert!(!overlap, "rects {}/{} overlap", i, j);
        }
    }
}

#[test]
fn x_positions_are_layer_aligned() {
    let layout = DagLayout::compute(&chain());
    for index in 0..3 {
        let layer = layout.layer(index).expect("layer assigned");
        let rect = layout.rect(index).expect("rect assigned");
        let expected_x = PADDING + f32::from(layer as i16) * (NODE_W + H_GAP);
        assert!((rect.x - expected_x).abs() < 0.001,
            "node {} x mismatch: got {}, expected {}", index, rect.x, expected_x);
    }
}

#[test]
fn forward_edges_go_right() {
    let layout = DagLayout::compute(&chain());
    for edge in layout.edges() {
        if !edge.back {
            let from_rect = layout.rect(edge.from).expect("from rect");
            let to_rect = layout.rect(edge.to).expect("to rect");
            assert!(from_rect.right() < to_rect.x,
                "forward edge {}->{} should go right", edge.from, edge.to);
        }
    }
}

#[test]
fn diamond_shapes_layer_correctly() {
    let mut graph = DagGraph::default();
    for index in 0..3 {
        graph.add_node(NodeKind::Ec2, format!("n{index}"), format!("node {index}"));
    }
    graph.add_edge(0, 1);
    graph.add_edge(1, 2);
    graph.add_edge(0, 2);

    let layout = DagLayout::compute(&graph);
    assert_eq!(layout.layer(0), Some(0));
    assert_eq!(layout.layer(1), Some(1));
    assert_eq!(layout.layer(2), Some(2),
        "node 2 should be at layer 2 due to path 0->1->2");
}

#[test]
fn overlay_edges_do_not_affect_layout() {
    let mut graph = DagGraph::default();
    for index in 0..3 {
        graph.add_node(NodeKind::Ec2, format!("n{index}"), format!("node {index}"));
    }
    graph.add_edge(0, 1);
    graph.add_edge(1, 2);
    let layout_base = DagLayout::compute(&graph);
    graph.add_overlay_edge(0, 2, "references".to_string(), "reference".to_string());
    let layout_overlay = DagLayout::compute(&graph);
    for index in 0..3 {
        assert_eq!(layout_base.layer(index), layout_overlay.layer(index));
        assert_eq!(layout_base.rect(index), layout_overlay.rect(index));
    }

    let overlay_edges: Vec<_> = layout_overlay.edges()
        .iter()
        .filter(|e| e.overlay && !e.back)
        .collect();
    assert_eq!(overlay_edges.len(), 1);
    assert_eq!(overlay_edges[0].from, 0);
    assert_eq!(overlay_edges[0].to, 2);
}

#[test]
fn self_loop_is_back_edge() {
    let mut graph = DagGraph::default();
    graph.add_node(NodeKind::Ec2, "self", "self");
    graph.add_edge(0, 0);

    let layout = DagLayout::compute(&graph);
    assert_eq!(layout.back_edges(), 1);
    assert_eq!(layout.layer(0), Some(0));

    let edge = layout.edges().first().expect("edge exists");
    assert!(edge.back);
    assert_eq!(edge.from, 0);
    assert_eq!(edge.to, 0);
}

#[test]
fn placed_with_node_cut() {
    let mut graph = DagGraph::default();
    for index in 0..5 {
        graph.add_node(NodeKind::Ec2, format!("n{index}"), format!("node {index}"));
    }
    graph.add_edge(0, 1);
    graph.add_edge(1, 2);
    graph.add_edge(2, 3);
    graph.add_edge(3, 4);

    let layout = DagLayout::placed(&graph, 3, 2);

    assert_eq!(layout.dropped_nodes(), 2);
    assert_eq!(layout.node_count(), 3);

    for index in 0..3 {
        assert!(layout.rect(index).is_some(), "node {} should be placed", index);
    }
    assert!(layout.rect(3).is_none(), "node 3 should be dropped");
    assert!(layout.rect(4).is_none(), "node 4 should be dropped");

    assert_eq!(layout.dropped_edges(), 2);
    assert_eq!(layout.edges().len(), 2);
}

#[test]
fn order_is_permutation_of_nodes() {
    let graph = chain();
    let layout = DagLayout::compute(&graph);
    let mut order = layout.order().to_vec();

    order.sort();
    let expected: Vec<usize> = (0..graph.nodes().len()).collect();
    assert_eq!(order, expected);
}

#[test]
fn hit_test_finds_correct_node() {
    let graph = chain();
    let layout = DagLayout::compute(&graph);

    for index in 0..3 {
        let rect = layout.rect(index).expect("rect exists");
        let center_x = rect.x + rect.width * 0.5;
        let center_y = rect.y + rect.height * 0.5;
        assert_eq!(layout.hit_test(center_x, center_y), Some(index),
            "hit test failed for node {}", index);
    }
}

#[test]
fn crossing_edges_order_nodes_to_reduce_crossings() {
    let mut graph = DagGraph::default();
    for index in 0..4 {
        graph.add_node(NodeKind::Ec2, format!("n{index}"), format!("node {index}"));
    }
    graph.add_edge(0, 2);
    graph.add_edge(0, 3);
    graph.add_edge(1, 2);
    graph.add_edge(1, 3);

    let layout = DagLayout::compute(&graph);
    let rect2 = layout.rect(2).expect("rect for 2");
    let rect3 = layout.rect(3).expect("rect for 3");
    assert!(rect3.y > rect2.y,
        "node 3 should be below node 2 to reduce crossings");
}