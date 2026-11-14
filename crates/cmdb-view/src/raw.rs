use crate::meta::{Provenance, ScanStatus};
use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Deserialize)]
pub(crate) struct RawDocument {
    pub(crate) header: RawHeader,
    #[serde(default)]
    pub(crate) records: Vec<RawRecord>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RawHeader {
    pub(crate) artifact: String,
    pub(crate) version: u16,
    pub(crate) provenance: Provenance,
    pub(crate) scan: RawScan,
    #[serde(default)]
    pub(crate) catalog_digest: Option<String>,
    #[serde(default)]
    pub(crate) evidence_digest: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RawScan {
    pub(crate) scan_id: String,
    pub(crate) resources: u64,
    pub(crate) edges: u64,
    pub(crate) completed_scopes: u64,
    pub(crate) failed_scopes: u64,
    pub(crate) status: ScanStatus,
    pub(crate) started_at: i64,
    pub(crate) finished_at: i64,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "record", content = "data", rename_all = "snake_case")]
pub(crate) enum RawRecord {
    Resource(RawResourceRecord),
    Edge(RawEdgeRecord),
    Scope(RawScopeRecord),
    Failure(RawFailureRecord),
}

#[derive(Debug, Deserialize)]
pub(crate) struct RawResourceRecord {
    pub(crate) resource: RawResource,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RawResource {
    pub(crate) id: String,
    pub(crate) account_id: String,
    pub(crate) region: String,
    pub(crate) kind: String,
    #[serde(default)]
    pub(crate) name: Option<String>,
    #[serde(default)]
    pub(crate) properties: Value,
    #[serde(default)]
    pub(crate) tags: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RawEdgeRecord {
    pub(crate) edge: RawEdge,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RawEdge {
    pub(crate) from_id: String,
    pub(crate) to_id: String,
    pub(crate) relation: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RawScopeRecord {
    pub(crate) scope: RawCollectionScope,
    #[serde(default)]
    pub(crate) complete: bool,
    #[serde(default)]
    pub(crate) failed: bool,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RawFailureRecord {
    pub(crate) scope: RawCollectionScope,
    pub(crate) category: String,
    pub(crate) message: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RawCollectionScope {
    pub(crate) account_id: String,
    pub(crate) region: String,
    pub(crate) collector: String,
    #[serde(default)]
    pub(crate) parent_id: Option<String>,
}
