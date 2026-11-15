use super::super::projection_read::{ProjectionEdgeHeader, ProjectionResource, ProjectionTag};
use inventory_core::inventory::ResourceKey;
use sha2::{Digest, Sha256};
use std::fmt::Write;

const MAX_QUERY_BYTES: usize = 1_048_576;

pub(super) fn node(key: &ResourceKey, resource: &ProjectionResource) -> Result<String, QueryError> {
    let identity = hex(key.digest().as_bytes())?;
    let mut query = String::with_capacity(512);
    let content_hash = hex(&resource.content_hash)?;
    let name = resource.name.as_deref();
    parameters(
        &mut query,
        &[
            ("key", Some(&identity)),
            ("account", Some(key.account.as_str())),
            ("region", Some(key.region.as_str())),
            ("kind", Some(key.kind.as_str())),
            ("id", Some(key.id.as_str())),
            ("name", name),
            ("content_hash", Some(&content_hash)),
        ],
    )?;
    query.push_str(" MERGE (n:Resource {key:$key}) SET n.account=$account,n.region=$region,n.kind=$kind,n.id=$id,n.name=$name,n.content_hash=$content_hash");
    bounded(query)
}

pub(super) fn delete(key: &ResourceKey) -> Result<String, QueryError> {
    let identity = hex(key.digest().as_bytes())?;
    let mut query = String::with_capacity(160);
    parameters(&mut query, &[("key", Some(&identity))])?;
    query.push_str(" MATCH (n:Resource {key:$key}) DETACH DELETE n");
    bounded(query)
}

pub(super) fn edge(edge: &ProjectionEdgeHeader) -> Result<String, QueryError> {
    let source = hex(edge.from.digest().as_bytes())?;
    let target = hex(edge.to.digest().as_bytes())?;
    let relation = edge.relation.as_str();
    let label = identifier(relation)?;
    let mut query = String::with_capacity(512);
    parameters(
        &mut query,
        &[
            ("source", Some(&source)),
            ("target", Some(&target)),
            ("relation", Some(relation)),
            ("provenance", Some(edge.source.as_str())),
        ],
    )?;
    write!(query, " MATCH (s:Resource {{key:$source}}),(t:Resource {{key:$target}}) MERGE (s)-[r:{label} {{relation:$relation,provenance:$provenance}}]->(t)").map_err(|_| QueryError::TooLarge)?;
    bounded(query)
}
pub(super) fn clear_edges(key: &ResourceKey) -> Result<String, QueryError> {
    let identity = hex(key.digest().as_bytes())?;
    let mut query = String::with_capacity(192);
    parameters(&mut query, &[("key", Some(&identity))])?;
    query.push_str(" MATCH (n:Resource {key:$key})-[r]-(other:Resource) DELETE r");
    bounded(query)
}

pub(super) fn tag(key: &ResourceKey, tag: &ProjectionTag) -> Result<String, QueryError> {
    let resource = hex(key.digest().as_bytes())?;
    let identity = tag_identity(key, tag)?;
    let mut query = String::with_capacity(768);
    parameters(
        &mut query,
        &[
            ("resource", Some(&resource)),
            ("tag", Some(&identity)),
            ("account", Some(key.account.as_str())),
            ("region", Some(key.region.as_str())),
            ("tag_key", Some(&tag.key)),
            ("tag_value", Some(&tag.value)),
            ("provenance", Some("inventory.tags")),
        ],
    )?;
    query.push_str(" MATCH (r:Resource {key:$resource}) MERGE (t:Tag {key:$tag}) SET t.account=$account,t.region=$region,t.tag_key=$tag_key,t.tag_value=$tag_value MERGE (r)-[:HAS_TAG {provenance:$provenance}]->(t)");
    bounded(query)
}

fn tag_identity(key: &ResourceKey, tag: &ProjectionTag) -> Result<String, QueryError> {
    let mut digest = Sha256::new();
    [key.account.as_str(), key.region.as_str(), &tag.key, &tag.value]
        .into_iter()
        .try_for_each(|value| {
            let length = u64::try_from(value.len()).map_err(|_| QueryError::TooLarge)?;
            digest.update(length.to_be_bytes());
            digest.update(value.as_bytes());
            Ok::<_, QueryError>(())
        })?;
    hex(&digest.finalize())
}

