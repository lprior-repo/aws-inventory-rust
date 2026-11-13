//! Projects the filtered fixture graph onto the DAG model.
//!
//! The fixture carries flat resource rows plus resource-to-resource edges.
//! The builder lifts each row through its containment path
//! (account → region → kind → resource) and dedupes the shared prefixes, so
//! the result has four visual tiers plus the data-flow edges as overlays.

use crate::data::{Graph as DataGraph, Row};

use super::graph::{DagGraph, NodeKind, truncate_label};

/// Builds a capped, deduplicated DAG from the fixture projection.
pub fn from_data_graph(source: &DataGraph) -> DagGraph {
    let mut dag = DagGraph::default();
    let mut rows: Vec<&Row> = source.nodes.iter().collect();
    rows.sort_by_key(|row| (row.account, row.region, row.kind, row.id));
    for row in rows {
        add_row(&mut dag, row);
    }
    add_resource_edges(&mut dag, source);
    dag
}

fn add_row(dag: &mut DagGraph, row: &Row) {
    let Some((_, _, kind)) = add_containment(dag, row) else {
        return;
    };
    let label = truncate_label(row.name);
    let Some(resource) = dag.add_node(NodeKind::from_facet(row.kind), row.id, label) else {
        return;
    };
    dag.add_edge(kind, resource);
}

fn add_containment(dag: &mut DagGraph, row: &Row) -> Option<(usize, usize, usize)> {
    let account = dag.add_node(
        NodeKind::Account,
        format!("account:{}", row.account),
        row.account,
    )?;
    let region = dag.add_node(
        NodeKind::Region,
        format!("region:{}/{}", row.account, row.region),
        row.region,
    )?;
    let kind = dag.add_node(
        NodeKind::Kind,
        format!("kind:{}/{}/{}", row.account, row.region, row.kind),
        row.kind,
    )?;
    dag.add_edge(account, region);
    dag.add_edge(region, kind);
    Some((account, region, kind))
}

fn add_resource_edges(dag: &mut DagGraph, source: &DataGraph) {
    for edge in &source.edges {
        let from = dag.index_of(edge.from);
        let to = dag.index_of(edge.to);
        if let (Some(from), Some(to)) = (from, to) {
            dag.add_overlay_edge(from, to);
        }
    }
}
