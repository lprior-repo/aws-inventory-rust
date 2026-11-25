use crate::encoding::{Query, digest, hex, identifier, quote};
use crate::{EdgeKey, EdgeRow, GraphError, MAX_EDGES, MAX_NODES, MAX_PAGE_SIZE, NodeKey, NodeRow};

pub const COMMAND: &str = "GRAPH.QUERY";
pub const READ_COMMAND: &str = "GRAPH.RO_QUERY";
pub(crate) const RESOURCE_INDEX: &str = "CREATE INDEX FOR (n:Resource) ON (n.key)";
pub(crate) const TAG_INDEX: &str = "CREATE INDEX FOR (n:Tag) ON (n.key)";
pub(crate) const EDGE_BATCH_ROWS: usize = 1_000;

const NODE_FIELDS: &str = "n.account,n.region,n.kind,n.id";
const EDGE_FIELDS: &str =
    "s.account,s.region,s.kind,s.id,t.account,t.region,t.kind,t.id,r.relation,r.provenance";
const EDGE_ORDER: &str = "s.key,t.key,r.relation,r.provenance";
const TRAVERSAL_FIELDS: &str = "startNode(r).account,startNode(r).region,startNode(r).kind,startNode(r).id,endNode(r).account,endNode(r).region,endNode(r).kind,endNode(r).id,r.relation,r.provenance";

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

pub(crate) fn has_nodes() -> &'static str {
    "MATCH (n) RETURN 1 LIMIT 1"
}

pub fn replace_edges(key: &NodeKey, edges: &[EdgeRow]) -> Result<String, GraphError> {
    edge_query(key, edges, true)
}

pub(crate) fn append_edges(key: &NodeKey, edges: &[EdgeRow]) -> Result<String, GraphError> {
    edge_query(key, edges, false)
}

fn edge_query(key: &NodeKey, edges: &[EdgeRow], clear: bool) -> Result<String, GraphError> {
    if edges.len() > EDGE_BATCH_ROWS {
        return Err(GraphError::Limits);
    }
    edges.iter().try_for_each(|edge| {
        if &edge.from != key && &edge.to != key {
            return Err(GraphError::UnrelatedEdge);
        }
        if edges
            .first()
            .is_some_and(|first| first.relation != edge.relation)
        {
            return Err(GraphError::Limits);
        }
        Ok(())
    })?;
    let mut query = keyed(key)?;
    edge_parameters(&mut query, edges)?;
    query.push(" MATCH (owner:Resource {key:$key})")?;
    if clear {
        query.push(
            " OPTIONAL MATCH (owner)-[old]-(other:Resource) DELETE old WITH DISTINCT owner",
        )?;
    }
    if let Some(edge) = edges.first() {
        edge_merge(&mut query, edge)?;
    }
    query.push(" RETURN owner.key")?;
    Ok(query.finish())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EdgeChunk<'a> {
    Clear,
    Append(&'a [EdgeRow]),
}

pub(crate) fn edge_chunks(edges: &[EdgeRow], clear: bool) -> impl Iterator<Item = EdgeChunk<'_>> {
    clear.then_some(EdgeChunk::Clear).into_iter().chain(
        edges
            .chunk_by(|left, right| left.relation == right.relation)
            .flat_map(|group| group.chunks(EDGE_BATCH_ROWS))
            .map(EdgeChunk::Append),
    )
}

pub(crate) fn edge_chunk_query(
    owner: &NodeKey,
    chunk: EdgeChunk<'_>,
) -> Result<String, GraphError> {
    match chunk {
        EdgeChunk::Clear => replace_edges(owner, &[]),
        EdgeChunk::Append(edges) => append_edges(owner, edges),
    }
}

fn edge_parameters(query: &mut Query, edges: &[EdgeRow]) -> Result<(), GraphError> {
    query.push(" rows=[")?;
    edges.iter().enumerate().try_for_each(|(index, edge)| {
        if index != 0 {
            query.push(",")?;
        }
        edge_parameter_row(query, edge)
    })?;
    query.push("]")
}

