//! Query-string shapes and their translation into view-model requests.

use crate::error::AppError;
use crate::{DEFAULT_GRAPH_EDGES, DEFAULT_GRAPH_NODES, DEFAULT_PAGE_LIMIT};
use axum::extract::Query;
use axum::extract::rejection::QueryRejection;
use cmdb_view::{GraphLimits, LodLimits, PageCursor, PageRequest, ResourceFilter};
use serde::Deserialize;

/// `GET /api/v1/resources` parameters.
#[derive(Debug, Default, Deserialize)]
pub struct ResourceQuery {
    pub account: Option<String>,
    pub region: Option<String>,
    pub kind: Option<String>,
    pub contains: Option<String>,
    pub limit: Option<usize>,
    pub cursor: Option<String>,
}

/// `GET /api/v1/graph/{id}` parameters.
#[derive(Debug, Default, Deserialize)]
pub struct GraphQuery {
    pub depth: Option<u8>,
    pub max_nodes: Option<usize>,
    pub max_edges: Option<usize>,
}

/// `GET /api/v1/lod` parameters.
#[derive(Debug, Default, Deserialize)]
pub struct LodQuery {
    pub top: Option<usize>,
    pub buckets: Option<usize>,
}

/// Convert an extractor result into a typed error on malformed input.
pub fn into_params<T>(value: Result<Query<T>, QueryRejection>) -> Result<T, AppError> {
    value
        .map(|Query(params)| params)
        .map_err(|_| AppError::malformed_query())
}

/// Build the filter and page request, preferring `cursor` over the start.
pub fn page_request(query: &ResourceQuery) -> Result<(ResourceFilter, PageRequest), AppError> {
    let filter = ResourceFilter {
        account_id: query.account.clone(),
        region: query.region.clone(),
        kind: query.kind.clone(),
        text: query.contains.clone(),
    };
    let limit = query.limit.unwrap_or(DEFAULT_PAGE_LIMIT);
    let request = match query.cursor.as_deref() {
        Some(cursor) => PageRequest::after(&PageCursor::parse(cursor)?, limit)?,
        None => PageRequest::new(0, limit)?,
    };
    Ok((filter, request))
}

/// Validated graph traversal limits.
pub fn graph_limits(query: &GraphQuery) -> Result<GraphLimits, AppError> {
    Ok(GraphLimits::new(
        query.depth.unwrap_or(1),
        query.max_nodes.unwrap_or(DEFAULT_GRAPH_NODES),
        query.max_edges.unwrap_or(DEFAULT_GRAPH_EDGES),
    )?)
}

/// Validated LOD caps; one `buckets` value bounds kinds and accounts alike.
pub fn lod_limits(query: &LodQuery) -> Result<LodLimits, AppError> {
    let defaults = LodLimits::default();
    Ok(LodLimits::new(
        query.buckets.unwrap_or(defaults.max_kinds()),
        query.buckets.unwrap_or(defaults.max_accounts()),
        query.top.unwrap_or(defaults.max_top_degree()),
    )?)
}
