use super::append_edges;
use crate::{EdgeRow, GraphError, MAX_QUERY_BYTES, NodeKey, Provenance};
use inventory_core::inventory::{AccountId, Kind, Region, Relation, ResourceId};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn owner() -> Result<NodeKey, Box<dyn std::error::Error>> {
    Ok(NodeKey {
        account: AccountId::parse("123456789012")?,
        region: Region::parse("us-east-1")?,
        kind: Kind::parse("AWS::Example::Thing")?,
        id: ResourceId::parse("owner")?,
    })
}

#[test]
fn maximum_width_append_query_keeps_1000_rows_inside_the_byte_bound() -> TestResult {
    let key = owner()?;
    let edge = EdgeRow {
        from: key.clone(),
        to: key.clone(),
        relation: Relation::parse(&"A".repeat(128))?,
        provenance: Provenance::parse(&"\\".repeat(128))?,
    };
    let query = append_edges(&key, &vec![edge; 1_000])?;
    assert_eq!(query.matches("FOREACH").count(), 1);
    assert!(!query.contains("DELETE"));
    assert!(query.len() <= MAX_QUERY_BYTES);
    Ok(())
}

#[test]
fn append_rejects_a_nonincident_edge_with_exact_error() -> TestResult {
    let key = owner()?;
    let mut other = key.clone();
    other.id = ResourceId::parse("other")?;
    let edge = EdgeRow {
        from: other.clone(),
        to: other,
        relation: Relation::parse("attached")?,
        provenance: Provenance::parse("fixture")?,
    };
    let error = append_edges(&key, &[edge])
        .err()
        .ok_or("expected an error")?;
    assert_eq!(error.to_string(), GraphError::UnrelatedEdge.to_string());
    assert_eq!(
        error.to_string(),
        "replacement edge is not incident to its owner"
    );
    Ok(())
}

fn partition_case(count: usize, expected_sizes: &[usize]) -> TestResult {
    use super::{EdgeChunk, edge_chunks};
    let key = owner()?;
    let edges = (0..count)
        .map(|index| {
            let mut target = key.clone();
            target.id = ResourceId::parse(&format!("target-{index}"))?;
            Ok::<_, Box<dyn std::error::Error>>(EdgeRow {
                from: key.clone(),
                to: target,
                relation: Relation::parse("attached")?,
                provenance: Provenance::parse("fixture")?,
            })
        })
        .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;
    let plan: Vec<_> = edge_chunks(&edges, true).collect();
    assert_eq!(plan.first(), Some(&EdgeChunk::Clear));
    assert_eq!(
        plan.iter()
            .filter(|step| matches!(step, EdgeChunk::Clear))
            .count(),
        1
    );
    let batches: Vec<_> = plan
        .iter()
        .filter_map(|step| match step {
            EdgeChunk::Clear => None,
            EdgeChunk::Append(rows) => Some(*rows),
        })
        .collect();
    assert_eq!(
        batches.iter().map(|rows| rows.len()).collect::<Vec<_>>(),
        expected_sizes
    );
    assert_eq!(
        batches.into_iter().flatten().collect::<Vec<_>>(),
        edges.iter().collect::<Vec<_>>()
    );
    assert_eq!(
        edge_chunks(&edges, false)
            .filter(|step| matches!(step, EdgeChunk::Clear))
            .count(),
        0
    );
    Ok(())
}

#[test]
fn replacement_plan_clears_once_and_preserves_every_edge_at_2000_2001_and_5000() -> TestResult {
    partition_case(2_000, &[1_000, 1_000])?;
    partition_case(2_001, &[1_000, 1_000, 1])?;
    partition_case(5_000, &[1_000, 1_000, 1_000, 1_000, 1_000])
}

#[test]
fn empty_replacement_still_clears_once() -> TestResult {
    assert_eq!(
        super::edge_chunks(&[], true).collect::<Vec<_>>(),
        vec![super::EdgeChunk::Clear]
    );
    assert_eq!(super::edge_chunks(&[], false).count(), 0);
    Ok(())
}