fn edge_parameter_row(query: &mut Query, edge: &EdgeRow) -> Result<(), GraphError> {
    let source = digest(&edge.from)?;
    let target = digest(&edge.to)?;
    query.push("{")?;
    [
        ("source", source.as_str()),
        ("target", target.as_str()),
        ("relation", edge.relation.as_str()),
        ("provenance", edge.provenance.as_str()),
    ]
    .into_iter()
    .enumerate()
    .try_for_each(|(index, (name, value))| {
        if index != 0 {
            query.push(",")?;
        }
        query.push(name)?;
        query.push(":")?;
        query.push(&quote(value)?)
    })?;
    query.push("}")
}

fn edge_merge(query: &mut Query, edge: &EdgeRow) -> Result<(), GraphError> {
    let label = identifier(edge.relation.as_str())?;
    query.push(&format!(" UNWIND $rows AS row OPTIONAL MATCH (s:Resource {{key:row.source}}),(t:Resource {{key:row.target}}) FOREACH (_ IN CASE WHEN s IS NOT NULL AND t IS NOT NULL THEN [1] ELSE [] END | MERGE (s)-[:{label} {{relation:row.relation,provenance:row.provenance}}]->(t)) WITH DISTINCT owner"))
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
            let source = digest(&edge.from)?;
            let target = digest(&edge.to)?;
            [
                ("source0", source.as_str()),
                ("target0", target.as_str()),
                ("relation0", edge.relation.as_str()),
                ("provenance0", edge.provenance.as_str()),
            ]
            .into_iter()
            .try_for_each(|(name, value)| query.parameter(name, Some(value)))?;
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
    let frontier = key_list(frontier)?;
    let expanded = key_list(expanded)?;
    let mut query = Query::new();
    query.push("CYPHER frontier=")?;
    query.push(&frontier)?;
    query.push(" excluded=")?;
    query.push(&expanded)?;
    query.push(" UNWIND $frontier AS frontier_key MATCH ")?;
    match direction {
        Direction::Both => query.push("(s:Resource {key:frontier_key})-[r]-(n:Resource) WHERE NOT n.key IN $excluded AND (NOT n.key IN $frontier OR startNode(r)=s)")?,
        Direction::Incoming => query.push("(n:Resource)-[r]->(s:Resource {key:frontier_key}) WHERE NOT n.key IN $excluded")?,
    }
    query.push(&format!(" RETURN {TRAVERSAL_FIELDS} LIMIT {limit}"))?;
    Ok(query.finish())
}

pub(crate) fn induced(keys: &[NodeKey], limit: usize) -> Result<String, GraphError> {
    if keys.len() > MAX_NODES || limit > MAX_EDGES.saturating_add(1) {
        return Err(GraphError::Limits);
    }
    let keys = key_list(keys)?;
    let mut query = Query::new();
    query.push(&format!(
        "MATCH (s:Resource)-[r]->(t:Resource) WHERE s.key IN {keys} AND t.key IN {keys} RETURN {EDGE_FIELDS} ORDER BY {EDGE_ORDER} LIMIT {limit}"
    ))?;
    Ok(query.finish())
}

