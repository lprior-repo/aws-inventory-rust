use super::{TestResult, peer};
use crate::falkor::FalkorDb;
use crate::{GraphBackend, MAX_QUERY_BYTES, cypher};

#[tokio::test]
async fn given_exact_query_byte_bound_when_sent_then_peer_and_adapter_agree() -> TestResult {
    let peer = peer::Peer::start(|query| {
        if matches!(query, cypher::RESOURCE_INDEX | cypher::TAG_INDEX)
            || query.len() == MAX_QUERY_BYTES
        {
            Ok(peer::empty())
        } else {
            Err(format!("unexpected query byte count {}", query.len()))
        }
    })?;
    let graph = FalkorDb::connect(&peer.endpoint, "query-boundary")?;
    graph.claim_projection(1).await?;
    let query = "x".repeat(MAX_QUERY_BYTES);
    let accepted = graph.request(&query).await?;
    let oversized = graph
        .request(&format!("{query}x"))
        .await
        .err()
        .ok_or("expected oversized query")?;
    assert_eq!(
        accepted,
        redis::Value::Array(vec![
            redis::Value::Array(vec![]),
            redis::Value::Array(vec![]),
        ])
    );
    assert_eq!(oversized.to_string(), "graph query exceeds its byte bound");
    drop(graph);
    assert_eq!(
        peer.finish().await?,
        vec![
            cypher::RESOURCE_INDEX.to_owned(),
            cypher::TAG_INDEX.to_owned(),
            query,
        ]
    );
    Ok(())
}
