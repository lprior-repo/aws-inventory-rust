use super::{
    pipeline::{BenchResult, Phase, Workload},
    pipeline_fixture,
};
use futures_util::{TryStreamExt, stream};
use inventory_engine::inventory::{BinaryStore, run_projection};
use inventory_graph::falkor::{FalkorDb, inspect};
use redis::aio::MultiplexedConnection;
use std::time::{Duration, Instant};
#[path = "pipeline_native_oracle.rs"]
mod resource_oracle;

#[derive(Clone, Debug)]
pub(super) struct Target {
    pub endpoint: String,
    pub graph: String,
}
impl Target {
    pub fn validate(&self) -> BenchResult {
        // Never target ambient/cloud services or reuse an existing user graph.
        let local = ["redis://127.0.0.1:", "redis://localhost:", "redis://[::1]:"]
            .iter()
            .any(|prefix| self.endpoint.starts_with(prefix));
        if !local
            || !self.graph.starts_with("cmdb_bench_")
            || self.graph.len() > 128
            || !self
                .graph
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        {
            return Err("graph requires an explicit loopback redis://host:port endpoint and fresh cmdb_bench_ name".into());
        }
        // Redis validates authority, port, and credentials; prefix checking alone
        // is insufficient (a crafted user-info authority could escape loopback).
        let info = redis::Client::open(self.endpoint.as_str())?;
        match info.get_connection_info().addr() {
            redis::ConnectionAddr::Tcp(host, _)
                if matches!(host.as_str(), "127.0.0.1" | "localhost" | "::1") =>
            {
                Ok(())
            }
            _ => Err("benchmark graph endpoint must resolve from a loopback-only authority".into()),
        }
    }
}

async fn connection(target: &Target) -> BenchResult<MultiplexedConnection> {
    Ok(tokio::time::timeout(
        Duration::from_secs(5),
        redis::Client::open(target.endpoint.as_str())?.get_multiplexed_async_connection(),
    )
    .await??)
}

pub(super) async fn require_fresh(target: &Target) -> BenchResult {
    target.validate()?;
    let mut connection = connection(target).await?;
    let names: Vec<String> = tokio::time::timeout(
        Duration::from_secs(10),
        redis::cmd("GRAPH.LIST").query_async(&mut connection),
    )
    .await??;
    if names.iter().any(|name| name == &target.graph) {
        return Err("benchmark refuses to modify an existing graph; use a fresh name".into());
    }
    Ok(())
}

pub(super) async fn consume_and_check(
    workload: &Workload,
    store: &BinaryStore,
    target: &Target,
    phase: Phase,
) -> BenchResult {
    let state = store.projection_state().await?;
    let pending = state.pending;
    let started = Instant::now();
    // Each successful call acknowledges at least one revision (or rebuild marker).
    // No unbounded retry or silently successful offline benchmark.
    let backend = FalkorDb::connect(&target.endpoint, &target.graph)?;
    let backend = &backend;
    let acknowledged = stream::try_unfold(pending, |remaining| async move {
        if remaining == 0 {
            return Ok::<_, Box<dyn std::error::Error + Send + Sync>>(None);
        }
        let report = run_projection(store, backend, 1000).await?;
        if report.offline
            || report.rebuild_incomplete
            || report.acknowledged == 0
            || report.pending >= remaining
        {
            return Err("native projection failed/offline or made no bounded progress".into());
        }
        Ok(Some((report.acknowledged, report.pending)))
    })
    .try_fold(0_u64, |acknowledged, batch| async move {
        acknowledged
            .checked_add(batch)
            .ok_or_else(|| "ack counter overflow".into())
    })
    .await?;
    let elapsed = started.elapsed();
    if store.projection_state().await?.pending != 0 {
        return Err("native consumer did not drain the durable outbox".into());
    }
    check_topology(workload, target, phase).await?;
    let refreshes = if state.rebuild_required {
        u64::try_from(workload.resources()?)?
    } else {
        acknowledged
    };
    println!(
        "pipeline_graph={} input_pending={pending} acknowledged={acknowledged} acknowledged_unit=outbox_rows_one_per_resource_or_rebuild_marker resource_refreshes={refreshes} pending=0 wall_us={} resource_refreshes_per_second={} rate_clock=projection_delivery_wall rate_scope=delivery_calls_and_pending_check_excluding_topology_oracle exact_native_state=true graph={}",
        phase.name(),
        elapsed.as_micros(),
        super::pipeline::rate(refreshes, elapsed)?,
        target.graph
    );
    Ok(())
}

