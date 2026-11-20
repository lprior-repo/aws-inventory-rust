//! Benchmarks for DagLayout::compute over generated graph shapes.
//!
//! Measures the pure layout computation pass (graph construction + layout
//! solve) across four shapes at three sizes: chain, wide fan-out, layered
//! random DAG, and cyclic graph. Graph sizes: 100, 500, 2000 (MAX_LAYOUT_NODES).
//!
//! Uses a fixed-seed xorshift PRNG for deterministic graph generation. No rand
//! dependency required.

use std::hint::black_box;

use cmdb_ui::dag::graph::{DagGraph, NodeKind};
use cmdb_ui::dag::layout::DagLayout;
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};

/// Simple xorshift64 PRNG for deterministic graph generation.
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Self(seed)
    }

    fn next_u32(&mut self) -> u32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        u32::try_from(self.0 >> 32).unwrap_or(u32::MAX)
    }

    /// Uniform-ish value in `low..high` (high exclusive). Returns `low` if the range is empty.
    fn range(&mut self, low: usize, high: usize) -> usize {
        let span = u64::try_from(high.saturating_sub(low))
            .unwrap_or(u64::MAX)
            .max(1);
        let offset = u64::from(self.next_u32()).checked_rem(span).unwrap_or(0);
        low.saturating_add(usize::try_from(offset).unwrap_or(0))
    }
}

/// Build a linear chain: 0 -> 1 -> 2 -> ... -> N-1
fn build_chain(n: usize) -> DagGraph {
    let mut g = DagGraph::default();
    for i in 0..n {
        g.add_node(
            NodeKind::Ec2,
            format!("chain-{}", i),
            format!("Chain {}", i),
        );
    }
    for i in 0..n.saturating_sub(1) {
        g.add_hierarchy_edge(i, i.saturating_add(1), String::new());
    }
    g
}

/// Build a wide fan-out: 0 (root) -> all others
fn build_fanout(n: usize) -> DagGraph {
    let mut g = DagGraph::default();
    g.add_node(NodeKind::Account, "root", "Root");
    for i in 1..n {
        g.add_node(NodeKind::Ec2, format!("leaf-{}", i), format!("Leaf {}", i));
    }
    for i in 1..n {
        g.add_hierarchy_edge(0, i, String::new());
    }
    g
}

/// Build a layered random DAG with approximately 2N edges.
/// Nodes are arranged in layers; edges go from lower layers to higher layers.
fn build_layered(n: usize) -> DagGraph {
    let mut g = DagGraph::default();
    let rng = &mut Rng::new(42);

    // Add nodes
    for i in 0..n {
        g.add_node(
            NodeKind::Lambda,
            format!("node-{}", i),
            format!("Node {}", i),
        );
    }

    // Edges: each node connects to 1-3 nodes with higher indices
    let target_edges = n.saturating_mul(2);
    let mut edge_count = 0_usize;
    for i in 0..n {
        let num_out = if edge_count.saturating_add(3) > target_edges {
            target_edges.saturating_sub(edge_count)
        } else {
            3
        };
        for _ in 0..num_out {
            if i.saturating_add(1) >= n {
                break;
            }
            let j = rng.range(i.saturating_add(1), n);
            // add_hierarchy_edge returns false for duplicates
            if g.add_hierarchy_edge(i, j, String::new()) {
                edge_count = edge_count.saturating_add(1);
            }
        }
        if edge_count >= target_edges {
            break;
        }
    }

    g
}

/// Build a graph with cycles: a DAG plus some back-edges.
fn build_cyclic(n: usize) -> DagGraph {
    let mut g = DagGraph::default();
    let rng = &mut Rng::new(123);

    // Build base DAG (chain with some skips)
    for i in 0..n {
        g.add_node(NodeKind::S3, format!("node-{}", i), format!("Node {}", i));
    }
    for i in 0..n.saturating_sub(1) {
        g.add_hierarchy_edge(i, i.saturating_add(1), String::new());
    }

    // Add back-edges to create cycles (~10% of nodes)
    let num_back_edges = n / 10;
    for _ in 0..num_back_edges {
        let i = rng.range(0, n.saturating_sub(2));
        let j = rng.range(i.saturating_add(2), n);
        g.add_hierarchy_edge(j, i, String::new());
    }

    g
}

fn bench_chain(c: &mut Criterion) {
    let mut group = c.benchmark_group("layout/chain");
    for &n in &[100, 500, 2000] {
        let graph = build_chain(n);
        group.bench_with_input(BenchmarkId::new("chain", n), &graph, |b, graph| {
            b.iter(|| {
                let layout = DagLayout::compute(black_box(graph));
                black_box(layout);
            });
        });
    }
    group.finish();
}

fn bench_fanout(c: &mut Criterion) {
    let mut group = c.benchmark_group("layout/fanout");
    for &n in &[100, 500, 2000] {
        let graph = build_fanout(n);
        group.bench_with_input(BenchmarkId::new("fanout", n), &graph, |b, graph| {
            b.iter(|| {
                let layout = DagLayout::compute(black_box(graph));
                black_box(layout);
            });
        });
    }
    group.finish();
}

fn bench_layered(c: &mut Criterion) {
    let mut group = c.benchmark_group("layout/layered");
    for &n in &[100, 500, 2000] {
        let graph = build_layered(n);
        group.bench_with_input(BenchmarkId::new("layered", n), &graph, |b, graph| {
            b.iter(|| {
                let layout = DagLayout::compute(black_box(graph));
                black_box(layout);
            });
        });
    }
    group.finish();
}

fn bench_cyclic(c: &mut Criterion) {
    let mut group = c.benchmark_group("layout/cyclic");
    for &n in &[100, 500, 2000] {
        let graph = build_cyclic(n);
        group.bench_with_input(BenchmarkId::new("cyclic", n), &graph, |b, graph| {
            b.iter(|| {
                let layout = DagLayout::compute(black_box(graph));
                black_box(layout);
            });
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_chain,
    bench_fanout,
    bench_layered,
    bench_cyclic
);
criterion_main!(benches);