fn key_list(keys: &[NodeKey]) -> Result<String, GraphError> {
    let mut query = Query::new();
    query.push("[")?;
    keys.iter().enumerate().try_for_each(|(index, key)| {
        if index != 0 {
            query.push(",")?;
        }
        query.push("'")?;
        query.push(&digest(key)?)?;
        query.push("'")
    })?;
    query.push("]")?;
    Ok(query.finish())
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

pub fn tag(key: &NodeKey, tag: &crate::TagRow) -> Result<String, GraphError> {
    let mut query = Query::new();
    query.push("CYPHER")?;
    let resource = digest(key)?;
    let identity = hex(&tag.identity)?;
    [
        ("resource", resource.as_str()),
        ("tag", identity.as_str()),
        ("account", key.account.as_str()),
        ("region", key.region.as_str()),
        ("tag_key", tag.key.as_str()),
        ("tag_value", tag.value.as_str()),
        ("provenance", "inventory.tags"),
    ]
    .into_iter()
    .try_for_each(|(name, value)| query.parameter(name, Some(value)))?;
    query.push(" MATCH (r:Resource {key:$resource}) MERGE (t:Tag {key:$tag}) SET t.account=$account,t.region=$region,t.tag_key=$tag_key,t.tag_value=$tag_value MERGE (r)-[:HAS_TAG {provenance:$provenance}]->(t)")?;
    Ok(query.finish())
}

pub fn clear_tags(key: &NodeKey) -> Result<String, GraphError> {
    let mut query = keyed(key)?;
    query.push(" MATCH (r:Resource {key:$key})-[link:HAS_TAG]->(t:Tag) WITH link,t LIMIT 1000 DELETE link WITH DISTINCT t WHERE NOT (:Resource)-[:HAS_TAG]->(t) DELETE t")?;
    Ok(query.finish())
}

pub fn tags_exist(key: &NodeKey) -> Result<String, GraphError> {
    let mut query = keyed(key)?;
    query.push(" MATCH (:Resource {key:$key})-[:HAS_TAG]->(:Tag) RETURN 1 LIMIT 1")?;
    Ok(query.finish())
}

pub fn tag_affiliations(key: &NodeKey, fetch: usize) -> Result<String, GraphError> {
    if fetch == 0 || fetch > 1001 {
        return Err(GraphError::Limits);
    }
    let mut query = keyed(key)?;
    [
        ("account", key.account.as_str()),
        ("region", key.region.as_str()),
        ("kind", key.kind.as_str()),
        ("id", key.id.as_str()),
    ]
    .into_iter()
    .try_for_each(|(name, value)| query.parameter(name, Some(value)))?;
    query.push(&format!(" MATCH (s:Resource {{key:$key,account:$account,region:$region,kind:$kind,id:$id}})-[r:HAS_TAG]->(t:Tag) WHERE t.account=$account AND t.region=$region AND r.provenance='inventory.tags' RETURN t.account,t.region,t.tag_key,t.tag_value,r.provenance ORDER BY t.tag_key,t.tag_value,r.provenance LIMIT {fetch}"))?;
    Ok(query.finish())
}

pub(crate) fn clear_page() -> &'static str {
    "MATCH (n) WITH n LIMIT 1000 DETACH DELETE n"
}

