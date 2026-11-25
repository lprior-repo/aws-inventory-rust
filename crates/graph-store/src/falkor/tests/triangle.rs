use super::cycle::edge_rows;
use super::{TestResult, owner, peer};
use crate::cypher::Direction;
use crate::falkor::FalkorDb;
use crate::{
    EdgeRow, GraphBackend, GraphLimits, NodeKey, PathResult, Provenance, Truncation, cypher,
};
use inventory_core::inventory::{Relation, ResourceId};
use std::collections::VecDeque;

fn fixture() -> Result<([NodeKey; 3], [EdgeRow; 3]), Box<dyn std::error::Error>> {
    let key = |id| {
        let mut key = owner()?;
        key.id = ResourceId::parse(id)?;
        Ok::<_, Box<dyn std::error::Error>>(key)
    };
    let (a, r, s) = (key("A")?, key("R")?, key("s")?);
    let mut ranked = [
        (crate::encoding::digest(&a)?, a),
        (crate::encoding::digest(&r)?, r),
        (crate::encoding::digest(&s)?, s),
    ];
    ranked.sort_by(|left, right| left.0.cmp(&right.0));
    let [(_, a), (_, s), (_, root)] = ranked;
    let relation = Relation::parse("refers")?;
    let provenance = Provenance::parse("fixture")?;
    let edge = |from: &NodeKey, to: &NodeKey| EdgeRow {
        from: from.clone(),
        to: to.clone(),
        relation: relation.clone(),
        provenance: provenance.clone(),
    };
    let edges = [edge(&a, &s), edge(&root, &a), edge(&root, &s)];
    Ok(([root, a, s], edges))
}

fn script(
    keys: &[NodeKey; 3],
    edges: &[EdgeRow; 3],
    additional: bool,
    edge_budget: usize,
) -> Result<VecDeque<(String, String)>, crate::GraphError> {
    let [root, a, s] = keys;
    let [_, ra, rs] = edges;
    let completion = if additional {
        edge_rows(edges)
    } else {
        edge_rows(&[ra.clone(), rs.clone()])
    };
    Ok(VecDeque::from([
        (
            cypher::layer(
                std::slice::from_ref(root),
                &[],
                edge_budget.saturating_add(1),
                Direction::Both,
            )?,
            edge_rows(&[ra.clone(), rs.clone()]),
        ),
        (
            cypher::layer(&[a.clone(), s.clone()], keys, 1, Direction::Both)?,
            peer::empty(),
        ),
        (
            cypher::induced(keys, edge_budget.saturating_add(1))?,
            completion,
        ),
    ]))
}

async fn triangle_case(path: bool, additional: bool, edge_budget: usize) -> TestResult {
    let (keys, edges) = fixture()?;
    let mut script = script(&keys, &edges, additional, edge_budget)?;
    let expected_queries = script
        .iter()
        .map(|(query, _)| query.clone())
        .collect::<Vec<_>>();
    let peer = peer::Peer::start(move |query| {
        let (expected, response) = script.pop_front().ok_or("unexpected extra query")?;
        if query != expected {
            return Err(format!("expected {expected}, got {query}"));
        }
        Ok(response)
    })?;
    let graph = FalkorDb::connect(&peer.endpoint, "triangle")?;
    let [root, _, target] = keys;
    let [cross, ra, rs] = edges;
    let limits = GraphLimits::new(1, 3, edge_budget)?;
    if path {
        assert_eq!(
            graph.path(&root, &target, limits).await?,
            PathResult::Path(vec![rs])
        );
    } else {
        let result = graph.neighbors(&root, limits).await?;
        let mut expected = vec![ra, rs];
        if additional && edge_budget > 2 {
            expected.push(cross);
        }
        assert_eq!(result.edges, expected);
        assert_eq!(result.truncated(), additional && edge_budget == 2);
        let truncation = match (additional, edge_budget) {
            (true, 2) => Truncation::Budget,
            _ => Truncation::Complete,
        };
        assert_eq!(result.truncation, truncation);
    }
    drop(graph);
    assert_eq!(peer.finish().await?, expected_queries);
    Ok(())
}

#[tokio::test]
async fn a_budget_truncated_triangle_retains_both_root_edges_and_the_direct_path() -> TestResult {
    triangle_case(false, true, 2).await?;
    triangle_case(true, true, 2).await
}

#[tokio::test]
async fn an_exact_edge_budget_is_complete_when_no_additional_edge_is_refused() -> TestResult {
    triangle_case(false, false, 2).await
}

#[tokio::test]
async fn completion_appends_induced_edges_after_breadth_first_discovery_order() -> TestResult {
    triangle_case(false, true, 3).await
}

#[tokio::test]
async fn a_depth_boundary_reports_a_node_outside_the_visited_triangle() -> TestResult {
    let (keys, edges) = fixture()?;
    let [root, a, _] = &keys;
    let mut outside = root.clone();
    outside.id = ResourceId::parse("outside")?;
    let mut cutoff = edges.first().ok_or("missing triangle edge")?.clone();
    cutoff.from = a.clone();
    cutoff.to = outside;
    let mut script = script(&keys, &edges, true, 3)?;
    script.get_mut(1).ok_or("missing depth probe")?.1 = edge_rows(&[cutoff]);
    let expected_queries = script
        .iter()
        .map(|(query, _)| query.clone())
        .collect::<Vec<_>>();
    let peer = peer::Peer::start(move |query| {
        let (expected, response) = script.pop_front().ok_or("unexpected extra query")?;
        if query != expected {
            return Err(format!("expected {expected}, got {query}"));
        }
        Ok(response)
    })?;
    let graph = FalkorDb::connect(&peer.endpoint, "depth-cutoff")?;
    let result = graph.neighbors(root, GraphLimits::new(1, 3, 3)?).await?;
    let [cross, ra, rs] = edges;
    assert_eq!(result.edges, vec![ra, rs, cross]);
    assert_eq!(result.truncation, Truncation::Depth);
    drop(graph);
    assert_eq!(peer.finish().await?, expected_queries);
    Ok(())
}
