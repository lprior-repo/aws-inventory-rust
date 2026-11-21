//! SQLite-derived expected graph rows, shaped exactly as the legacy writer stores them.
//!
//! The writer (`crates/engine/src/graph.rs` `RESOURCE_QUERY`/`EDGE_QUERY` fed by
//! `graph/rows.rs`) stores each resource's properties and tags as serialized JSON text on the
//! `Resource` node, and each edge as a `RELATES_TO` relationship carrying `kind`/`source`.
//! Cells here are produced from the same core `Resource`/`Edge` types that were serialized
//! into `scan_resources`/`scan_edges`, so the expectation cannot drift from the stored payload.

use inventory_core::{Edge, Resource};
use serde_json::{Value, json};

use crate::HarnessError;

/// Node row for a resource persisted in `scan_resources`, in `verify_graph` column order.
pub(super) fn observed_node(
    scan_id: &str,
    row_id: &str,
    resource_json: &str,
) -> Result<Value, HarnessError> {
    let resource: Resource = serde_json::from_str(resource_json)
        .map_err(|e| HarnessError::invalid(format!("resource_json is not a resource: {e}")))?;
    if resource.id != row_id {
        return Err(HarnessError::invalid(format!(
            "resource_json id {} does not match scan_resources id {row_id}",
            resource.id
        )));
    }
    let properties = serde_json::to_string(&resource.properties)
        .map_err(|e| HarnessError::invalid(format!("properties serialization failed: {e}")))?;
    let tags = serde_json::to_string(&resource.tags)
        .map_err(|e| HarnessError::invalid(format!("tags serialization failed: {e}")))?;
    Ok(json!([
        resource.id,
        scan_id,
        "observed",
        format!("{scan_id}::{}", resource.id),
        resource.kind,
        resource.account_id,
        resource.region,
        resource.name.unwrap_or_default(),
        properties,
        tags,
        resource.source.as_str(),
        resource.observed_at,
        12,
        1
    ]))
}

/// Node row for an edge endpoint with no resource payload; the writer sets only key, id,
/// scan_id and `unresolved`.
pub(super) fn unresolved_node(scan_id: &str, id: &str) -> Value {
    json!([
        id,
        scan_id,
        "unresolved",
        format!("{scan_id}::{id}"),
        null,
        null,
        null,
        null,
        null,
        null,
        null,
        null,
        4,
        1
    ])
}

/// Relation row for an edge persisted in `scan_edges`, in `verify_graph` column order.
pub(super) fn edge_row(
    scan_id: &str,
    row_relation: &str,
    row_source: &str,
    edge_json: &str,
) -> Result<Value, HarnessError> {
    let edge: Edge = serde_json::from_str(edge_json)
        .map_err(|e| HarnessError::invalid(format!("edge_json is not an edge: {e}")))?;
    if edge.relation != row_relation || edge.source.as_str() != row_source {
        return Err(HarnessError::invalid(format!(
            "edge_json relation/source disagree with scan_edges row ({row_relation}, {row_source})"
        )));
    }
    let properties = serde_json::to_string(&edge.properties)
        .map_err(|e| HarnessError::invalid(format!("edge properties serialization failed: {e}")))?;
    Ok(json!([
        edge.from_id,
        edge.to_id,
        "RELATES_TO",
        edge.relation,
        edge.source.as_str(),
        scan_id,
        scan_id,
        edge.observed_at,
        properties,
        4,
        format!("{scan_id}::{}", edge.from_id),
        format!("{scan_id}::{}", edge.to_id)
    ]))
}