pub mod diagnostics {
    pub const RESOURCE_COUNT: &str = "MATCH (r:Resource) RETURN count(r)";
    pub const NODE_COUNT: &str = "MATCH (n) RETURN count(n)";
    pub const RESOURCE_NODE_COUNT: &str = "MATCH (n:Resource) RETURN count(n)";
    pub const SCAN_STATE: &str = "MATCH (s:InventoryScan) RETURN s.state";
    pub const LEGACY_EDGE_COUNT: &str = "MATCH ()-[e:RELATES_TO]->() RETURN count(e)";
    pub const LEGACY_RELATION_COUNT: &str = "MATCH ()-[r:RELATES_TO]->() RETURN count(r)";
    pub const LEGACY_NAME: &str =
        "MATCH (r:Resource) WHERE r.unresolved=false RETURN r.name LIMIT 1";
    pub const NODE_LABELS: &str = "MATCH (n) RETURN labels(n)[0]";
    pub const NODE_IDS: &str = "MATCH (n) RETURN n.id";
    pub const SORTED_RESOURCE_IDS: &str = "MATCH (r:Resource) RETURN r.id ORDER BY r.id";
    pub const TAG_KEYS: &str = "MATCH (t:Tag) RETURN t.tag_key";
    pub const TAG_PROVENANCE: &str = "MATCH ()-[r:HAS_TAG]->() RETURN r.provenance";
    pub const TAG_ROWS: &str = "MATCH (t:Tag) RETURN t.account,t.region,t.tag_key,t.tag_value";
    pub const RESOURCE_ROWS: &str =
        "MATCH (r:Resource) RETURN r.account,r.region,r.kind,r.id,r.name,r.content_hash";
    pub const RESOURCE_IDENTITIES: &str =
        "MATCH (r:Resource) RETURN r.account,r.region,r.kind,r.id";
    pub const NODE_IDENTITIES: &str =
        "MATCH (n:Resource) RETURN n.account,n.region,n.kind,n.id,n.key";
    pub const NODE_HASHES: &str =
        "MATCH (n:Resource) RETURN n.account,n.region,n.kind,n.id,n.key,n.content_hash";
    pub const UNNAMED_RESOURCES: &str = "MATCH (n:Resource) WHERE n.name IS NULL RETURN n.account,n.region,n.kind,n.id,n.content_hash";
    pub const RESOURCE_EDGES: &str = "MATCH (a:Resource)-[r]->(b:Resource) RETURN a.account,a.region,a.kind,a.id,b.account,b.region,b.kind,b.id,r.relation,r.provenance";
    pub const SOURCE_TARGET_EDGES: &str = "MATCH (s:Resource)-[r]->(t:Resource) RETURN s.account,s.region,s.kind,s.id,t.account,t.region,t.kind,t.id,r.relation,r.provenance";
    pub const AFFILIATIONS: &str = "MATCH (r:Resource)-[a:HAS_TAG]->(t:Tag) RETURN r.account,r.region,r.kind,r.id,t.tag_key,t.tag_value,a.provenance";
    pub const SCOPED_AFFILIATIONS: &str = "MATCH (r:Resource)-[a:HAS_TAG]->(t:Tag) RETURN r.account,r.region,r.kind,r.id,t.account,t.region,t.tag_key,t.tag_value,a.provenance";
    pub const RELATION_TYPES: &str = "MATCH ()-[r]->() RETURN type(r)";
    #[cfg(any(test, feature = "test-fixtures"))]
    pub const CORRUPT_ACCOUNT: &str = "MATCH (n:Resource) SET n.account='invalid'";
    #[cfg(any(test, feature = "test-fixtures"))]
    pub const NON_IDEMPOTENT_RESOURCE_FIXTURE: &str =
        "CREATE (r:Resource {id:'custom-non-idempotent'})";
    pub const TAG_MERGE: &str = "MERGE (t:Tag";
    pub const NODE_MERGE: &str = "MERGE (n:Resource";
    pub const NODE_UPSERT_PATTERN: &str = "MERGE (n:Resource {key:$key})";
    pub const TAG_UPSERT_PATTERN: &str = "MERGE (t:Tag {key:$tag})";
    pub const TAG_LINK_UPSERT_PATTERN: &str = "MERGE (r)-[:HAS_TAG {provenance:$provenance}]->(t)";
    #[cfg(any(test, feature = "test-fixtures"))]
    pub const HOSTILE_TAG_VALUE: &str = "'\\%_`) MATCH (n) DETACH DELETE n //";
    #[cfg(any(test, feature = "test-fixtures"))]
    pub const HOSTILE_PROPERTY: &str = "x\"}) MATCH (n) DETACH DELETE n //";

    pub fn legacy_nodes(limit: usize) -> String {
        format!(
            "MATCH (r:Resource) RETURN r.id,r.scan_id,CASE WHEN r.unresolved=true THEN 'unresolved' WHEN r.unresolved=false THEN 'observed' ELSE 'invalid' END,r.key,r.kind,r.account_id,r.region,r.name,r.properties_json,r.tags_json,r.source,r.observed_at,size(keys(r)),size(labels(r)) ORDER BY r.id LIMIT {limit}"
        )
    }