#[test]
fn foreign_tag_fixture_quotes_all_untrusted_parameter_values() -> TestResult {
    let query = super::diagnostics::foreign_tag_fixture("d' }", "a\\", "r'")?;
    assert_eq!(
        query,
        "CYPHER digest='d\\' }' account='a\\\\' region='r\\'' MATCH (s:Resource {key:$digest}) CREATE (s)-[:HAS_TAG {provenance:'inventory.tags'}]->(:Tag {account:$account,region:$region,tag_key:'owner',tag_value:'after'}), (s)-[:HAS_TAG {provenance:'inventory.tags'}]->(:Tag {account:'222222222222',region:$region,tag_key:'owner',tag_value:'foreign'})"
    );
    Ok(())
}

#[test]
fn append_binds_one_directed_provenance_edge_without_clearing() -> TestResult {
    let key = owner()?;
    let mut target = key.clone();
    target.id = ResourceId::parse("target")?;
    let edge = EdgeRow {
        from: target.clone(),
        to: key.clone(),
        relation: Relation::parse("attached")?,
        provenance: Provenance::parse("fixture")?,
    };
    let query = append_edges(&key, &[edge])?;
    let owner = crate::encoding::digest(&key)?;
    let target = crate::encoding::digest(&target)?;
    assert_eq!(
        query,
        format!(
            "CYPHER key='{owner}' rows=[{{source:'{target}',target:'{owner}',relation:'attached',provenance:'fixture'}}] MATCH (owner:Resource {{key:$key}}) UNWIND $rows AS row OPTIONAL MATCH (s:Resource {{key:row.source}}),(t:Resource {{key:row.target}}) FOREACH (_ IN CASE WHEN s IS NOT NULL AND t IS NOT NULL THEN [1] ELSE [] END | MERGE (s)-[:`attached` {{relation:row.relation,provenance:row.provenance}}]->(t)) WITH DISTINCT owner RETURN owner.key"
        )
    );
    Ok(())
}

#[test]
fn given_one_or_a_thousand_edges_when_compiled_then_statement_depth_is_constant() -> TestResult {
    let key = owner()?;
    let edge = EdgeRow {
        from: key.clone(),
        to: key.clone(),
        relation: Relation::parse("CONTAINS")?,
        provenance: Provenance::parse("fixture")?,
    };
    let one = append_edges(&key, std::slice::from_ref(&edge))?;
    let thousand = append_edges(&key, &vec![edge; 1_000])?;
    let clauses = |query: &str| {
        [
            query.matches(" UNWIND ").count(),
            query.matches(" MATCH ").count(),
            query.matches(" FOREACH ").count(),
            query.matches(" WITH ").count(),
        ]
    };
    assert_eq!(clauses(&one), [1, 2, 1, 1]);
    assert_eq!(clauses(&thousand), [1, 2, 1, 1]);
    Ok(())
}

#[test]
fn frontier_lists_are_parameters_in_a_constant_query_body() -> TestResult {
    let key = owner()?;
    let expected = "$frontier AS frontier_key MATCH (s:Resource {key:frontier_key})-[r]-(n:Resource) WHERE NOT n.key IN $excluded AND (NOT n.key IN $frontier OR startNode(r)=s) RETURN startNode(r).account,startNode(r).region,startNode(r).kind,startNode(r).id,endNode(r).account,endNode(r).region,endNode(r).kind,endNode(r).id,r.relation,r.provenance LIMIT 1";
    for count in [1, 500] {
        let keys = vec![key.clone(); count];
        let query = super::layer(&keys, &keys, 1, super::Direction::Both)?;
        let (_, body) = query
            .split_once(" UNWIND ")
            .ok_or("frontier lists must be bound before the query body")?;
        assert_eq!(body, expected);
    }
    Ok(())
}
