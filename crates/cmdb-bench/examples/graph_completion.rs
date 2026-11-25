use inventory_core::inventory::{AccountId, Kind, Region, Relation, ResourceId};
use inventory_graph::falkor::{FalkorDb, inspect};
use inventory_graph::{
    EdgeRow, GraphBackend, GraphError, GraphLimits, NodeKey, NodeRow, Provenance,
};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

type BenchResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

struct Fixture {
    nodes: Vec<NodeRow>,
    star: Vec<EdgeRow>,
    hub: Vec<EdgeRow>,
}

impl Fixture {
    fn new() -> BenchResult<Self> {
        let account = AccountId::parse("123456789012")?;
        let region = Region::parse("us-east-1")?;
        let kind = Kind::parse("AWS::EC2::Vpc")?;
        let nodes = (0..500)
            .map(|index| -> BenchResult<NodeRow> {
                Ok(NodeRow {
                    key: NodeKey {
                        account: account.clone(),
                        region: region.clone(),
                        kind: kind.clone(),
                        id: ResourceId::parse(&format!("node-{index:03}"))?,
                    },
                    name: None,
                    content_hash: [0; 32],
                })
            })
            .collect::<BenchResult<Vec<_>>>()?;
        let root = &nodes.first().ok_or("missing root")?.key;
        let hub_key = &nodes.get(1).ok_or("missing hub")?.key;
        let mut star = Vec::new();
        for node in nodes.iter().skip(1) {
            star.push(edge(root, &node.key, "star", "fixture")?);
            star.push(edge(&node.key, root, "star", "fixture")?);
        }
        let mut hub = star
            .iter()
            .filter(|edge| &edge.from == hub_key || &edge.to == hub_key)
            .cloned()
            .collect::<Vec<_>>();
        for index in 0_usize..9_998 {
            let target = nodes
                .get(index.checked_rem(500).ok_or("target overflow")?)
                .ok_or("missing hub target")?;
            hub.push(edge(
                hub_key,
                &target.key,
                "hub",
                &format!("parallel-{index}"),
            )?);
        }
        if hub.len() != 10_000 {
            return Err(format!("expected 10000 hub edges, got {}", hub.len()).into());
        }
        Ok(Self { nodes, star, hub })
    }

    async fn seed(&self, graph: &FalkorDb) -> BenchResult {
        graph.claim_projection(1).await?;
        for node in &self.nodes {
            graph.upsert_node(node).await?;
        }
        let root = &self.nodes.first().ok_or("missing root")?.key;
        let hub = &self.nodes.get(1).ok_or("missing hub")?.key;
        graph.replace_edges_paged(root, pages(&self.star)).await?;
        graph.replace_edges_paged(hub, pages(&self.hub)).await?;
        let resources = inspect::resource_count(graph).await?;
        let relationships = inspect::relationship_count(graph).await?;
        if (resources, relationships) != (500, 10_996) {
            return Err(format!(
                "expected 500 resources and 10996 edges, got {resources}/{relationships}"
            )
            .into());
        }
        Ok(())
    }
}

fn edge(from: &NodeKey, to: &NodeKey, relation: &str, provenance: &str) -> BenchResult<EdgeRow> {
    Ok(EdgeRow {
        from: from.clone(),
        to: to.clone(),
        relation: Relation::parse(relation)?,
        provenance: Provenance::parse(provenance)?,
    })
}

fn pages(
    edges: &[EdgeRow],
) -> impl futures_util::Stream<Item = Result<Vec<EdgeRow>, GraphError>> + Send {
    futures_util::stream::iter(edges.chunks(2_000).map(|chunk| Ok(chunk.to_vec())))
}

#[derive(Clone, Copy, Debug)]
enum Operation {
    Neighbors,
    BlastRadius,
    Path,
}

struct Benchmark<'a> {
    graph: &'a FalkorDb,
    root: &'a NodeKey,
    target: &'a NodeKey,
    limits: GraphLimits,
}

