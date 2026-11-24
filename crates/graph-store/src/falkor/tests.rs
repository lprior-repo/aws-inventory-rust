mod boundary;
mod cycle;
mod paged;
mod peer;
mod transport;

use super::*;
use inventory_core::inventory::{AccountId, Kind, Region, Relation, ResourceId};
use redis::Value;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn owner() -> Result<NodeKey, Box<dyn std::error::Error>> {
    Ok(NodeKey {
        account: AccountId::parse("123456789012")?,
        region: Region::parse("us-east-1")?,
        kind: Kind::parse("AWS::Example::Thing")?,
        id: ResourceId::parse("owner")?,
    })
}

#[tokio::test]
async fn mutations_without_claim_are_refused_before_contacting_the_server() -> TestResult {
    let graph = FalkorDb::connect("redis://127.0.0.1:1", "unclaimed")?;
    let owner = owner()?;
    let row = NodeRow {
        key: owner.clone(),
        name: None,
        content_hash: [0; 32],
    };
    let upsert = graph
        .upsert_node(&row)
        .await
        .err()
        .ok_or("expected an error")?;
    let replacement = graph
        .replace_edges(&owner, &[])
        .await
        .err()
        .ok_or("expected an error")?;
    let clear = graph.clear().await.err().ok_or("expected an error")?;
    assert_eq!(upsert.to_string(), "projection fence has not been claimed");
    assert_eq!(
        replacement.to_string(),
        "projection fence has not been claimed"
    );
    assert_eq!(clear.to_string(), "projection fence has not been claimed");
    Ok(())
}

fn reply(rows: Vec<Value>) -> Value {
    Value::Array(vec![Value::Array(Vec::new()), Value::Array(rows)])
}

#[test]
fn replacement_response_rejects_a_missing_or_mismatched_owner() -> TestResult {
    let owner = owner()?;
    let missing = response::replacement_owner(&reply(Vec::new()), &owner)
        .err()
        .ok_or("expected an error")?;
    assert_eq!(missing.to_string(), "replacement owner does not exist");
    let wrong = reply(vec![Value::Array(vec![Value::SimpleString(
        "wrong".into(),
    )])]);
    let error = response::replacement_owner(&wrong, &owner)
        .err()
        .ok_or("expected an error")?;
    assert_eq!(
        error.to_string(),
        "graph response is malformed or exceeds its row bound"
    );
    response::replacement_owner(
        &reply(vec![Value::Array(vec![Value::SimpleString(
            crate::encoding::digest(&owner)?,
        )])]),
        &owner,
    )?;
    Ok(())
}

#[tokio::test]
async fn existing_resource_and_tag_indexes_do_not_reject_a_projection_claim() -> TestResult {
    let peer = peer::Peer::start(|query| match query {
        cypher::RESOURCE_INDEX | cypher::TAG_INDEX => {
            Ok("-ERR 'key' is already indexed\r\n".into())
        }
        _ => Err(format!("unexpected index query {query}")),
    })?;
    let graph = FalkorDb::connect(&peer.endpoint, "already_indexed")?;
    graph.claim_projection(1).await?;
    drop(graph);
    assert_eq!(
        peer.finish().await?,
        vec![cypher::RESOURCE_INDEX, cypher::TAG_INDEX]
    );
    Ok(())
}

#[tokio::test]
async fn an_oversized_stream_page_is_rejected_after_the_single_clear() -> TestResult {
    let owner = owner()?;
    let clear = cypher::replace_edges(&owner, &[])?;
    let expected_clear = clear.clone();
    let owner_digest = crate::encoding::digest(&owner)?;
    let peer = peer::Peer::start(move |query| match query {
        cypher::RESOURCE_INDEX | cypher::TAG_INDEX => Ok(peer::empty()),
        query if query == clear => Ok(peer::rows(&[vec![owner_digest.clone()]])),
        _ => Err(format!("unexpected oversized-page query {query}")),
    })?;
    let graph = FalkorDb::connect(&peer.endpoint, "oversized")?;
    graph.claim_projection(1).await?;
    let edge = EdgeRow {
        from: owner.clone(),
        to: owner.clone(),
        relation: Relation::parse("refers")?,
        provenance: crate::Provenance::parse("fixture")?,
    };
    let pages = futures_util::stream::iter([Ok::<_, GraphError>(vec![
        edge;
        crate::MAX_EDGES
            .saturating_add(1)
    ])]);
    let error = graph
        .replace_edges_paged(&owner, pages)
        .await
        .err()
        .ok_or("expected an error")?;
    assert!(matches!(error, GraphError::Limits));
    drop(graph);
    assert_eq!(
        peer.finish().await?,
        vec![
            cypher::RESOURCE_INDEX.to_owned(),
            cypher::TAG_INDEX.to_owned(),
            expected_clear,
        ]
    );
    Ok(())
}

