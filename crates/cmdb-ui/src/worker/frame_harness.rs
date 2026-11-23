#[cfg(test)]
mod tests;

use super::projection::graph_node_key;
use crate::dag::graph::{DagGraph, MAX_EDGES, MAX_NODES, NodeKind};
use cmdb_view::{
    Database, DatabaseSnapshot, EdgePageRequest, GraphNode, PageRequest, ResourceFilter,
};

pub(super) async fn load(database: &Database) -> Result<(DagGraph, u64, u64), String> {
    let mut snapshot = database
        .snapshot()
        .await
        .map_err(|error| error.to_string())?;
    let summary = snapshot
        .lod_summary()
        .await
        .map_err(|error| error.to_string())?;
    let (mut graph, nodes) = load_nodes(&mut snapshot).await?;
    if summary.edges > 0 {
        load_edges(&mut snapshot, &mut graph, &nodes).await?;
    }
    snapshot.finish().await.map_err(|error| error.to_string())?;
    Ok((graph, summary.resources, summary.edges))
}

async fn load_nodes(snapshot: &mut DatabaseSnapshot) -> Result<(DagGraph, Vec<GraphNode>), String> {
    let mut graph = DagGraph::default();
    let mut nodes = Vec::with_capacity(MAX_NODES);
    let mut after = None;
    for _ in 0..MAX_NODES.div_ceil(cmdb_view::MAX_PAGE_LIMIT) {
        let remaining = MAX_NODES.saturating_sub(nodes.len());
        let request = PageRequest {
            limit: remaining.min(cmdb_view::MAX_PAGE_LIMIT),
            after,
        };
        let page = snapshot
            .page_rows(&ResourceFilter::default(), request)
            .await
            .map_err(|error| error.to_string())?;
        page.rows
            .into_iter()
            .try_for_each(|row| insert_row(&mut graph, &mut nodes, row))?;
        after = page.next;
        if after.is_none() {
            break;
        }
    }
    Ok((graph, nodes))
}

fn insert_row(
    graph: &mut DagGraph,
    nodes: &mut Vec<GraphNode>,
    row: cmdb_view::ResourceRow,
) -> Result<(), String> {
    let node = GraphNode {
        account: row.key.account.as_str().to_owned(),
        region: row.key.region.as_str().to_owned(),
        kind: row.key.kind.as_str().to_owned(),
        id: row.key.id.as_str().to_owned(),
    };
    let label = match row.name {
        Some(name) => name,
        None => node.id.clone(),
    };
    let kind = NodeKind::from_facet(&node.kind);
    let key = graph_node_key(&node)?;
    graph
        .add_node(kind, key, label)
        .ok_or_else(|| "frame harness resource cap reached unexpectedly".to_owned())?;
    nodes.push(node);
    Ok(())
}

async fn load_edges(
    snapshot: &mut DatabaseSnapshot,
    graph: &mut DagGraph,
    nodes: &[GraphNode],
) -> Result<(), String> {
    for node in nodes.iter().take(MAX_NODES) {
        if graph.edges().len() >= MAX_EDGES {
            break;
        }
        load_node_edges(snapshot, graph, node).await?;
    }
    Ok(())
}

async fn load_node_edges(
    snapshot: &mut DatabaseSnapshot,
    graph: &mut DagGraph,
    node: &GraphNode,
) -> Result<(), String> {
    let mut after = None;
    for _ in 0..MAX_EDGES.div_ceil(cmdb_view::MAX_PAGE_LIMIT) {
        let request = EdgePageRequest {
            limit: cmdb_view::MAX_PAGE_LIMIT,
            after,
        };
        let page = snapshot
            .edges_page(node, request)
            .await
            .map_err(|error| error.to_string())?;
        let mut edges = page.edges.into_iter();
        edges.try_for_each(|edge| insert_edge(graph, edge))?;
        after = page.next;
        if after.is_none() || graph.edges().len() >= MAX_EDGES {
            break;
        }
    }
    Ok(())
}

fn insert_edge(graph: &mut DagGraph, edge: cmdb_view::GraphEdge) -> Result<(), String> {
    let from = graph.index_of(&graph_node_key(&edge.from)?);
    let to = graph.index_of(&graph_node_key(&edge.to)?);
    if let (Some(from), Some(to)) = (from, to) {
        graph.add_overlay_edge(from, to, edge.relation, edge.provenance);
    }
    Ok(())
}
