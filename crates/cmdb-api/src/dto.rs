//! JSON projections of the view model: health, meta, resources and pages.

use cmdb_view::{
    PageCursor, Provenance, ResourcePage, ResourceView, ScanStatus, Snapshot, SnapshotMeta,
};
use serde::Serialize;
use std::collections::BTreeMap;

/// `GET /health`.
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub fingerprint: String,
}

impl HealthResponse {
    pub fn new(fingerprint: &str) -> Self {
        Self {
            status: "ok",
            fingerprint: fingerprint.to_owned(),
        }
    }
}

/// `GET /api/v1/meta`.
#[derive(Debug, Serialize)]
pub struct MetaResponse {
    pub fingerprint: String,
    pub artifact: String,
    pub version: u16,
    pub provenance: Provenance,
    pub scan_id: String,
    pub status: ScanStatus,
    pub started_at: i64,
    pub finished_at: i64,
    pub normalized_resources: u64,
    pub normalized_edges: u64,
    pub completed_scopes: u64,
    pub failed_scopes: u64,
    pub catalog_digest: Option<String>,
    pub evidence_digest: Option<String>,
    pub parsed_resources: usize,
    pub parsed_edges: usize,
}

impl MetaResponse {
    pub fn from_snapshot(snapshot: &Snapshot) -> Self {
        let meta: &SnapshotMeta = snapshot.meta();
        Self {
            fingerprint: snapshot.fingerprint().to_owned(),
            artifact: meta.artifact.clone(),
            version: meta.version,
            provenance: meta.provenance,
            scan_id: meta.scan_id.clone(),
            status: meta.status,
            started_at: meta.started_at,
            finished_at: meta.finished_at,
            normalized_resources: meta.normalized_resources,
            normalized_edges: meta.normalized_edges,
            completed_scopes: meta.completed_scopes,
            failed_scopes: meta.failed_scopes,
            catalog_digest: meta.catalog_digest.clone(),
            evidence_digest: meta.evidence_digest.clone(),
            parsed_resources: snapshot.resources().len(),
            parsed_edges: snapshot.edges().len(),
        }
    }
}

/// One resource in a page or detail response.
#[derive(Debug, Serialize)]
pub struct ResourceDto {
    pub id: String,
    pub account_id: String,
    pub region: String,
    pub kind: String,
    pub name: Option<String>,
    pub tags: BTreeMap<String, String>,
    pub properties_bytes: usize,
}

impl From<&ResourceView> for ResourceDto {
    fn from(view: &ResourceView) -> Self {
        Self {
            id: view.id.clone(),
            account_id: view.account_id.clone(),
            region: view.region.clone(),
            kind: view.kind.clone(),
            name: view.name.clone(),
            tags: view.tags.clone(),
            properties_bytes: view.properties_bytes,
        }
    }
}

/// `GET /api/v1/resources/{id}`.
#[derive(Debug, Serialize)]
pub struct ResourceResponse {
    pub fingerprint: String,
    pub resource: ResourceDto,
}

/// `GET /api/v1/resources`.
#[derive(Debug, Serialize)]
pub struct ResourcePageResponse {
    pub fingerprint: String,
    pub total_matching: usize,
    pub offset: usize,
    pub limit: usize,
    pub next_cursor: Option<String>,
    pub resources: Vec<ResourceDto>,
}

impl ResourcePageResponse {
    pub fn from_page(fingerprint: &str, page: &ResourcePage<'_>) -> Self {
        Self {
            fingerprint: fingerprint.to_owned(),
            total_matching: page.total_matching,
            offset: page.offset,
            limit: page.limit,
            next_cursor: page
                .next_cursor
                .as_ref()
                .map(|cursor: &PageCursor| cursor.as_str().to_owned()),
            resources: page
                .resources
                .iter()
                .map(|resource| ResourceDto::from(*resource))
                .collect(),
        }
    }
}
