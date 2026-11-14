use std::cmp::Reverse;
use std::collections::BinaryHeap;

use super::adjacency::Adjacency;
use super::{MAX_LAYERS, ORDER_PASSES, f32_index};

pub(super) struct CycleBreak {
    pub(super) kept: Vec<bool>,
    pub(super) back: usize,
}

pub(super) fn break_cycles(out: &[Vec<(usize, usize)>], edge_count: usize) -> CycleBreak {
    let mut state = vec![0_u8; out.len()];
    let mut kept = vec![true; edge_count];
    let mut back = 0_usize;
    for start in 0..out.len() {
        if state.get(start).copied().unwrap_or(2) == 0 {
            back = back.saturating_add(dfs(start, out, &mut state, &mut kept));
        }
    }
    CycleBreak { kept, back }
}

fn mark_state(node: usize, state: &mut [u8], value: u8) {
    if let Some(slot) = state.get_mut(node) {
        *slot = value;
    }
}

fn dfs(node: usize, out: &[Vec<(usize, usize)>], state: &mut [u8], kept: &mut [bool]) -> usize {
    mark_state(node, state, 1);
    let mut back = 0_usize;
    if let Some(neighbors) = out.get(node) {
        for &(target, edge) in neighbors {
            match state.get(target).copied().unwrap_or(2) {
                0 => back = back.saturating_add(dfs(target, out, state, kept)),
                1 => {
                    if let Some(flag) = kept.get_mut(edge) {
                        *flag = false;
                    }
                    back = back.saturating_add(1);
                }
                _ => {}
            }
        }
    }
    mark_state(node, state, 2);
    back
}

pub(super) fn assign_layers(
    node_count: usize,
    out: &[Vec<(usize, usize)>],
    kept: &[bool],
) -> Vec<usize> {
    let mut indegree = indegrees(node_count, out, kept);
    let mut layer = vec![0_usize; node_count];
    let mut ready = ready_nodes(&indegree);
    while let Some(Reverse(node)) = ready.pop() {
        release(node, out, kept, &mut indegree, &mut layer, &mut ready);
    }
    layer
}

fn indegrees(node_count: usize, out: &[Vec<(usize, usize)>], kept: &[bool]) -> Vec<usize> {
    let mut indegree = vec![0_usize; node_count];
    for list in out {
        for &(target, edge) in list {
            if kept.get(edge).copied().unwrap_or(false)
                && let Some(slot) = indegree.get_mut(target)
            {
                *slot = slot.saturating_add(1);
            }
        }
    }
    indegree
}

fn ready_nodes(indegree: &[usize]) -> BinaryHeap<Reverse<usize>> {
    let mut ready = BinaryHeap::new();
    for (node, &degree) in indegree.iter().enumerate() {
        if degree == 0 {
            ready.push(Reverse(node));
        }
    }
    ready
}

fn release(
    node: usize,
    out: &[Vec<(usize, usize)>],
    kept: &[bool],
    indegree: &mut [usize],
    layer: &mut [usize],
    ready: &mut BinaryHeap<Reverse<usize>>,
) {
    let Some(neighbors) = out.get(node) else {
        return;
    };
    for &(target, edge) in neighbors {
        if !kept.get(edge).copied().unwrap_or(false) {
            continue;
        }
        relax(node, target, layer);
        if let Some(slot) = indegree.get_mut(target) {
            *slot = slot.saturating_sub(1);
            if *slot == 0 {
                ready.push(Reverse(target));
            }
        }
    }
}

fn relax(from: usize, to: usize, layer: &mut [usize]) {
    let next = layer
        .get(from)
        .copied()
        .unwrap_or(0)
        .saturating_add(1)
        .min(MAX_LAYERS.saturating_sub(1));
    if let Some(slot) = layer.get_mut(to) {
        *slot = (*slot).max(next);
    }
}

pub(super) fn order_layers(
    node_count: usize,
    layer_of: &[usize],
    adj: &Adjacency,
) -> Vec<Vec<usize>> {
    let mut layers = group_layers(layer_of);
    for _ in 0..ORDER_PASSES {
        sweep_down(&mut layers, node_count, adj, layer_of);
        sweep_up(&mut layers, node_count, adj, layer_of);
    }
    layers
}

fn group_layers(layer_of: &[usize]) -> Vec<Vec<usize>> {
    let mut layer_count = 0;
    for &layer in layer_of {
        layer_count = layer_count.max(layer.saturating_add(1));
    }
    let mut layers = vec![Vec::new(); layer_count.max(1)];
    for (node, &layer) in layer_of.iter().enumerate() {
        if let Some(list) = layers.get_mut(layer) {
            list.push(node);
        }
    }
    for slot in &mut layers {
        slot.sort_unstable();
    }
    layers
}

fn sweep_down(layers: &mut [Vec<usize>], node_count: usize, adj: &Adjacency, layer_of: &[usize]) {
    for layer in 1..layers.len() {
        let positions = positions_of(layers, node_count);
        sort_layer(
            layers.get_mut(layer),
            &adj.preds,
            &positions,
            layer_of,
            layer.saturating_sub(1),
        );
    }
}

fn sweep_up(layers: &mut [Vec<usize>], node_count: usize, adj: &Adjacency, layer_of: &[usize]) {
    for layer in (0..layers.len().saturating_sub(1)).rev() {
        let positions = positions_of(layers, node_count);
        sort_layer(
            layers.get_mut(layer),
            &adj.succs,
            &positions,
            layer_of,
            layer.saturating_add(1),
        );
    }
}

fn positions_of(layers: &[Vec<usize>], node_count: usize) -> Vec<usize> {
    let mut positions = vec![0_usize; node_count];
    for layer in layers {
        for (position, &node) in layer.iter().enumerate() {
            if let Some(slot) = positions.get_mut(node) {
                *slot = position;
            }
        }
    }
    positions
}

fn sort_layer(
    layer: Option<&mut Vec<usize>>,
    neighbors: &[Vec<usize>],
    positions: &[usize],
    layer_of: &[usize],
    target_layer: usize,
) {
    let Some(layer) = layer else {
        return;
    };
    layer.sort_by(|&a, &b| {
        let key_a = barycenter(a, neighbors, positions, layer_of, target_layer);
        let key_b = barycenter(b, neighbors, positions, layer_of, target_layer);
        key_a.total_cmp(&key_b).then_with(|| a.cmp(&b))
    });
}

fn barycenter(
    node: usize,
    neighbors: &[Vec<usize>],
    positions: &[usize],
    layer_of: &[usize],
    target_layer: usize,
) -> f32 {
    let (sum, count) = neighbor_sum(node, neighbors, positions, layer_of, target_layer);
    if count == 0 {
        f32_index(positions.get(node).copied().unwrap_or(0))
    } else {
        sum / f32_index(count)
    }
}

fn neighbor_sum(
    node: usize,
    neighbors: &[Vec<usize>],
    positions: &[usize],
    layer_of: &[usize],
    target_layer: usize,
) -> (f32, usize) {
    let mut sum = 0.0;
    let mut count = 0_usize;
    if let Some(list) = neighbors.get(node) {
        for &neighbor in list {
            if layer_of.get(neighbor).copied() != Some(target_layer) {
                continue;
            }
            if let Some(&position) = positions.get(neighbor) {
                sum += f32_index(position);
                count = count.saturating_add(1);
            }
        }
    }
    (sum, count)
}
