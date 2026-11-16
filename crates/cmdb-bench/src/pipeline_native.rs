use inventory_engine::inventory::{BinaryStore, run_projection};
use redis::{Value, aio::MultiplexedConnection};
use futures_util::{StreamExt, TryStreamExt, stream};
use std::time::{Duration, Instant};
use super::{pipeline::{BenchResult, Phase, Workload}, pipeline_fixture};
#[path = "pipeline_native_oracle.rs"]
mod resource_oracle;

#[derive(Clone, Debug)]
pub(super) struct Target { pub endpoint: String, pub graph: String }
impl Target {
    pub fn validate(&self) -> BenchResult {
        // Never target ambient/cloud services or reuse an existing user graph.
        let local = ["redis://127.0.0.1:", "redis://localhost:", "redis://[::1]:"]
            .iter().any(|prefix| self.endpoint.starts_with(prefix));
        if !local || !self.graph.starts_with("cmdb_bench_") || self.graph.len() > 128
            || !self.graph.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'_') {
            return Err("graph requires an explicit loopback redis://host:port endpoint and fresh cmdb_bench_ name".into());
        }
        // Redis validates authority, port, and credentials; prefix checking alone
        // is insufficient (a crafted user-info authority could escape loopback).
        let info = redis::Client::open(self.endpoint.as_str())?;
        match info.get_connection_info().addr() {
            redis::ConnectionAddr::Tcp(host, _) if matches!(host.as_str(), "127.0.0.1" | "localhost" | "::1") => Ok(()),
            _ => Err("benchmark graph endpoint must resolve from a loopback-only authority".into()),
        }
    }
}

async fn connection(target: &Target) -> BenchResult<MultiplexedConnection> {
    Ok(tokio::time::timeout(Duration::from_secs(5), redis::Client::open(target.endpoint.as_str())?.get_multiplexed_async_connection()).await??)
}

pub(super) async fn require_fresh(target: &Target) -> BenchResult {
    target.validate()?;
    let mut connection = connection(target).await?;
    let names: Vec<String> = tokio::time::timeout(Duration::from_secs(10), redis::cmd("GRAPH.LIST").query_async(&mut connection)).await??;
    if names.iter().any(|name| name == &target.graph) { return Err("benchmark refuses to modify an existing graph; use a fresh name".into()); }
    Ok(())
}

pub(super) async fn consume_and_check(workload: &Workload, store: &BinaryStore, target: &Target, phase: Phase) -> BenchResult {
    let state = store.projection_state().await?;
    let pending = state.pending;
    let started = Instant::now();
    // Each successful call acknowledges at least one revision (or rebuild marker).
    // No unbounded retry or silently successful offline benchmark.
    let acknowledged = stream::try_unfold(pending, |remaining| async move {
        if remaining == 0 { return Ok::<_, Box<dyn std::error::Error + Send + Sync>>(None); }
        let report = run_projection(store, &target.endpoint, &target.graph, 1000).await?;
        if report.offline || report.rebuild_incomplete || report.acknowledged == 0 || report.pending >= remaining {
            return Err("native projection failed/offline or made no bounded progress".into());
        }
        Ok(Some((report.acknowledged, report.pending)))
    }).try_fold(0_u64, |acknowledged, batch| async move {
        acknowledged.checked_add(batch).ok_or_else(|| "ack counter overflow".into())
    }).await?;
    let elapsed = started.elapsed();
    if store.projection_state().await?.pending != 0 { return Err("native consumer did not drain the durable outbox".into()); }
    check_topology(workload, target, phase).await?;
    let refreshes = if state.rebuild_required { u64::try_from(workload.resources()?)? } else { acknowledged };
    println!("pipeline_graph={} input_pending={pending} acknowledged={acknowledged} resource_refreshes={refreshes} pending=0 wall_us={} resource_refreshes_per_second={:.3} exact_native_state=true graph={}",
        phase.name(), elapsed.as_micros(), super::pipeline::rate(refreshes, elapsed)?, target.graph);
    Ok(())
}

async fn query_count(connection: &mut MultiplexedConnection, target: &Target, query: &str) -> BenchResult<u64> {
    let reply: Value = tokio::time::timeout(Duration::from_secs(30), redis::cmd("GRAPH.RO_QUERY").arg(&target.graph).arg(query).query_async(connection)).await??;
    count_reply(reply)
}

fn count_reply(reply: Value) -> BenchResult<u64> {
    let Value::Array(parts) = reply else { return Err("native count response is not an array".into()); };
    let Some(Value::Array(rows)) = parts.get(1) else { return Err("native count response has no rows".into()); };
    let [Value::Array(fields)] = rows.as_slice() else { return Err("native count response must have one row".into()); };
    let [Value::Int(count)] = fields.as_slice() else { return Err("native count response must have one integer".into()); };
    Ok(u64::try_from(*count)?)
}

