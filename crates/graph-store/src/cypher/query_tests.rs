use super::{clear_tags, replace_edges, tag};
use crate::{EdgeRow, NodeKey, Provenance, TagRow};
use inventory_core::inventory::{AccountId, Kind, Region, Relation, ResourceId};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn key(id: &str) -> Result<NodeKey, Box<dyn std::error::Error>> {
    Ok(NodeKey {
        account: AccountId::parse("123456789012")?,
        region: Region::parse("us-east-1")?,
        kind: Kind::parse("AWS::EC2::Vpc")?,
        id: ResourceId::parse(id)?,
    })
}

#[test]
fn replacement_preserves_label_provenance_and_both_edge_directions() -> TestResult {
    let source = key("vpc-a")?;
    let target = key("vpc-b")?;
    let edge = EdgeRow {
        from: target.clone(),
        to: source.clone(),
        relation: Relation::parse("CONTAINS")?,
        provenance: Provenance::parse("reviewed-rule")?,
    };
    let query = replace_edges(&source, &[edge])?;
    let source = crate::encoding::digest(&source)?;
    let target = crate::encoding::digest(&target)?;
    assert_eq!(
        query,
        format!(
            "CYPHER key='{source}' rows=[{{source:'{target}',target:'{source}',relation:'CONTAINS',provenance:'reviewed-rule'}}] MATCH (owner:Resource {{key:$key}}) OPTIONAL MATCH (owner)-[old]-(other:Resource) DELETE old WITH DISTINCT owner UNWIND $rows AS row OPTIONAL MATCH (s:Resource {{key:row.source}}),(t:Resource {{key:row.target}}) FOREACH (_ IN CASE WHEN s IS NOT NULL AND t IS NOT NULL THEN [1] ELSE [] END | MERGE (s)-[:`CONTAINS` {{relation:row.relation,provenance:row.provenance}}]->(t)) WITH DISTINCT owner RETURN owner.key"
        )
    );
    Ok(())
}

#[test]
fn tag_parameters_preserve_scoped_identity_and_provenance() -> TestResult {
    let resource = key("vpc-a")?;
    let row = TagRow {
        identity: [7; 32],
        key: "team".into(),
        value: "platform".into(),
    };
    let resource_digest = crate::encoding::digest(&resource)?;
    let identity = "0707070707070707070707070707070707070707070707070707070707070707";
    assert_eq!(
        tag(&resource, &row)?,
        format!(
            "CYPHER resource='{resource_digest}' tag='{identity}' account='123456789012' region='us-east-1' tag_key='team' tag_value='platform' provenance='inventory.tags' MATCH (r:Resource {{key:$resource}}) MERGE (t:Tag {{key:$tag}}) SET t.account=$account,t.region=$region,t.tag_key=$tag_key,t.tag_value=$tag_value MERGE (r)-[:HAS_TAG {{provenance:$provenance}}]->(t)"
        )
    );
    Ok(())
}

#[test]
fn tag_data_is_bound_and_orphans_are_only_deleted_after_last_owner() -> TestResult {
    let resource = key("vpc-a")?;
    let row = TagRow {
        identity: [7; 32],
        key: "quote'\\\\".into(),
        value: "line\\n\\t'".into(),
    };
    let resource_digest = crate::encoding::digest(&resource)?;
    let identity = "0707070707070707070707070707070707070707070707070707070707070707";
    assert_eq!(
        tag(&resource, &row)?,
        format!(
            r"CYPHER resource='{resource_digest}' tag='{identity}' account='123456789012' region='us-east-1' tag_key='quote\'\\\\' tag_value='line\\n\\t\'' provenance='inventory.tags' MATCH (r:Resource {{key:$resource}}) MERGE (t:Tag {{key:$tag}}) SET t.account=$account,t.region=$region,t.tag_key=$tag_key,t.tag_value=$tag_value MERGE (r)-[:HAS_TAG {{provenance:$provenance}}]->(t)"
        )
    );
    assert_eq!(
        clear_tags(&resource)?,
        format!(
            "CYPHER key='{resource_digest}' MATCH (r:Resource {{key:$key}})-[link:HAS_TAG]->(t:Tag) WITH link,t LIMIT 1000 DELETE link WITH DISTINCT t WHERE NOT (:Resource)-[:HAS_TAG]->(t) DELETE t"
        )
    );
    Ok(())
}
