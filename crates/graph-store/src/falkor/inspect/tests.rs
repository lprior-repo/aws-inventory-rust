use super::*;
use inventory_core::inventory::{AccountId, Kind, Region, ResourceId};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn reply(count: i64) -> redis::Value {
    redis::Value::Array(vec![
        redis::Value::Array(Vec::new()),
        redis::Value::Array(vec![redis::Value::Array(vec![redis::Value::Int(count)])]),
        redis::Value::Array(Vec::new()),
    ])
}

fn check(check: Inspection, query: &str, expected: u64) -> TestResult {
    assert_eq!(cypher::inspection(check), query);
    assert_eq!(decode_count(&reply(i64::try_from(expected)?))?, expected);
    Ok(())
}

#[test]
fn resource_count_query_and_reply_are_exact() -> TestResult {
    check(
        Inspection::Resources,
        "MATCH (r:Resource) RETURN count(r)",
        17,
    )
}

#[test]
fn tag_count_query_and_reply_are_exact() -> TestResult {
    check(Inspection::Tags, "MATCH (t:Tag) RETURN count(t)", 4)
}

#[test]
fn tag_link_count_query_and_reply_are_exact() -> TestResult {
    check(
        Inspection::TagLinks,
        "MATCH ()-[e:HAS_TAG]->() RETURN count(e)",
        34,
    )
}

#[test]
fn relationship_count_query_and_reply_are_exact() -> TestResult {
    check(
        Inspection::Relationships,
        "MATCH ()-[e]->() RETURN count(e)",
        34,
    )
}

#[test]
fn node_count_query_and_reply_are_exact() -> TestResult {
    check(Inspection::Nodes, "MATCH (n) RETURN count(n)", 21)
}

#[test]
fn distinct_resource_count_query_and_reply_are_exact() -> TestResult {
    check(
        Inspection::DistinctResources,
        "MATCH (r:Resource) RETURN count(DISTINCT r.key)",
        17,
    )
}

#[test]
fn invalid_tag_scope_count_query_and_reply_are_exact() -> TestResult {
    check(
        Inspection::InvalidTagScopes,
        "MATCH (r:Resource)-[e:HAS_TAG]->(t:Tag) WHERE t.account IS NULL OR t.region IS NULL OR r.account<>t.account OR r.region<>t.region OR e.provenance IS NULL OR e.provenance<>'inventory.tags' RETURN count(e)",
        0,
    )
}

#[test]
fn orphan_tag_count_query_and_reply_are_exact() -> TestResult {
    check(
        Inspection::OrphanTags,
        "MATCH (t:Tag) WHERE NOT (:Resource)-[:HAS_TAG]->(t) RETURN count(t)",
        0,
    )
}

#[test]
fn initial_tag_assignment_query_and_reply_are_exact() -> TestResult {
    check(
        Inspection::InvalidTagAssignments(TagFixturePhase::Initial),
        "MATCH (r:Resource)-[:HAS_TAG]->(t:Tag) WHERE t.tag_key IS NULL OR t.tag_value IS NULL OR NOT ((t.tag_key='environment' AND t.tag_value='bench') OR (t.tag_key='owner' AND t.tag_value='inventory')) RETURN count(t)",
        0,
    )
}

#[test]
fn updated_tag_assignment_query_and_reply_are_exact() -> TestResult {
    check(
        Inspection::InvalidTagAssignments(TagFixturePhase::Updated),
        "MATCH (r:Resource)-[:HAS_TAG]->(t:Tag) WHERE t.tag_key IS NULL OR t.tag_value IS NULL OR toInteger(substring(r.name,6))%4=0 OR NOT ((t.tag_key='owner' AND t.tag_value='inventory') OR (t.tag_key='environment' AND t.tag_value=CASE WHEN toInteger(substring(r.name,6))%4=1 THEN 'bench-updated' ELSE 'bench' END)) RETURN count(t)",
        0,
    )
}

#[test]
fn initial_tag_cardinality_query_and_reply_are_exact() -> TestResult {
    check(
        Inspection::InvalidTagCardinality(TagFixturePhase::Initial),
        "MATCH (r:Resource) OPTIONAL MATCH (r)-[e:HAS_TAG]->(t:Tag) WITH r,count(e) AS links,count(DISTINCT t.tag_key) AS tag_keys WHERE links<>2 OR tag_keys<>2 RETURN count(r)",
        0,
    )
}

