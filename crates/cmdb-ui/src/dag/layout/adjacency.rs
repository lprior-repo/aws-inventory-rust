//! Edge adjacency used by cycle breaking, layering and ordering.
//!
//! Overlay edges are deliberately absent: they are drawn as same-tier data
//! flow and must not stretch the containment layers.

use crate::dag::graph::{DagEdge, DagGraph};

pub(super) struct Adjacency {
    pub(super) out: Vec<Vec<(usize, usize)>>,
    pub(super) preds: Vec<Vec<usize>>,
    pub(super) succs: Vec<Vec<usize>>,
}

impl Adjacency {
    pub(super) fn build(graph: &DagGraph, node_count: usize) -> Self {
        let mut adj = Self {
            out: vec![Vec::new(); node_count],
            preds: vec![Vec::new(); node_count],
            succs: vec![Vec::new(); node_count],
        };
        for (edge_index, edge) in graph.edges().iter().enumerate() {
            adj.push(edge, edge_index, node_count);
        }
        adj.sort();
        adj
    }

    fn push(&mut self, edge: &DagEdge, edge_index: usize, node_count: usize) {
        if edge.overlay || edge.from >= node_count || edge.to >= node_count {
            return;
        }
        if let Some(list) = self.out.get_mut(edge.from) {
            list.push((edge.to, edge_index));
        }
        if let Some(list) = self.preds.get_mut(edge.to) {
            list.push(edge.from);
        }
        if let Some(list) = self.succs.get_mut(edge.from) {
            list.push(edge.to);
        }
    }

    fn sort(&mut self) {
        for list in &mut self.out {
            list.sort_unstable();
        }
        for list in &mut self.preds {
            list.sort_unstable();
            list.dedup();
        }
        for list in &mut self.succs {
            list.sort_unstable();
            list.dedup();
        }
    }
}