impl Benchmark<'_> {
    async fn call<const VALIDATE: bool>(
        &self,
        operation: Operation,
        completion: bool,
    ) -> BenchResult {
        match operation {
            Operation::Neighbors | Operation::BlastRadius => {
                let incoming = matches!(operation, Operation::BlastRadius);
                let result = if !completion {
                    inspect::neighborhood_without_completion(
                        self.graph,
                        self.root,
                        self.limits,
                        incoming,
                    )
                    .await?
                } else if incoming {
                    self.graph.blast_radius(self.root, self.limits).await?
                } else {
                    self.graph.neighbors(self.root, self.limits).await?
                };
                if VALIDATE {
                    check_visited(self.root, &result)?;
                }
                std::hint::black_box(result);
            }
            Operation::Path => {
                let result = if completion {
                    self.graph.path(self.root, self.target, self.limits).await?
                } else {
                    inspect::path_without_completion(
                        self.graph,
                        self.root,
                        self.target,
                        self.limits,
                    )
                    .await?
                };
                if VALIDATE {
                    let expected = inventory_graph::PathResult::Path(vec![edge(
                        self.root,
                        self.target,
                        "star",
                        "fixture",
                    )?]);
                    if result != expected {
                        return Err(format!("expected direct fixture path, got {result:?}").into());
                    }
                }
                std::hint::black_box(result);
            }
        }
        Ok(())
    }

    async fn measure(&self, operation: Operation) -> BenchResult {
        self.call::<true>(operation, false)
            .await
            .map_err(|error| format!("{operation:?} without-induced warmup failed: {error}"))?;
        self.call::<true>(operation, true)
            .await
            .map_err(|error| format!("{operation:?} with-induced warmup failed: {error}"))?;
        let mut baseline = Vec::new();
        let mut completed = Vec::new();
        for _ in 0..20 {
            let started = Instant::now();
            self.call::<false>(operation, false).await?;
            baseline.push(started.elapsed().as_micros());
            let started = Instant::now();
            self.call::<false>(operation, true).await?;
            completed.push(started.elapsed().as_micros());
        }
        let baseline = summary(baseline)?;
        let completed = summary(completed)?;
        println!(
            "operation={operation:?} samples=20 without_induced_median_us={} without_induced_worst_us={} with_induced_median_us={} with_induced_worst_us={} induced_more_than_doubles_median={}",
            baseline.0,
            baseline.1,
            completed.0,
            completed.1,
            completed.0 > baseline.0.saturating_mul(2)
        );
        Ok(())
    }
}

fn check_visited(root: &NodeKey, result: &inventory_graph::GraphNeighborhood) -> BenchResult {
    let visited = result
        .edges
        .iter()
        .flat_map(|edge| [&edge.from, &edge.to])
        .chain(std::iter::once(root))
        .collect::<std::collections::BTreeSet<_>>();
    if visited.len() != 500 {
        return Err(format!("expected 500 visited nodes, got {}", visited.len()).into());
    }
    Ok(())
}

fn summary(mut times: Vec<u128>) -> BenchResult<(u128, u128)> {
    times.sort_unstable();
    let left = times.get(9).ok_or("missing lower median")?;
    let right = times.get(10).ok_or("missing upper median")?;
    let median = left.checked_add(*right).ok_or("median overflow")? / 2;
    Ok((median, *times.last().ok_or("missing worst sample")?))
}

async fn run(graph: &FalkorDb) -> BenchResult {
    let fixture = Fixture::new()?;
    fixture.seed(graph).await?;
    let benchmark = Benchmark {
        graph,
        root: &fixture.nodes.first().ok_or("missing root")?.key,
        target: &fixture.nodes.last().ok_or("missing target")?.key,
        limits: GraphLimits::new(1, 500, 2_000)?,
    };
    println!(
        "nodes=500 hub_incident_edges=10000 total_edges=10996 depth=1 node_budget=500 edge_budget=2000"
    );
    for operation in [
        Operation::Neighbors,
        Operation::BlastRadius,
        Operation::Path,
    ] {
        benchmark.measure(operation).await?;
    }
    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> BenchResult {
    let endpoint = std::env::args()
        .nth(1)
        .ok_or("pass an explicit loopback FalkorDB URL")?;
    let info = redis::Client::open(endpoint.as_str())?;
    match info.get_connection_info().addr() {
        redis::ConnectionAddr::Tcp(host, _)
            if matches!(host.as_str(), "127.0.0.1" | "localhost" | "::1") => {}
        _ => return Err("benchmark requires a loopback TCP endpoint".into()),
    }
    let name = format!(
        "cmdb_bench_completion_{}_{}",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    );
    let mut connection = info.get_multiplexed_async_connection().await?;
    let graphs: Vec<String> = redis::cmd("GRAPH.LIST")
        .query_async(&mut connection)
        .await?;
    if graphs.contains(&name) {
        return Err("fresh benchmark graph already exists".into());
    }
    let graph = FalkorDb::connect(&endpoint, &name)?;
    let result = run(&graph).await;
    let deleted = redis::cmd("GRAPH.DELETE")
        .arg(&name)
        .query_async::<String>(&mut connection)
        .await;
    let fence = redis::cmd("DEL")
        .arg(format!("aws-inventory:projection-owner:{name}"))
        .query_async::<u64>(&mut connection)
        .await;
    println!("graph={name} graph_delete={deleted:?} fence_delete={fence:?}");
    deleted?;
    if fence? != 1 {
        return Err("benchmark projection fence was not removed".into());
    }
    result
}