#[test]
fn updated_tag_cardinality_query_and_reply_are_exact() -> TestResult {
    check(
        Inspection::InvalidTagCardinality(TagFixturePhase::Updated),
        "MATCH (r:Resource) OPTIONAL MATCH (r)-[e:HAS_TAG]->(t:Tag) WITH r,count(e) AS links,count(DISTINCT t.tag_key) AS tag_keys WHERE links<>CASE WHEN toInteger(substring(r.name,6))%4=0 THEN 0 ELSE 2 END OR tag_keys<>CASE WHEN toInteger(substring(r.name,6))%4=0 THEN 0 ELSE 2 END RETURN count(r)",
        0,
    )
}

fn expected_resource() -> Result<NodeRow, Box<dyn std::error::Error>> {
    Ok(NodeRow {
        key: crate::NodeKey {
            account: AccountId::parse("123456789012")?,
            region: Region::parse("us-east-1")?,
            kind: Kind::parse("AWS::DynamoDB::Table")?,
            id: ResourceId::parse(
                "arn:aws:dynamodb:us-east-1:123456789012:table/bench-000000000001",
            )?,
        },
        name: Some("bench-000000000001".into()),
        content_hash: [7; 32],
    })
}

#[test]
fn resource_mismatch_query_and_reply_are_exact() -> TestResult {
    let row = expected_resource()?;
    let digest = crate::encoding::digest(&row.key)?;
    let expected = format!(
        "CYPHER key0='{digest}' account0='123456789012' region0='us-east-1' kind0='AWS::DynamoDB::Table' id0='arn:aws:dynamodb:us-east-1:123456789012:table/bench-000000000001' name0='bench-000000000001' content_hash0='0707070707070707070707070707070707070707070707070707070707070707' UNWIND [{{key:$key0,account:$account0,region:$region0,kind:$kind0,id:$id0,name:$name0,content_hash:$content_hash0}}] AS expected OPTIONAL MATCH (r:Resource {{key:expected.key}}) WITH expected,r WHERE r IS NULL OR r.account IS NULL OR r.account<>expected.account OR r.region IS NULL OR r.region<>expected.region OR r.kind IS NULL OR r.kind<>expected.kind OR r.id IS NULL OR r.id<>expected.id OR r.name IS NULL OR r.name<>expected.name OR r.content_hash IS NULL OR r.content_hash<>expected.content_hash RETURN count(expected)"
    );
    assert_eq!(cypher::inspect_resources(&[row])?, expected);
    assert_eq!(decode_count(&reply(3))?, 3);
    Ok(())
}

#[test]
fn resource_oracle_rejects_oversized_and_unnamed_expectations() -> TestResult {
    let row = expected_resource()?;
    let error = cypher::inspect_resources(&vec![row.clone(); 1001])
        .err()
        .ok_or("expected an error")?;
    assert_eq!(
        error.to_string(),
        "graph traversal or page limit is invalid"
    );
    let unnamed = NodeRow { name: None, ..row };
    let error = cypher::inspect_resources(&[unnamed])
        .err()
        .ok_or("expected an error")?;
    assert_eq!(error.to_string(), "graph row is invalid");
    Ok(())
}

#[test]
fn count_decoder_rejects_missing_extra_negative_and_noninteger_values() -> TestResult {
    [
        redis::Value::Nil,
        redis::Value::Array(Vec::new()),
        redis::Value::Array(vec![redis::Value::Nil, redis::Value::Array(Vec::new())]),
        redis::Value::Array(vec![
            redis::Value::Nil,
            redis::Value::Array(vec![redis::Value::Nil]),
        ]),
        redis::Value::Array(vec![
            redis::Value::Nil,
            redis::Value::Array(vec![redis::Value::Array(vec![
                redis::Value::Int(1),
                redis::Value::Int(2),
            ])]),
        ]),
        redis::Value::Array(vec![
            redis::Value::Nil,
            redis::Value::Array(vec![redis::Value::Array(vec![redis::Value::SimpleString(
                "7".into(),
            )])]),
        ]),
        redis::Value::Array(vec![
            redis::Value::Nil,
            redis::Value::Array(vec![
                redis::Value::Array(vec![redis::Value::Int(1)]),
                redis::Value::Array(vec![redis::Value::Int(2)]),
            ]),
        ]),
        reply(-1),
    ]
    .iter()
    .try_for_each(|value| {
        let error = decode_count(value).err().ok_or("expected an error")?;
        assert_eq!(
            error.to_string(),
            "graph response is malformed or exceeds its row bound"
        );
        Ok::<_, Box<dyn std::error::Error>>(())
    })
}

#[tokio::test]
async fn empty_resource_expectations_return_exact_zero_without_contacting_graph() -> TestResult {
    let graph = FalkorDb::connect("redis://127.0.0.1:1", "empty_inspection")?;
    assert_eq!(resource_mismatch_count(&graph, &[]).await?, 0);
    Ok(())
}
