//! JSON projections for graph neighborhoods, LOD summaries and coverage.

use crate::dto::ResourceDto;
use cmdb_view::{
    CountBucket, Coverage, DegreeEntry, EdgeView, FailureView, GraphLimits, GraphNode, LodLimits,
    LodSummary, ScopeCoverage, Subgraph,
};
use serde::Serialize;

/// One node of a graph response; `resource` is absent for dangling targets.
#[derive(Debug, Serialize)]
pub struct GraphNodeDto {
    pub id: String,
    pub present: bool,
    pub resource: Option<ResourceDto>,
}

impl From<&GraphNode<'_>> for GraphNodeDto {
    fn from(node: &GraphNode<'_>) -> Self {
        Self {
            id: node.id.to_owned(),
            present: node.is_present(),
            resource: node.resource.map(ResourceDto::from),
        }
    }
}

/// One directed edge of a graph response.
#[derive(Debug, Serialize)]
pub struct EdgeDto {
    pub from_id: String,
    pub to_id: String,
    pub relation: String,
}

impl From<&EdgeView> for EdgeDto {
    fn from(edge: &EdgeView) -> Self {
        Self {
            from_id: edge.from_id.clone(),
            to_id: edge.to_id.clone(),
            relation: edge.relation.clone(),
        }
    }
}

/// `GET /api/v1/graph/{id}`.
#[derive(Debug, Serialize)]
pub struct GraphResponse {
    pub fingerprint: String,
    pub root: String,
    pub depth: u8,
    pub max_nodes: usize,
    pub max_edges: usize,
    pub nodes: Vec<GraphNodeDto>,
    pub edges: Vec<EdgeDto>,
    pub truncated_nodes: bool,
    pub truncated_edges: bool,
    pub truncated_depth: bool,
}

impl GraphResponse {
    pub fn from_subgraph(fingerprint: &str, subgraph: &Subgraph<'_>, limits: GraphLimits) -> Self {
        Self {
            fingerprint: fingerprint.to_owned(),
            root: subgraph.root.id.to_owned(),
            depth: limits.depth(),
            max_nodes: limits.max_nodes(),
            max_edges: limits.max_edges(),
            nodes: subgraph.nodes.iter().map(GraphNodeDto::from).collect(),
            edges: subgraph
                .edges
                .iter()
                .map(|edge| EdgeDto::from(*edge))
                .collect(),
            truncated_nodes: subgraph.truncated_nodes,
            truncated_edges: subgraph.truncated_edges,
            truncated_depth: subgraph.truncated_depth,
        }
    }
}

/// One `(key, count)` LOD bucket.
#[derive(Debug, Serialize)]
pub struct BucketDto {
    pub key: String,
    pub count: usize,
}

impl From<&CountBucket> for BucketDto {
    fn from(bucket: &CountBucket) -> Self {
        Self {
            key: bucket.key.clone(),
            count: bucket.count,
        }
    }
}

/// One top-degree entry.
#[derive(Debug, Serialize)]
pub struct DegreeDto {
    pub id: String,
    pub kind: String,
    pub degree: usize,
}

impl From<&DegreeEntry<'_>> for DegreeDto {
    fn from(entry: &DegreeEntry<'_>) -> Self {
        Self {
            id: entry.id.to_owned(),
            kind: entry.kind.to_owned(),
            degree: entry.degree,
        }
    }
}

/// `GET /api/v1/lod`.
#[derive(Debug, Serialize)]
pub struct LodResponse {
    pub fingerprint: String,
    pub total_resources: usize,
    pub kinds: Vec<BucketDto>,
    pub accounts: Vec<BucketDto>,
    pub top_degree: Vec<DegreeDto>,
    pub truncated_kinds: bool,
    pub truncated_accounts: bool,
    pub truncated_top_degree: bool,
    pub max_kinds: usize,
    pub max_accounts: usize,
    pub max_top_degree: usize,
}

impl LodResponse {
    pub fn from_summary(fingerprint: &str, summary: &LodSummary<'_>, limits: LodLimits) -> Self {
        Self {
            fingerprint: fingerprint.to_owned(),
            total_resources: summary.total_resources,
            kinds: summary.kinds.iter().map(BucketDto::from).collect(),
            accounts: summary.accounts.iter().map(BucketDto::from).collect(),
            top_degree: summary.top_degree.iter().map(DegreeDto::from).collect(),
            truncated_kinds: summary.truncated_kinds,
            truncated_accounts: summary.truncated_accounts,
            truncated_top_degree: summary.truncated_top_degree,
            max_kinds: limits.max_kinds(),
            max_accounts: limits.max_accounts(),
            max_top_degree: limits.max_top_degree(),
        }
    }
}

/// One collection scope.
#[derive(Debug, Serialize)]
pub struct ScopeDto {
    pub account_id: String,
    pub region: String,
    pub collector: String,
    pub parent_id: Option<String>,
    pub complete: bool,
    pub failed: bool,
}

impl From<&ScopeCoverage> for ScopeDto {
    fn from(scope: &ScopeCoverage) -> Self {
        Self {
            account_id: scope.account_id.clone(),
            region: scope.region.clone(),
            collector: scope.collector.clone(),
            parent_id: scope.parent_id.clone(),
            complete: scope.complete,
            failed: scope.failed,
        }
    }
}

/// One collection failure.
#[derive(Debug, Serialize)]
pub struct FailureDto {
    pub account_id: String,
    pub region: String,
    pub collector: String,
    pub category: String,
    pub message: String,
}

impl From<&FailureView> for FailureDto {
    fn from(failure: &FailureView) -> Self {
        Self {
            account_id: failure.account_id.clone(),
            region: failure.region.clone(),
            collector: failure.collector.clone(),
            category: failure.category.clone(),
            message: failure.message.clone(),
        }
    }
}

/// `GET /api/v1/coverage`.
#[derive(Debug, Serialize)]
pub struct CoverageResponse {
    pub fingerprint: String,
    pub scopes: Vec<ScopeDto>,
    pub failures: Vec<FailureDto>,
    pub scopes_truncated: bool,
    pub failures_truncated: bool,
}

impl CoverageResponse {
    pub fn from_coverage(fingerprint: &str, coverage: &Coverage) -> Self {
        Self {
            fingerprint: fingerprint.to_owned(),
            scopes: coverage.scopes.iter().map(ScopeDto::from).collect(),
            failures: coverage.failures.iter().map(FailureDto::from).collect(),
            scopes_truncated: coverage.scopes_truncated,
            failures_truncated: coverage.failures_truncated,
        }
    }
}