#[derive(Clone, Copy)]
struct TopologyCounts {
    resources: u64,
    tags: usize,
    links: usize,
}

fn topology_counts(workload: &Workload, phase: Phase) -> BenchResult<TopologyCounts> {
    let resources = u64::try_from(workload.resources()?)?;
    let tags = pipeline_fixture::expected_tag_vertices(workload.rows_per_scope()?, phase)
        .checked_mul(workload.concurrency)
        .ok_or("tag vertex count overflow")?;
    let links = pipeline_fixture::expected_tags(workload.rows_per_scope()?, phase)?
        .checked_mul(workload.concurrency)
        .ok_or("tag link count overflow")?;
    Ok(TopologyCounts {
        resources,
        tags,
        links,
    })
}

async fn run_count_queries(graph: &FalkorDb, counts: TopologyCounts) -> BenchResult {
    let TopologyCounts {
        resources,
        tags,
        links,
    } = counts;
    let expected = [
        (inspect::resource_count(graph).await?, resources),
        (inspect::tag_count(graph).await?, u64::try_from(tags)?),
        (inspect::tag_link_count(graph).await?, u64::try_from(links)?),
        (
            inspect::relationship_count(graph).await?,
            u64::try_from(links)?,
        ),
        (
            inspect::node_count(graph).await?,
            resources
                .checked_add(u64::try_from(tags)?)
                .ok_or("native vertex count overflow")?,
        ),
        (inspect::distinct_resource_count(graph).await?, resources),
        (inspect::invalid_tag_scope_count(graph).await?, 0),
        (inspect::orphan_tag_count(graph).await?, 0),
    ];
    for (actual, expected) in expected {
        if actual != expected {
            return Err(format!(
                "native topology count mismatch: expected={expected} actual={actual}"
            )
            .into());
        }
    }
    Ok(())
}

async fn check_resource_identities(graph: &FalkorDb, workload: &Workload) -> BenchResult {
    for start in (0..workload.resources()?).step_by(1000) {
        let rows = resource_oracle::rows(workload, start)?;
        if inspect::resource_mismatch_count(graph, &rows).await? != 0 {
            return Err(
                "native resource identity, name, or binary content hash differs from fixture"
                    .into(),
            );
        }
    }
    Ok(())
}

async fn check_tag_assignments(graph: &FalkorDb, phase: Phase) -> BenchResult {
    if inspect::invalid_tag_assignment_count(graph, fixture_phase(phase)).await? != 0 {
        return Err("native graph tag assignments disagree with exact fixture state".into());
    }
    Ok(())
}

async fn check_link_cardinality(graph: &FalkorDb, phase: Phase) -> BenchResult {
    if inspect::invalid_tag_cardinality_count(graph, fixture_phase(phase)).await? != 0 {
        return Err("native resource tag cardinality or duplicate assignment mismatch".into());
    }
    Ok(())
}

fn fixture_phase(phase: Phase) -> inspect::TagFixturePhase {
    match phase {
        Phase::Initial => inspect::TagFixturePhase::Initial,
        Phase::TagUpdate | Phase::Replay => inspect::TagFixturePhase::Updated,
    }
}

async fn check_topology(workload: &Workload, target: &Target, phase: Phase) -> BenchResult {
    let graph = FalkorDb::connect(&target.endpoint, &target.graph)?;
    let counts = topology_counts(workload, phase)?;
    run_count_queries(&graph, counts).await?;
    check_resource_identities(&graph, workload).await?;
    check_tag_assignments(&graph, phase).await?;
    check_link_cardinality(&graph, phase).await?;
    let TopologyCounts {
        resources,
        tags,
        links,
    } = counts;
    println!(
        "pipeline_graph_state={} resources={resources} tag_vertices={tags} has_tag_links={links} dependency_edges=0 orphan_tags=0",
        phase.name()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn graph_target_rejects_remote_authority_and_nonfixture_names() {
        assert!(
            Target {
                endpoint: "redis://127.0.0.1:6379@remote.example:6379".into(),
                graph: "cmdb_bench_safe".into()
            }
            .validate()
            .is_err()
        );
        assert!(
            Target {
                endpoint: "redis://127.0.0.1:6379".into(),
                graph: "user_graph".into()
            }
            .validate()
            .is_err()
        );
        assert!(
            Target {
                endpoint: "redis://127.0.0.1:6379".into(),
                graph: "cmdb_bench_safe".into()
            }
            .validate()
            .is_ok()
        );
    }
}