    pub fn legacy_edges(limit: usize) -> String {
        format!(
            "MATCH (a)-[e]->(b) WHERE type(e) <> 'HAS_TAG' RETURN a.id,b.id,type(e),e.kind,e.source,a.scan_id,b.scan_id,e.observed_at,e.properties_json,size(keys(e)),a.key,b.key ORDER BY a.id,b.id,e.kind,e.source LIMIT {limit}"
        )
    }

    pub fn legacy_tags(limit: usize) -> String {
        format!(
            "MATCH (r:Resource)-[e:HAS_TAG]->(t:Tag) RETURN r.id,r.key,t.account,t.region,t.tag_key,t.tag_value,e.provenance ORDER BY r.key,t.tag_key,t.tag_value,e.provenance LIMIT {limit}"
        )
    }

    pub fn legacy_marker(limit: usize) -> String {
        format!(
            "MATCH (s:InventoryScan) RETURN s.id,s.state,s.resources,s.edges,size(keys(s)),size(labels(s)) LIMIT {limit}"
        )
    }
    #[cfg(any(test, feature = "test-fixtures"))]
    pub fn resource_fixture<'a>(
        nodes: &'a [super::NodeRow],
        edges: &'a [super::EdgeRow],
    ) -> impl Iterator<Item = Result<String, super::GraphError>> + 'a {
        nodes.iter().map(super::node).chain(
            edges
                .iter()
                .map(|edge| super::append_edges(&edge.from, std::slice::from_ref(edge))),
        )
    }

    #[cfg(any(test, feature = "test-fixtures"))]
    pub fn foreign_tag_fixture(
        digest: &str,
        account: &str,
        region: &str,
    ) -> Result<String, super::GraphError> {
        let mut query = super::Query::new();
        query.push("CYPHER")?;
        [("digest", digest), ("account", account), ("region", region)]
            .into_iter()
            .try_for_each(|(name, value)| query.parameter(name, Some(value)))?;
        query.push(" MATCH (s:Resource {key:$digest}) CREATE (s)-[:HAS_TAG {provenance:'inventory.tags'}]->(:Tag {account:$account,region:$region,tag_key:'owner',tag_value:'after'}), (s)-[:HAS_TAG {provenance:'inventory.tags'}]->(:Tag {account:'222222222222',region:$region,tag_key:'owner',tag_value:'foreign'})")?;
        Ok(query.finish())
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum Inspection {
    Resources,
    Tags,
    TagLinks,
    Relationships,
    Nodes,
    DistinctResources,
    InvalidTagScopes,
    OrphanTags,
    InvalidTagAssignments(crate::falkor::inspect::TagFixturePhase),
    InvalidTagCardinality(crate::falkor::inspect::TagFixturePhase),
}

pub(crate) fn inspection(check: Inspection) -> &'static str {
    use crate::falkor::inspect::TagFixturePhase::{Initial, Updated};
    match check {
        Inspection::Resources => "MATCH (r:Resource) RETURN count(r)",
        Inspection::Tags => "MATCH (t:Tag) RETURN count(t)",
        Inspection::TagLinks => "MATCH ()-[e:HAS_TAG]->() RETURN count(e)",
        Inspection::Relationships => "MATCH ()-[e]->() RETURN count(e)",
        Inspection::Nodes => "MATCH (n) RETURN count(n)",
        Inspection::DistinctResources => "MATCH (r:Resource) RETURN count(DISTINCT r.key)",
        Inspection::InvalidTagScopes => {
            "MATCH (r:Resource)-[e:HAS_TAG]->(t:Tag) WHERE t.account IS NULL OR t.region IS NULL OR r.account<>t.account OR r.region<>t.region OR e.provenance IS NULL OR e.provenance<>'inventory.tags' RETURN count(e)"
        }
        Inspection::OrphanTags => {
            "MATCH (t:Tag) WHERE NOT (:Resource)-[:HAS_TAG]->(t) RETURN count(t)"
        }
        Inspection::InvalidTagAssignments(Initial) => {
            "MATCH (r:Resource)-[:HAS_TAG]->(t:Tag) WHERE t.tag_key IS NULL OR t.tag_value IS NULL OR NOT ((t.tag_key='environment' AND t.tag_value='bench') OR (t.tag_key='owner' AND t.tag_value='inventory')) RETURN count(t)"
        }
        Inspection::InvalidTagAssignments(Updated) => {
            "MATCH (r:Resource)-[:HAS_TAG]->(t:Tag) WHERE t.tag_key IS NULL OR t.tag_value IS NULL OR toInteger(substring(r.name,6))%4=0 OR NOT ((t.tag_key='owner' AND t.tag_value='inventory') OR (t.tag_key='environment' AND t.tag_value=CASE WHEN toInteger(substring(r.name,6))%4=1 THEN 'bench-updated' ELSE 'bench' END)) RETURN count(t)"
        }
        Inspection::InvalidTagCardinality(Initial) => {
            "MATCH (r:Resource) OPTIONAL MATCH (r)-[e:HAS_TAG]->(t:Tag) WITH r,count(e) AS links,count(DISTINCT t.tag_key) AS tag_keys WHERE links<>2 OR tag_keys<>2 RETURN count(r)"
        }
        Inspection::InvalidTagCardinality(Updated) => {
            "MATCH (r:Resource) OPTIONAL MATCH (r)-[e:HAS_TAG]->(t:Tag) WITH r,count(e) AS links,count(DISTINCT t.tag_key) AS tag_keys WHERE links<>CASE WHEN toInteger(substring(r.name,6))%4=0 THEN 0 ELSE 2 END OR tag_keys<>CASE WHEN toInteger(substring(r.name,6))%4=0 THEN 0 ELSE 2 END RETURN count(r)"
        }
    }
}