async fn check_topology(workload: &Workload, target: &Target, phase: Phase) -> BenchResult {
    let connection = connection(target).await?;
    let resources = u64::try_from(workload.resources()?)?;
    let tags = pipeline_fixture::expected_tag_vertices(workload.rows_per_scope()?, phase).checked_mul(workload.concurrency).ok_or("tag vertex count overflow")?;
    let links = pipeline_fixture::expected_tags(workload.rows_per_scope()?, phase)?.checked_mul(workload.concurrency).ok_or("tag link count overflow")?;
    let expected = [
        ("MATCH (r:Resource) RETURN count(r)", resources),
        ("MATCH (t:Tag) RETURN count(t)", u64::try_from(tags)?),
        ("MATCH ()-[e:HAS_TAG]->() RETURN count(e)", u64::try_from(links)?),
        ("MATCH ()-[e]->() RETURN count(e)", u64::try_from(links)?),
        ("MATCH (n) RETURN count(n)", resources.checked_add(u64::try_from(tags)?).ok_or("native vertex count overflow")?),
        ("MATCH (r:Resource) RETURN count(DISTINCT r.key)", resources),
        ("MATCH (r:Resource)-[e:HAS_TAG]->(t:Tag) WHERE t.account IS NULL OR t.region IS NULL OR r.account<>t.account OR r.region<>t.region OR e.provenance IS NULL OR e.provenance<>'inventory.tags' RETURN count(e)", 0),
        ("MATCH (t:Tag) WHERE NOT (:Resource)-[:HAS_TAG]->(t) RETURN count(t)", 0),
    ];
    let connection = stream::iter(expected).map(Ok::<_, Box<dyn std::error::Error + Send + Sync>>)
        .try_fold(connection, |mut connection, (query, expected)| async move {
            let actual = query_count(&mut connection, target, query).await?;
            if actual != expected { return Err(format!("native topology count mismatch: expected={expected} actual={actual} query={query}").into()); }
            Ok(connection)
        }).await?;
    let mut connection = stream::iter((0..workload.resources()?).step_by(1000))
        .map(Ok::<_, Box<dyn std::error::Error + Send + Sync>>)
        .try_fold(connection, |mut connection, start| async move {
            let query = resource_oracle::query(workload, start)?;
            if query_count(&mut connection, target, &query).await? != 0 {
                return Err("native resource identity, name, or binary content hash differs from fixture".into());
            }
            Ok(connection)
        }).await?;
    let invalid_tags = if phase == Phase::Initial {
        "MATCH (r:Resource)-[:HAS_TAG]->(t:Tag) WHERE t.tag_key IS NULL OR t.tag_value IS NULL OR NOT ((t.tag_key='environment' AND t.tag_value='bench') OR (t.tag_key='owner' AND t.tag_value='inventory')) RETURN count(t)"
    } else {
        "MATCH (r:Resource)-[:HAS_TAG]->(t:Tag) WHERE t.tag_key IS NULL OR t.tag_value IS NULL OR toInteger(substring(r.name,6))%4=0 OR NOT ((t.tag_key='owner' AND t.tag_value='inventory') OR (t.tag_key='environment' AND t.tag_value=CASE WHEN toInteger(substring(r.name,6))%4=1 THEN 'bench-updated' ELSE 'bench' END)) RETURN count(t)"
    };
    if query_count(&mut connection, target, invalid_tags).await? != 0 { return Err("native graph tag assignments disagree with exact fixture state".into()); }
    let expected_links = if phase == Phase::Initial { "2" } else { "CASE WHEN toInteger(substring(r.name,6))%4=0 THEN 0 ELSE 2 END" };
    let link_query = format!("MATCH (r:Resource) OPTIONAL MATCH (r)-[e:HAS_TAG]->(t:Tag) WITH r,count(e) AS links,count(DISTINCT t.tag_key) AS tag_keys WHERE links<>{expected_links} OR tag_keys<>{expected_links} RETURN count(r)");
    if query_count(&mut connection, target, &link_query).await? != 0 { return Err("native resource tag cardinality or duplicate assignment mismatch".into()); }
    println!("pipeline_graph_state={} resources={resources} tag_vertices={tags} has_tag_links={links} dependency_edges=0 orphan_tags=0", phase.name());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn count_parser_rejects_truncated_or_noninteger_responses() {
        assert!(count_reply(Value::Nil).is_err());
        assert!(count_reply(Value::Array(Vec::new())).is_err());
        let valid = Value::Array(vec![Value::Array(vec![]), Value::Array(vec![Value::Array(vec![Value::Int(7)])])]);
        assert_eq!(count_reply(valid).ok(), Some(7));
    }
    #[test]
    fn graph_target_rejects_remote_authority_and_nonfixture_names() {
        assert!(Target { endpoint: "redis://127.0.0.1:6379@remote.example:6379".into(), graph: "cmdb_bench_safe".into() }.validate().is_err());
        assert!(Target { endpoint: "redis://127.0.0.1:6379".into(), graph: "user_graph".into() }.validate().is_err());
        assert!(Target { endpoint: "redis://127.0.0.1:6379".into(), graph: "cmdb_bench_safe".into() }.validate().is_ok());
    }
}