/// Removes this resource's affiliations and their now-unreferenced vertices.
/// One query prevents interrupted replay from leaving orphan tag vertices.
pub(super) fn clear_tags(key: &ResourceKey) -> Result<String, QueryError> {
    let resource = hex(key.digest().as_bytes())?;
    let mut query = String::with_capacity(384);
    parameters(&mut query, &[("resource", Some(&resource))])?;
    query.push_str(" MATCH (r:Resource {key:$resource})-[link:HAS_TAG]->(t:Tag) WITH link,t LIMIT 1000 DELETE link WITH DISTINCT t WHERE NOT (:Resource)-[:HAS_TAG]->(t) DELETE t");
    bounded(query)
}

pub(super) fn tags_exist(key: &ResourceKey) -> Result<String, QueryError> {
    let resource = hex(key.digest().as_bytes())?;
    let mut query = String::with_capacity(192);
    parameters(&mut query, &[("resource", Some(&resource))])?;
    query.push_str(" MATCH (:Resource {key:$resource})-[:HAS_TAG]->(:Tag) RETURN 1 LIMIT 1");
    bounded(query)
}

fn parameters(query: &mut String, values: &[(&str, Option<&str>)]) -> Result<(), QueryError> {
    query.push_str("CYPHER");
    values.iter().try_for_each(|(name, value)| match value {
        Some(value) => write!(query, " {name}={}", quote(value)?).map_err(|_| QueryError::TooLarge),
        None => {
            query.push(' ');
            query.push_str(name);
            query.push_str("=null");
            Ok(())
        }
    })?;
    Ok(())
}

fn quote(value: &str) -> Result<String, QueryError> {
    let mut quoted = String::with_capacity(value.len().saturating_add(2));
    quoted.push('\'');
    for character in value.chars() {
        match character {
            '\\' => quoted.push_str("\\\\"),
            '\'' => quoted.push_str("\\'"),
            '\n' => quoted.push_str("\\n"),
            '\r' => quoted.push_str("\\r"),
            '\t' => quoted.push_str("\\t"),
            value if value.is_control() => {
                write!(quoted, "\\u{:04x}", u32::from(value)).map_err(|_| QueryError::TooLarge)?
            }
            value => quoted.push(value),
        }
    }
    quoted.push('\'');
    Ok(quoted)
}

fn identifier(value: &str) -> Result<String, QueryError> {
    if value.is_empty() || value.len() > 128 || value.chars().any(char::is_control) {
        return Err(QueryError::InvalidIdentifier);
    }
    Ok(format!("`{}`", value.replace('`', "``")))
}

fn hex(bytes: &[u8]) -> Result<String, QueryError> {
    let mut value = String::with_capacity(bytes.len().saturating_mul(2));
    bytes
        .iter()
        .try_for_each(|byte| write!(value, "{byte:02x}").map_err(|_| QueryError::TooLarge))?;
    Ok(value)
}

fn bounded(query: String) -> Result<String, QueryError> {
    if query.len() > MAX_QUERY_BYTES {
        return Err(QueryError::TooLarge);
    }
    Ok(query)
}

#[derive(Clone, Copy, Debug, thiserror::Error, PartialEq, Eq)]
pub(super) enum QueryError {
    #[error("projection relation identifier is invalid")]
    InvalidIdentifier,
    #[error("projection query exceeds its byte bound")]
    TooLarge,
    #[error("projection response is malformed or cleanup exceeds its page bound")]
    InvalidResponse,
}
#[cfg(test)]
mod tests {
    use super::{edge, node};
    use crate::inventory::{ProjectionEdgeHeader, ProjectionResource};
    use inventory_core::inventory::{AccountId, Kind, Region, Relation, ResourceId, ResourceKey};

    fn key(account: &str, id: &str) -> Result<ResourceKey, Box<dyn std::error::Error>> {
        Ok(ResourceKey {
            account: AccountId::parse(account)?,
            region: Region::parse("us-east-1")?,
            kind: Kind::parse("AWS::EC2::Vpc")?,
            id: ResourceId::parse(id)?,
        })
    }

