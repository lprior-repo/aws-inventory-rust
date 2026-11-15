use crate::data::{Edge, Graph, Row};
use cmdb_view::{GraphEdge, GraphNode, StoredResource};
use inventory_core::inventory::{AccountId, Kind, Region, ResourceId, ResourceKey};

pub(super) fn row_from_resource(resource: &StoredResource) -> Result<Row, String> {
    let key = &resource.key;
    let hash = hex_digest(resource.content_hash.as_bytes())?;
    Ok(Row {
        account: key.account.as_str().to_owned(),
        region: key.region.as_str().to_owned(),
        kind: key.kind.as_str().to_owned(),
        id: key.id.as_str().to_owned(),
        name: resource.name.clone().map_or_else(|| key.id.as_str().to_owned(), |name| name),
        key: resource_key_text(key)?,
        record_bytes: Some(resource.record.as_bytes().len()),
        content_hash: Some(hash),
        tags: resource.tags.clone(),
        tags_truncated: resource.tags_truncated,
        tag_observation: resource.tag_observation,
    })
}

pub(super) fn graph_from_edges(edges: Vec<GraphEdge>) -> Result<Graph, String> {
    edges.iter().try_fold(Graph::default(), |mut graph, edge| {
        let from = graph_node_key(&edge.from)?;
        let to = graph_node_key(&edge.to)?;
        insert_graph_node(&mut graph, &edge.from, from.clone());
        insert_graph_node(&mut graph, &edge.to, to.clone());
        graph.edges.push(Edge {
            from,
            to,
            label: edge.relation.clone(),
            provenance: edge.provenance.clone(),
        });
        Ok(graph)
    })
}

fn insert_graph_node(graph: &mut Graph, node: &GraphNode, key: String) {
    if !graph.nodes.iter().any(|row| row.key == key) {
        graph.nodes.push(Row {
            account: node.account.clone(),
            region: node.region.clone(),
            kind: node.kind.clone(),
            id: node.id.clone(),
            name: node.id.clone(),
            key,
            record_bytes: None,
            content_hash: None,
            tags: Vec::new(),
            tags_truncated: false,
            tag_observation: None,
        });
    }
}

pub(super) fn graph_node_key(node: &GraphNode) -> Result<String, String> {
    let key = ResourceKey {
        account: AccountId::parse(&node.account).map_err(|error| error.to_string())?,
        region: Region::parse(&node.region).map_err(|error| error.to_string())?,
        kind: Kind::parse(&node.kind).map_err(|error| error.to_string())?,
        id: ResourceId::parse(&node.id).map_err(|error| error.to_string())?,
    };
    resource_key_text(&key)
}

fn resource_key_text(key: &ResourceKey) -> Result<String, String> {
    hex_digest(key.digest().as_bytes())
}

fn hex_digest(bytes: &[u8]) -> Result<String, String> {
    let mut digest = String::with_capacity(bytes.len().saturating_mul(2));
    bytes.iter().try_for_each(|byte| {
        std::fmt::Write::write_fmt(&mut digest, format_args!("{byte:02x}"))
            .map_err(|_| "failed to encode inventory hash".to_owned())
    })?;
    Ok(digest)
}
