use super::{TestResult, owner, peer};
use crate::falkor::FalkorDb;
use crate::{EdgeRow, GraphBackend, MAX_QUERY_BYTES, NodeKey, Provenance, cypher};
use inventory_core::inventory::{Relation, ResourceId};

fn maximum_edges() -> Result<(NodeKey, Vec<EdgeRow>), Box<dyn std::error::Error>> {
    let key = owner()?;
    let edge = EdgeRow {
        from: key.clone(),
        to: key.clone(),
        relation: Relation::parse(&"A".repeat(128))?,
        provenance: Provenance::parse(&"\\".repeat(128))?,
    };
    Ok((key, vec![edge; 2_000]))
}

#[tokio::test]
async fn maximum_width_2000_edge_replacement_emits_only_bounded_queries() -> TestResult {
    // Given: 2,000 edges with maximum-length labels and maximally escaped provenance.
    let (key, edges) = maximum_edges()?;
    let error = cypher::replace_edges(&key, &edges)
        .err()
        .ok_or("expected a row-limit error")?;
    assert_eq!(
        error.to_string(),
        "graph traversal or page limit is invalid"
    );
    let reply = peer::rows(&[vec![crate::encoding::digest(&key)?]]);
    let peer = peer::Peer::start(move |query| {
        if matches!(query, cypher::RESOURCE_INDEX | cypher::TAG_INDEX) {
            Ok(peer::empty())
        } else {
            Ok(reply.clone())
        }
    })?;
    let graph = FalkorDb::connect(&peer.endpoint, "maximum_width")?;
    graph.claim_projection(1).await?;
    // When: the adapter sends the complete replacement through its fenced connection.
    graph.replace_edges(&key, &edges).await?;
    drop(graph);
    let queries = peer
        .finish()
        .await?
        .into_iter()
        .filter(|query| !matches!(query.as_str(), cypher::RESOURCE_INDEX | cypher::TAG_INDEX))
        .collect::<Vec<_>>();
    // Then: one clear and two complete append batches each fit the encoded byte bound.
    assert_eq!(queries.len(), 3);
    assert_eq!(queries.first(), Some(&cypher::replace_edges(&key, &[])?));
    assert_eq!(
        queries
            .iter()
            .map(|query| query.matches("DELETE old").count())
            .sum::<usize>(),
        1
    );
    assert_eq!(
        queries
            .iter()
            .map(|query| query.matches("FOREACH").count())
            .collect::<Vec<_>>(),
        vec![0, 1, 1]
    );
    let bytes = queries.iter().map(String::len).collect::<Vec<_>>();
    println!("2000 maximum-width edges: emitted query bytes={bytes:?}; limit={MAX_QUERY_BYTES}");
    assert!(bytes.iter().all(|bytes| *bytes <= MAX_QUERY_BYTES));
    Ok(())
}

#[tokio::test]
async fn a_foreign_edge_in_a_large_replacement_is_rejected_before_the_clear() -> TestResult {
    // Given: the last edge in a maximum-size batch is not incident to the owner.
    let (key, mut edges) = maximum_edges()?;
    let mut foreign = key.clone();
    foreign.id = ResourceId::parse("foreign")?;
    let edge = edges.last_mut().ok_or("missing final edge")?;
    edge.from = foreign.clone();
    edge.to = foreign;
    let graph = FalkorDb::connect("redis://127.0.0.1:1/", "no_io")?;
    // When: replacement is requested without any usable connection.
    let error = graph
        .replace_edges(&key, &edges)
        .await
        .err()
        .ok_or("expected an unrelated edge")?;
    // Then: validation fails before even the clear can touch the backend.
    assert_eq!(
        error.to_string(),
        "replacement edge is not incident to its owner"
    );
    Ok(())
}

#[tokio::test]
async fn mixed_relation_replacement_preserves_labels_and_order_without_clause_chains() -> TestResult
{
    // Given: a short incident set with nonconsecutive repeated relation labels.
    let key = owner()?;
    let edges = ["FIRST", "SECOND", "FIRST"]
        .into_iter()
        .enumerate()
        .map(|(index, label)| {
            Ok::<_, Box<dyn std::error::Error>>(EdgeRow {
                from: key.clone(),
                to: key.clone(),
                relation: Relation::parse(label)?,
                provenance: Provenance::parse(&format!("fixture-{index}"))?,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let reply = peer::rows(&[vec![crate::encoding::digest(&key)?]]);
    let peer = peer::Peer::start(move |query| {
        if matches!(query, cypher::RESOURCE_INDEX | cypher::TAG_INDEX) {
            Ok(peer::empty())
        } else {
            Ok(reply.clone())
        }
    })?;
    let graph = FalkorDb::connect(&peer.endpoint, "mixed-relations")?;
    graph.claim_projection(1).await?;
    // When: the adapter compiles the mixed-label set, rather than rejecting it.
    graph.replace_edges(&key, &edges).await?;
    drop(graph);
    // Then: each label retains its position and every statement has fixed depth.
    let expected = [
        cypher::RESOURCE_INDEX.to_owned(),
        cypher::TAG_INDEX.to_owned(),
        cypher::replace_edges(&key, &[])?,
    ]
    .into_iter()
    .chain(
        edges
            .iter()
            .map(|edge| cypher::append_edges(&key, std::slice::from_ref(edge)))
            .collect::<Result<Vec<_>, _>>()?,
    )
    .collect::<Vec<_>>();
    let queries = peer.finish().await?;
    assert_eq!(queries, expected);
    assert_eq!(
        queries
            .iter()
            .map(|query| query.matches(" UNWIND ").count())
            .collect::<Vec<_>>(),
        vec![0, 0, 0, 1, 1, 1]
    );
    Ok(())
}
