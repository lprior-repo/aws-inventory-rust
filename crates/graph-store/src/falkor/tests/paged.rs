use super::{TestResult, owner, peer};
use crate::falkor::FalkorDb;
use crate::{EdgeRow, GraphBackend, GraphError, NodeKey, Provenance, cypher};
use inventory_core::inventory::{Relation, ResourceId};
use std::sync::{Arc, Mutex};

type TestValue<T> = Result<T, Box<dyn std::error::Error>>;
type Edges = Arc<Mutex<Vec<EdgeRow>>>;

#[derive(Clone, Copy)]
enum Failure {
    Limits,
    Transport,
    Unrelated,
}

impl Failure {
    fn page(self, edge: &EdgeRow, foreign: &EdgeRow) -> Result<Vec<EdgeRow>, GraphError> {
        match self {
            Self::Limits => Ok(vec![edge.clone(); crate::MAX_EDGES.saturating_add(1)]),
            Self::Transport => Err(GraphError::Transport("page 2 fetch failed".into())),
            Self::Unrelated => Ok(vec![foreign.clone()]),
        }
    }

    fn message(self) -> &'static str {
        match self {
            Self::Limits => "graph traversal or page limit is invalid",
            Self::Transport => "graph command failed: page 2 fetch failed",
            Self::Unrelated => "replacement edge is not incident to its owner",
        }
    }
}

fn fixture() -> TestValue<(NodeKey, [EdgeRow; 2], EdgeRow)> {
    let owner = owner()?;
    let mut first = owner.clone();
    first.id = ResourceId::parse("first")?;
    let mut second = owner.clone();
    second.id = ResourceId::parse("second")?;
    let relation = Relation::parse("refers")?;
    let provenance = Provenance::parse("new")?;
    let edges = [
        EdgeRow {
            from: owner.clone(),
            to: first,
            relation: relation.clone(),
            provenance: provenance.clone(),
        },
        EdgeRow {
            from: second.clone(),
            to: owner.clone(),
            relation: relation.clone(),
            provenance: provenance.clone(),
        },
    ];
    let foreign = EdgeRow {
        from: second.clone(),
        to: second,
        relation,
        provenance,
    };
    Ok((owner, edges, foreign))
}

fn stateful_peer(owner: &NodeKey, edges: &[EdgeRow; 2], state: Edges) -> TestValue<peer::Peer> {
    let clear = cypher::replace_edges(owner, &[])?;
    let writes = edges
        .iter()
        .map(|edge| {
            Ok((
                cypher::append_edges(owner, std::slice::from_ref(edge))?,
                edge.clone(),
            ))
        })
        .collect::<Result<Vec<_>, GraphError>>()?;
    let digest = crate::encoding::digest(owner)?;
    let owner = owner.clone();
    peer::Peer::start(move |query| {
        if matches!(query, cypher::RESOURCE_INDEX | cypher::TAG_INDEX) {
            return Ok(peer::empty());
        }
        let mut stored = state.lock().map_err(|error| error.to_string())?;
        if query == clear {
            stored.retain(|edge| edge.from != owner && edge.to != owner);
        } else {
            let (_, edge) = writes
                .iter()
                .find(|(expected, _)| query == expected)
                .ok_or_else(|| format!("unexpected paged query {query}"))?;
            stored.push(edge.clone());
        }
        Ok(peer::rows(&[vec![digest.clone()]]))
    })
}

async fn replay_after(failure: Failure) -> TestResult {
    let (owner, edges, foreign) = fixture()?;
    let [first, second] = &edges;
    let mut stale = first.clone();
    stale.provenance = Provenance::parse("stale")?;
    let state = Arc::new(Mutex::new(vec![stale, foreign.clone()]));
    let peer = stateful_peer(&owner, &edges, Arc::clone(&state))?;
    let graph = FalkorDb::connect(&peer.endpoint, "paged-replay")?;
    graph.claim_projection(1).await?;
    let pages =
        futures_util::stream::iter([Ok(vec![first.clone()]), failure.page(second, &foreign)]);
    let error = graph
        .replace_edges_paged(&owner, pages)
        .await
        .err()
        .ok_or("expected page 2 failure")?;
    assert_eq!(error.to_string(), failure.message());
    assert_eq!(
        *state.lock().map_err(|error| error.to_string())?,
        vec![foreign.clone(), first.clone()]
    );
    let pages = futures_util::stream::iter(
        edges
            .iter()
            .map(|edge| Ok::<_, GraphError>(vec![edge.clone()])),
    );
    graph.replace_edges_paged(&owner, pages).await?;
    assert_eq!(
        *state.lock().map_err(|error| error.to_string())?,
        vec![foreign, first.clone(), second.clone()]
    );
    drop(graph);
    let queries = peer.finish().await?;
    let clear = cypher::replace_edges(&owner, &[])?;
    assert_eq!(queries.iter().filter(|query| **query == clear).count(), 2);
    assert_eq!(queries.len(), 7);
    Ok(())
}

#[tokio::test]
async fn given_limits_on_page_two_when_replayed_then_exact_new_edges_converge() -> TestResult {
    replay_after(Failure::Limits).await
}

#[tokio::test]
async fn given_transport_on_page_two_when_replayed_then_exact_new_edges_converge() -> TestResult {
    replay_after(Failure::Transport).await
}

#[tokio::test]
async fn given_unrelated_page_two_when_replayed_then_exact_new_edges_converge() -> TestResult {
    replay_after(Failure::Unrelated).await
}
