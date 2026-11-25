mod adjacency;
mod order;
mod place;

use crate::dag::graph::DagGraph;

pub const MAX_LAYOUT_NODES: usize = 2_000;

pub const MAX_LAYERS: usize = 64;

const _: () = assert!(MAX_LAYOUT_NODES <= 65_535);
const _: () = assert!(MAX_LAYERS <= 65_535);

const ORDER_PASSES: usize = 4;

pub const NODE_W: f32 = 160.0;
pub const NODE_H: f32 = 32.0;
pub const H_GAP: f32 = 72.0;
pub const V_GAP: f32 = 8.0;
pub const PADDING: f32 = 20.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LayoutRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl LayoutRect {
    pub fn right(self) -> f32 {
        self.x + self.width
    }

    pub fn center_y(self) -> f32 {
        self.y + self.height * 0.5
    }

    pub fn contains(self, x: f32, y: f32) -> bool {
        x >= self.x && x <= self.right() && y >= self.y && y <= self.y + self.height
    }

    pub fn intersects(self, other: Self) -> bool {
        self.x <= other.right()
            && other.x <= self.right()
            && self.y <= other.y + other.height
            && other.y <= self.y + self.height
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LayoutEdge {
    pub from: usize,
    pub to: usize,
    pub back: bool,
    pub overlay: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DagLayout {
    rects: Vec<Option<LayoutRect>>,
    edges: Vec<LayoutEdge>,
    order: Vec<usize>,
    layers: Vec<usize>,
    back_edges: usize,
    dropped_nodes: usize,
    dropped_edges: usize,
    width: f32,
    height: f32,
}

impl DagLayout {
    pub fn compute(graph: &DagGraph) -> Self {
        let node_count = graph.nodes().len().min(MAX_LAYOUT_NODES);
        let dropped_nodes = graph.nodes().len().saturating_sub(node_count);
        if node_count == 0 {
            return Self {
                dropped_nodes,
                ..Self::default()
            };
        }
        Self::placed(graph, node_count, dropped_nodes)
    }

    pub fn placed(graph: &DagGraph, node_count: usize, dropped_nodes: usize) -> Self {
        let adj = adjacency::Adjacency::build(graph, node_count);
        let broken = order::break_cycles(&adj.out, graph.edges().len());
        let (edges, dropped_edges) = collect_edges(graph, node_count, &broken.kept);
        let layer_of = order::assign_layers(node_count, &adj.out, &broken.kept);
        let layers = order::order_layers(node_count, &layer_of, &adj);
        let (rects, width, height, order) = place::place(&layers, node_count);
        Self {
            rects,
            edges,
            order,
            layers: layer_of,
            back_edges: broken.back,
            dropped_nodes,
            dropped_edges,
            width,
            height,
        }
    }

    pub fn rect(&self, index: usize) -> Option<LayoutRect> {
        self.rects.get(index).copied().flatten()
    }

    pub fn edges(&self) -> &[LayoutEdge] {
        &self.edges
    }

    pub fn order(&self) -> &[usize] {
        &self.order
    }

    #[cfg(test)]
    pub fn layer(&self, index: usize) -> Option<usize> {
        self.layers.get(index).copied()
    }

    pub fn node_count(&self) -> usize {
        self.rects.iter().filter(|slot| slot.is_some()).count()
    }

    pub fn back_edges(&self) -> usize {
        self.back_edges
    }

    pub fn dropped_nodes(&self) -> usize {
        self.dropped_nodes
    }

    pub fn dropped_edges(&self) -> usize {
        self.dropped_edges
    }

    pub fn width(&self) -> f32 {
        self.width
    }

    pub fn height(&self) -> f32 {
        self.height
    }

    pub fn hit_test(&self, x: f32, y: f32) -> Option<usize> {
        self.order
            .iter()
            .rev()
            .copied()
            .find(|&index| self.rect(index).is_some_and(|rect| rect.contains(x, y)))
    }
}

fn collect_edges(graph: &DagGraph, node_count: usize, kept: &[bool]) -> (Vec<LayoutEdge>, usize) {
    let mut edges = Vec::new();
    let mut dropped = 0_usize;
    for (index, edge) in graph.edges().iter().enumerate() {
        if edge.from >= node_count || edge.to >= node_count {
            dropped = dropped.saturating_add(1);
            continue;
        }
        if let Some(&keep) = kept.get(index) {
            edges.push(LayoutEdge {
                from: edge.from,
                to: edge.to,
                back: !keep,
                overlay: edge.overlay,
            });
        }
    }
    (edges, dropped)
}

fn f32_index(index: usize) -> f32 {
    f32::from(u16::try_from(index).unwrap_or(u16::MAX))
}

#[cfg(test)]
#[path = "property_tests.rs"]
mod property_tests;
