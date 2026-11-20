//! Benchmarks for DagLayout::compute over generated graph shapes.
//!
//! Measures the layout computation pass across four shapes at three sizes:
//! deep chain (depth capped at MAX_LAYERS=64), wide fan-out, random forward-edge
//! DAG with duplicate edges dropped, and cyclic graph. Graph sizes: 100, 500,
//! 2000 (MAX_LAYOUT_NODES).
//!
//! Uses a fixed-seed xorshift PRNG for deterministic graph generation. No rand
//! dependency required.

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

/// Build a linear chain: 0 -> 1 -> 2 -> ... -> N-1.
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

/// Build a wide fan-out: 0 (root) -> all others.
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

/// Build a random forward-edge DAG with duplicate edges dropped.
/// Nodes are arranged in layers; edges go from lower layers to higher layers.
fn build_random_forward(n: usize) -> DagGraph {
    let mut g = DagGraph::default();
    for i in 0..n {
        g.add_node(
            NodeKind::Lambda,
            format!("node-{}", i),
            format!("Node {}", i),
        );
    }
    add_random_forward_edges(&mut g, n);
    g
}

/// Add random forward edges to a graph, targeting approximately 2N edges.
/// Duplicate edges are dropped by DagGraph.
fn add_random_forward_edges(g: &mut DagGraph, n: usize) {
    let rng = &mut Rng::new(42);
    let target_edges = n.saturating_mul(2);
    let mut edge_count = 0_usize;
    for i in 0..n {
        let remaining = target_edges.saturating_sub(edge_count);
        if remaining == 0 {
            break;
        }
        let num_out = if i.saturating_add(1) >= n {
            0
        } else {
            3.min(remaining)
        };
        for _ in 0..num_out {
            let j = rng.range(i.saturating_add(1), n);
            if g.add_hierarchy_edge(i, j, String::new()) {
                edge_count = edge_count.saturating_add(1);
            }
        }
    }
}

/// Build a graph with cycles: a chain plus back-edges.
fn build_cyclic(n: usize) -> DagGraph {
    let mut g = DagGraph::default();
    let rng = &mut Rng::new(123);
    for i in 0..n {
        g.add_node(NodeKind::S3, format!("node-{}", i), format!("Node {}", i));
    }
    for i in 0..n.saturating_sub(1) {
        g.add_hierarchy_edge(i, i.saturating_add(1), String::new());
    }
    let num_back_edges = n / 10;
    for _ in 0..num_back_edges {
        let i = rng.range(0, n.saturating_sub(2));
        let j = rng.range(i.saturating_add(2), n);
        g.add_hierarchy_edge(j, i, String::new());
    }
    g
}

/// Validate layout results and exit on failure.
fn validate_layout(name: &str, layout: &DagLayout, n: usize, expect_back_edges: bool) {
    if layout.node_count() != n {
        eprintln!("{}: node count {} != {}", name, layout.node_count(), n);
        std::process::exit(1);
    }
    if expect_back_edges {
        if layout.back_edges() == 0 {
            eprintln!("{}: expected back edges, got 0", name);
            std::process::exit(1);
        }
    } else if layout.back_edges() != 0 {
        eprintln!("{}: unexpected back edges {}", name, layout.back_edges());
        std::process::exit(1);
    }
    if layout.dropped_edges() != 0 {
        eprintln!("{}: dropped edges {}", name, layout.dropped_edges());
        std::process::exit(1);
    }
}

/// Run a layout benchmark for one graph shape across all sizes.
fn run_shape_bench(
    c: &mut Criterion,
    name: &str,
    builder: fn(usize) -> DagGraph,
    expect_back_edges: bool,
) {
    let mut group = c.benchmark_group(name);
    for &n in &[100, 500, 2000] {
        let graph = builder(n);
        // Validate layout correctness before timing.
        let layout = DagLayout::compute(&graph);
        validate_layout(name, &layout, n, expect_back_edges);
        group.bench_with_input(BenchmarkId::new(name, n), &graph, |b, graph| {
            b.iter(|| DagLayout::compute(graph))
        });
    }
    group.finish();
}

fn bench_deep_chain_capped(c: &mut Criterion) {
    // Depth is capped at MAX_LAYERS=64, so chains with n>=63 collapse
    // into layer 63.
    run_shape_bench(c, "layout/deep_chain_capped", build_chain, false);
}

fn bench_fanout(c: &mut Criterion) {
    run_shape_bench(c, "layout/fanout", build_fanout, false);
}

fn bench_random_forward(c: &mut Criterion) {
    run_shape_bench(c, "layout/random_forward", build_random_forward, false);
}

fn bench_cyclic(c: &mut Criterion) {
    run_shape_bench(c, "layout/cyclic", build_cyclic, true);
}

criterion_group!(
    benches,
    bench_deep_chain_capped,
    bench_fanout,
    bench_random_forward,
    bench_cyclic
);
criterion_main!(benches);