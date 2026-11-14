use super::*;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ExportHeader {
    pub artifact: String,

    pub version: u16,

    pub provenance: ProvenanceMode,

    pub scan: ExportScan,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ExportScan {
    #[serde(flatten)]
    pub summary: ScanSummary,

    pub status: ScanStatus,

    pub started_at: i64,

    pub finished_at: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ObservationProvenance {
    pub scope: CollectionScope,

    pub page_number: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ExportResource {
    pub resource: Resource,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observation: Option<ObservationProvenance>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ExportEdge {
    pub edge: Edge,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observation: Option<ObservationProvenance>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ExportScope {
    pub scope: CollectionScope,

    pub last_page: u32,

    pub complete: bool,

    pub failed: bool,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct ExportCounts {
    pub resources: u64,

    pub edges: u64,

    pub scopes: u64,

    pub failures: u64,

    pub records: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "record", content = "data", rename_all = "snake_case")]
pub enum ExportRecord {
    Header(ExportHeader),

    Resource(ExportResource),

    Edge(ExportEdge),

    Scope(ExportScope),

    Failure(CollectionFailure),

    Summary(ExportCounts),
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ExportSummary {
    pub scan_id: String,

    pub status: ScanStatus,

    pub format: ExportFormat,

    pub provenance: ProvenanceMode,

    #[serde(flatten)]
    pub counts: ExportCounts,

    pub bytes: u64,

    pub sha256: String,
}