#[tokio::test]
async fn clear_stops_after_ten_million_vertices_and_a_final_probe() -> TestResult {
    let peer = peer::Peer::start(|query| match query {
        query if query == cypher::has_nodes() => Ok("*2\r\n*0\r\n*1\r\n*1\r\n:1\r\n".into()),
        cypher::RESOURCE_INDEX | cypher::TAG_INDEX => Ok(peer::empty()),
        query if query == cypher::clear_page() => Ok(peer::empty()),
        _ => Err(format!("unexpected clear query {query}")),
    })?;
    let graph = FalkorDb::connect(&peer.endpoint, "clear_budget")?;
    graph.claim_projection(1).await?;
    let error = graph.clear().await.err().ok_or("expected an error")?;
    assert!(matches!(error, GraphError::Limits));
    drop(graph);
    let queries = peer.finish().await?;
    let expected = [cypher::RESOURCE_INDEX, cypher::TAG_INDEX]
        .into_iter()
        .chain((0..10_000).flat_map(|_| [cypher::has_nodes(), cypher::clear_page()]))
        .chain(std::iter::once(cypher::has_nodes()))
        .collect::<Vec<_>>();
    assert_eq!(queries, expected);
    assert_eq!(
        cypher::clear_page(),
        "MATCH (n) WITH n LIMIT 1000 DETACH DELETE n"
    );
    let deletes = queries
        .iter()
        .filter(|query| query.as_str() == cypher::clear_page())
        .count();
    assert_eq!(deletes.checked_mul(1_000), Some(10_000_000));
    Ok(())
}

#[tokio::test]
async fn an_expired_read_request_maps_to_timeout() -> TestResult {
    let peer = peer::Peer::start(|_| Ok(peer::empty()))?;
    let graph = FalkorDb::connect(&peer.endpoint, "timeout")?;
    let mut session = graph.read_session().await?;
    session.deadline = Instant::now()
        .checked_sub(Duration::from_secs(1))
        .ok_or("deadline underflow")?;
    let error = session
        .request("MATCH (n) RETURN n LIMIT 1")
        .await
        .err()
        .ok_or("expected an error")?;
    assert!(matches!(error, GraphError::Timeout));
    drop(session);
    drop(graph);
    peer.finish().await?;
    Ok(())
}

#[tokio::test]
async fn tag_affiliations_return_the_exact_scoped_rows_and_provenance() -> TestResult {
    let key = owner()?;
    let expected_query = cypher::tag_affiliations(&key, 3)?;
    let peer = peer::Peer::start(move |query| {
        if query != expected_query {
            return Err(format!("unexpected tag query {query}"));
        }
        Ok(peer::rows(&[
            vec![
                "123456789012".into(),
                "us-east-1".into(),
                "team".into(),
                "platform".into(),
                "inventory.tags".into(),
            ],
            vec![
                "123456789012".into(),
                "us-east-1".into(),
                "tier".into(),
                "production".into(),
                "inventory.tags".into(),
            ],
        ]))
    })?;
    let graph = FalkorDb::connect(&peer.endpoint, "tag_rows")?;
    let affiliations = graph.tag_affiliations(&key, 2).await?;
    let affiliation = |key: &str, value: &str| crate::TagAffiliation {
        tag: crate::ScopedTag {
            account: "123456789012".into(),
            region: "us-east-1".into(),
            key: key.into(),
            value: value.into(),
        },
        provenance: "inventory.tags".into(),
    };
    assert_eq!(
        affiliations,
        crate::TagAffiliations {
            affiliations: vec![
                affiliation("team", "platform"),
                affiliation("tier", "production")
            ],
            truncated: false,
        }
    );
    drop(graph);
    assert_eq!(peer.finish().await?.len(), 1);
    Ok(())
}

mod replacement;
mod triangle;