    #[test]
    fn node_uses_full_scoped_digest_and_escaped_parameters()
    -> Result<(), Box<dyn std::error::Error>> {
        let source = key("123456789012", "vpc-'\\-1")?;
        let resource = ProjectionResource {
            key: source.clone(),
            name: Some("O'Brien\\Cloud\n".to_owned()),
            content_hash: [7; 32],
        };
        let query = node(&source, &resource)?;
        let digest = super::hex(source.digest().as_bytes())?;
        assert!(query.starts_with(&format!("CYPHER key='{digest}'")));
        assert!(query.contains("name='O\\'Brien\\\\Cloud\\n'"));
        assert!(query.contains("MERGE (n:Resource {key:$key})"));
        assert_ne!(source.digest(), key("000000000000", "vpc-'\\-1")?.digest());
        Ok(())
    }

    #[test]
    fn edge_keeps_relation_label_and_provenance_as_parameters()
    -> Result<(), Box<dyn std::error::Error>> {
        let source = key("123456789012", "vpc-a")?;
        let target = key("123456789012", "subnet-b")?;
        let relation = Relation::parse("CONTAINS")?;
        let header = ProjectionEdgeHeader {
            from: source,
            to: target,
            relation,
            source: "reviewed-rule".to_owned(),
        };
        let edge_query = edge(&header)?;
        assert!(edge_query.contains("[r:`CONTAINS` {relation:$relation,provenance:$provenance}]"));
        assert!(edge_query.contains("relation='CONTAINS'"));
        assert!(edge_query.contains("provenance='reviewed-rule'"));
        Ok(())
    }

    #[test]
    fn replacement_clears_incoming_and_outgoing_edges_and_deletion_detaches()
    -> Result<(), Box<dyn std::error::Error>> {
        let resource = key("123456789012", "vpc-a")?;
        let replacement = super::clear_edges(&resource)?;
        let deletion = super::delete(&resource)?;
        assert!(replacement.contains("MATCH (n:Resource {key:$key})-[r]-(other:Resource) DELETE r"));
        assert!(deletion.contains("DETACH DELETE n"));
        Ok(())
    }

    #[test]
    fn scoped_tag_pairs_are_shared_without_resource_cliques()
    -> Result<(), Box<dyn std::error::Error>> {
        let resource = key("123456789012", "vpc-a")?;
        let other = key("123456789012", "vpc-b")?;
        let pair = super::ProjectionTag { key: "team".into(), value: "platform".into() };
        assert_eq!(super::tag_identity(&resource, &pair)?, super::tag_identity(&other, &pair)?);
        assert_ne!(super::tag_identity(&resource, &pair)?, super::tag_identity(&key("000000000000", "vpc-a")?, &pair)?);
        let mut other_region = resource.clone();
        other_region.region = Region::parse("us-west-2")?;
        assert_ne!(super::tag_identity(&resource, &pair)?, super::tag_identity(&other_region, &pair)?);
        let query = super::tag(&resource, &pair)?;
        assert!(query.contains("MERGE (t:Tag {key:$tag})"));
        assert!(query.contains("MERGE (r)-[:HAS_TAG {provenance:$provenance}]->(t)"));
        assert!(query.contains("provenance='inventory.tags'"));
        assert!(!query.contains("(other:Resource)"));
        Ok(())
    }

    #[test]
    fn tag_data_is_bound_and_orphans_are_only_deleted_after_last_owner()
    -> Result<(), Box<dyn std::error::Error>> {
        let resource = key("123456789012", "vpc-a")?;
        let pair = super::ProjectionTag { key: "quote'\\\\".into(), value: "line\\n\\t'".into() };
        let query = super::tag(&resource, &pair)?;
        assert!(query.contains("tag_key='quote\\'\\\\\\\\'"));
        assert!(query.contains("t.tag_key=$tag_key,t.tag_value=$tag_value"));
        let cleanup = super::clear_tags(&resource)?;
        assert!(cleanup.contains("DELETE link WITH DISTINCT t"));
        assert!(cleanup.contains("WHERE NOT (:Resource)-[:HAS_TAG]->(t) DELETE t"));
        assert!(cleanup.contains("WITH link,t LIMIT 1000"));
        Ok(())
    }
}