pub(crate) fn inspect_resources(expected: &[NodeRow]) -> Result<String, GraphError> {
    if expected.len() > 1000 {
        return Err(GraphError::Limits);
    }
    let mut query = Query::new();
    query.push("CYPHER")?;
    expected.iter().enumerate().try_for_each(|(index, row)| {
        let key = digest(&row.key)?;
        let hash = hex(&row.content_hash)?;
        let name = row.name.as_deref().ok_or(GraphError::InvalidRow)?;
        [
            ("key", key.as_str()),
            ("account", row.key.account.as_str()),
            ("region", row.key.region.as_str()),
            ("kind", row.key.kind.as_str()),
            ("id", row.key.id.as_str()),
            ("name", name),
            ("content_hash", hash.as_str()),
        ]
        .into_iter()
        .try_for_each(|(field, value)| query.parameter(&format!("{field}{index}"), Some(value)))
    })?;
    query.push(" UNWIND [")?;
    expected.iter().enumerate().try_for_each(|(index, _)| {
        if index != 0 {
            query.push(",")?;
        }
        query.push(&format!("{{key:$key{index},account:$account{index},region:$region{index},kind:$kind{index},id:$id{index},name:$name{index},content_hash:$content_hash{index}}}"))
    })?;
    query.push("] AS expected OPTIONAL MATCH (r:Resource {key:expected.key}) WITH expected,r WHERE r IS NULL OR r.account IS NULL OR r.account<>expected.account OR r.region IS NULL OR r.region<>expected.region OR r.kind IS NULL OR r.kind<>expected.kind OR r.id IS NULL OR r.id<>expected.id OR r.name IS NULL OR r.name<>expected.name OR r.content_hash IS NULL OR r.content_hash<>expected.content_hash RETURN count(expected)")?;
    Ok(query.finish())
}

#[cfg(test)]
#[path = "cypher/paged_tests.rs"]
mod paged_tests;

#[cfg(test)]
#[path = "cypher/query_tests.rs"]
mod query_tests;
