use crate::encoding::{Query, digest, hex, identifier};
use crate::{EdgeKey, EdgeRow, GraphError, MAX_EDGES, MAX_NODES, MAX_PAGE_SIZE, NodeKey, NodeRow};

const NODE_FIELDS: &str = "n.account,n.region,n.kind,n.id";
const EDGE_FIELDS: &str =
    "s.account,s.region,s.kind,s.id,t.account,t.region,t.kind,t.id,r.relation,r.provenance";
const EDGE_ORDER: &str = "s.key,t.key,r.relation,r.provenance";

pub fn node(row: &NodeRow) -> Result<String, GraphError> {
    let mut query = Query::new();
    query.push("CYPHER")?;
    let key = &row.key;
    let hash = hex(&row.content_hash)?;
    let identity = digest(key)?;
    [
        ("key", Some(identity.as_str())),
        ("account", Some(key.account.as_str())),
        ("region", Some(key.region.as_str())),
        ("kind", Some(key.kind.as_str())),
        ("id", Some(key.id.as_str())),
        ("name", row.name.as_deref()),
        ("content_hash", Some(hash.as_str())),
    ]
    .into_iter()
    .try_for_each(|(name, value)| query.parameter(name, value))?;
    query.push(" MERGE (n:Resource {key:$key}) SET n.account=$account,n.region=$region,n.kind=$kind,n.id=$id,n.name=$name,n.content_hash=$content_hash")?;
    Ok(query.finish())
}

pub fn delete(key: &NodeKey) -> Result<String, GraphError> {
    let mut query = keyed(key)?;
    query.push(" MATCH (n:Resource {key:$key}) DETACH DELETE n")?;
    Ok(query.finish())
}

pub fn clear() -> &'static str {
    "MATCH (n) WITH n LIMIT 1000000 DETACH DELETE n"
}

pub(crate) fn has_nodes() -> &'static str {
    "MATCH (n) RETURN 1 LIMIT 1"
}

pub fn replace_edges(key: &NodeKey, edges: &[EdgeRow]) -> Result<String, GraphError> {
    if edges.len() > MAX_EDGES {
        return Err(GraphError::Limits);
    }
    let mut query = keyed(key)?;
    edges.iter().enumerate().try_for_each(|(index, edge)| {
        if &edge.from != key && &edge.to != key {
            return Err(GraphError::UnrelatedEdge);
        }
        edge_parameters(&mut query, edge, index)
    })?;
    query.push(" MATCH (owner:Resource {key:$key}) OPTIONAL MATCH (owner)-[old]-(other:Resource) DELETE old WITH DISTINCT owner")?;
    edges
        .iter()
        .enumerate()
        .try_for_each(|(index, edge)| edge_merge(&mut query, edge, index))?;
    query.push(" RETURN owner.key")?;
    Ok(query.finish())
}

fn edge_parameters(query: &mut Query, edge: &EdgeRow, index: usize) -> Result<(), GraphError> {
    let source = digest(&edge.from)?;
    let target = digest(&edge.to)?;
    [
        ("source", source.as_str()),
        ("target", target.as_str()),
        ("relation", edge.relation.as_str()),
        ("provenance", edge.provenance.as_str()),
    ]
    .into_iter()
    .try_for_each(|(name, value)| query.parameter(&format!("{name}{index}"), Some(value)))
}

fn edge_merge(query: &mut Query, edge: &EdgeRow, index: usize) -> Result<(), GraphError> {
    let label = identifier(edge.relation.as_str())?;
    query.push(&format!(" OPTIONAL MATCH (s:Resource {{key:$source{index}}}),(t:Resource {{key:$target{index}}}) FOREACH (_ IN CASE WHEN s IS NOT NULL AND t IS NOT NULL THEN [1] ELSE [] END | MERGE (s)-[:{label} {{relation:$relation{index},provenance:$provenance{index}}}]->(t)) WITH DISTINCT owner"))
}

fn keyed(key: &NodeKey) -> Result<Query, GraphError> {
    let mut query = Query::new();
    query.push("CYPHER")?;
    query.parameter("key", Some(&digest(key)?))?;
    Ok(query)
}

pub fn node_keys(after: Option<&NodeKey>, limit: u16) -> Result<String, GraphError> {
    page_limit(limit)?;
    let mut query = Query::new();
    let predicate = match after {
        Some(key) => {
            query.push("CYPHER")?;
            query.parameter("after", Some(&digest(key)?))?;
            " WHERE n.key > $after"
        }
        None => "",
    };
    query.push(&format!(
        " MATCH (n:Resource){predicate} RETURN {NODE_FIELDS} ORDER BY n.key LIMIT {limit}"
    ))?;
    Ok(query.finish())
}

pub fn edge_keys(after: Option<&EdgeKey>, limit: u16) -> Result<String, GraphError> {
    page_limit(limit)?;
    let mut query = Query::new();
    let predicate = match after {
        Some(edge) => {
            query.push("CYPHER")?;
            edge_parameters(&mut query, edge, 0)?;
            " WHERE s.key > $source0 OR (s.key=$source0 AND t.key > $target0) OR (s.key=$source0 AND t.key=$target0 AND r.relation > $relation0) OR (s.key=$source0 AND t.key=$target0 AND r.relation=$relation0 AND r.provenance > $provenance0)"
        }
        None => "",
    };
    query.push(&format!(" MATCH (s:Resource)-[r]->(t:Resource){predicate} RETURN {EDGE_FIELDS} ORDER BY {EDGE_ORDER} LIMIT {limit}"))?;
    Ok(query.finish())
}

pub(crate) fn layer(
    frontier: &[NodeKey],
    expanded: &[NodeKey],
    limit: usize,
    direction: Direction,
) -> Result<String, GraphError> {
    let valid = frontier.len() <= MAX_NODES && expanded.len() <= MAX_NODES;
    if !valid || limit > MAX_EDGES.saturating_add(1) {
        return Err(GraphError::Limits);
    }
    let mut query = Query::new();
    query.push("CYPHER frontier=")?;
    key_list(&mut query, frontier)?;
    query.push(" expanded=")?;
    key_list(&mut query, expanded)?;
    let predicate = match direction {
        Direction::Both => {
            "(s.key IN $frontier OR t.key IN $frontier) AND NOT (s.key IN $expanded OR t.key IN $expanded)"
        }
        Direction::Incoming => "t.key IN $frontier AND NOT s.key IN $expanded",
    };
    query.push(&format!(" MATCH (s:Resource)-[r]->(t:Resource) WHERE {predicate} RETURN {EDGE_FIELDS} ORDER BY {EDGE_ORDER} LIMIT {limit}"))?;
    Ok(query.finish())
}

fn key_list(query: &mut Query, keys: &[NodeKey]) -> Result<(), GraphError> {
    query.push("[")?;
    keys.iter().enumerate().try_for_each(|(index, key)| {
        if index != 0 {
            query.push(",")?;
        }
        query.push("'")?;
        query.push(&digest(key)?)?;
        query.push("'")
    })?;
    query.push("]")
}

fn page_limit(limit: u16) -> Result<(), GraphError> {
    if limit == 0 || limit > MAX_PAGE_SIZE {
        return Err(GraphError::Limits);
    }
    Ok(())
}

#[derive(Clone, Copy)]
pub(crate) enum Direction {
    Both,
    Incoming,
}
