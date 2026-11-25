use super::{TestResult, owner, peer};
use crate::cypher::Direction;
use crate::falkor::FalkorDb;
use crate::{EdgeRow, GraphBackend, GraphLimits, NodeKey, Provenance, Truncation, cypher};
use inventory_core::inventory::{Relation, ResourceId};
use std::collections::VecDeque;

fn fixture() -> Result<([NodeKey; 3], [EdgeRow; 3]), Box<dyn std::error::Error>> {
    let mut root = owner()?;
    root.id = ResourceId::parse("R")?;
    let mut a = root.clone();
    a.id = ResourceId::parse("A")?;
    let mut s = root.clone();
    s.id = ResourceId::parse("s")?;
    let relation = Relation::parse("refers")?;
    let provenance = Provenance::parse("fixture")?;
    let edge = |from: &NodeKey, to: &NodeKey| EdgeRow {
        from: from.clone(),
        to: to.clone(),
        relation: relation.clone(),
        provenance: provenance.clone(),
    };
    let edges = [edge(&a, &root), edge(&s, &a), edge(&root, &s)];
    Ok(([root, a, s], edges))
}

pub(super) fn edge_rows(edges: &[EdgeRow]) -> String {
    peer::rows(
        &edges
            .iter()
            .map(|edge| {
                [&edge.from, &edge.to]
                    .into_iter()
                    .flat_map(|key| {
                        [
                            key.account.as_str(),
                            key.region.as_str(),
                            key.kind.as_str(),
                            key.id.as_str(),
                        ]
                    })
                    .chain([edge.relation.as_str(), edge.provenance.as_str()])
                    .map(str::to_owned)
                    .collect()
            })
            .collect::<Vec<_>>(),
    )
}

fn script(
    keys: &[NodeKey; 3],
    edges: &[EdgeRow; 3],
    direction: Direction,
) -> Result<VecDeque<(String, String)>, crate::GraphError> {
    let [root, a, s] = keys;
    let [ar, sa, rs] = edges;
    let first = cypher::layer(std::slice::from_ref(root), &[], 11, direction)?;
    let mut script = match direction {
        Direction::Incoming => VecDeque::from([
            (first, edge_rows(std::slice::from_ref(ar))),
            (
                cypher::layer(
                    std::slice::from_ref(a),
                    std::slice::from_ref(root),
                    10,
                    direction,
                )?,
                edge_rows(std::slice::from_ref(sa)),
            ),
            (
                cypher::layer(
                    std::slice::from_ref(s),
                    &[root.clone(), a.clone()],
                    9,
                    direction,
                )?,
                peer::empty(),
            ),
        ]),
        Direction::Both => VecDeque::from([
            (first, edge_rows(&[ar.clone(), rs.clone()])),
            (
                cypher::layer(
                    &[a.clone(), s.clone()],
                    std::slice::from_ref(root),
                    9,
                    direction,
                )?,
                edge_rows(std::slice::from_ref(sa)),
            ),
        ]),
    };
    let keys = keys
        .iter()
        .map(|key| crate::encoding::digest(key).map(|key| format!("'{key}'")))
        .collect::<Result<Vec<_>, _>>()?
        .join(",");
    script.push_back((format!("MATCH (s:Resource)-[r]->(t:Resource) WHERE s.key IN [{keys}] AND t.key IN [{keys}] RETURN s.account,s.region,s.kind,s.id,t.account,t.region,t.kind,t.id,r.relation,r.provenance ORDER BY s.key,t.key,r.relation,r.provenance LIMIT 11"), edge_rows(edges)));
    Ok(script)
}

async fn cycle_case(direction: Direction) -> TestResult {
    let (keys, edges) = fixture()?;
    let mut script = script(&keys, &edges, direction)?;
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
    let graph = FalkorDb::connect(&peer.endpoint, "cycle")?;
    let [root, _, _] = keys;
    let limits = GraphLimits::new(3, 10, 10)?;
    let result = match direction {
        Direction::Incoming => graph.blast_radius(&root, limits).await?,
        Direction::Both => graph.neighbors(&root, limits).await?,
    };
    let [ar, sa, rs] = edges;
    let expected = match direction {
        Direction::Incoming => vec![ar, sa, rs],
        Direction::Both => vec![ar, rs, sa],
    };
    assert_eq!(result.edges, expected);
    assert_eq!(result.truncation, Truncation::Complete);
    drop(graph);
    assert_eq!(peer.finish().await?, expected_queries);
    Ok(())
}

#[tokio::test]
async fn a_three_edge_cycle_preserves_the_exact_induced_edges_in_both_directions() -> TestResult {
    cycle_case(Direction::Incoming).await?;
    cycle_case(Direction::Both).await
}
