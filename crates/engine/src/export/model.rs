//! Wire envelopes and publication accounting for portable snapshots.
use super::*;

/// Versioned header emitted before data records in either export format.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ExportHeader {
    /// Artifact discriminator set to `aws-inventory.snapshot` by the writer.
    pub artifact: String,
    /// Wire format version, currently 1.
    pub version: u16,
    /// Whether data records contain preferred or all persisted observations.
    pub provenance: ProvenanceMode,
    /// Finalized scan identity, counts, status and timestamps from SQLite.
    pub scan: ExportScan,
}

/// Finalized scan metadata; counts describe normalized inventory even in all-observations mode.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ExportScan {
    /// Scan identity and normalized counts, flattened into the wire object.
    #[serde(flatten)]
    pub summary: ScanSummary,
    /// Persisted terminal outcome; a running scan cannot be exported.
    pub status: ScanStatus,
    /// Persisted scan start, in the caller-supplied store time units.
    pub started_at: i64,
    /// Persisted scan finish, in the same units as `started_at`.
    pub finished_at: i64,
}

/// Original collection location attached only in all-observations mode.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ObservationProvenance {
    /// Validated collection identity, including source and optional parent.
    pub scope: CollectionScope,
    /// One-based persisted page number, bounded by the collection page limit.
    pub page_number: u32,
}

/// One resource value with optional original observation location.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ExportResource {
    /// Preferred resource or original observation, according to header provenance.
    pub resource: Resource,
    /// Original scope/page, or `None` for normalized exports; omitted when absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observation: Option<ObservationProvenance>,
}

/// One relationship value with optional original observation location.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ExportEdge {
    /// Preferred relationship or original observation, according to provenance.
    pub edge: Edge,
    /// Original scope/page, or `None` for normalized exports; omitted when absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observation: Option<ObservationProvenance>,
}

/// Coverage evidence without opaque continuation tokens or page fingerprints.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ExportScope {
    /// Persisted collection identity, including source and optional parent.
    pub scope: CollectionScope,
    /// Last persisted page number; zero is permitted for a scope without pages.
    pub last_page: u32,
    /// Enumeration reached its terminal page; independent of the failure flag.
    pub complete: bool,
    /// A failure was persisted for the scope; does not imply resource absence.
    pub failed: bool,
}

/// Counts of emitted data records, not the normalized counts in the scan header.
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct ExportCounts {
    /// Emitted resource records, including repeats in all-observations mode.
    pub resources: u64,
    /// Emitted relationship records, including repeats in all-observations mode.
    pub edges: u64,
    /// Emitted collection-scope coverage records.
    pub scopes: u64,
    /// Emitted persisted failure records.
    pub failures: u64,
    /// Data records only; JSONL also has a header and a summary record.
    pub records: u64,
}

/// JSONL lines use this tagged envelope. JSON's records array contains only
/// Resource, Edge, Scope and Failure, with header/summary in top-level fields.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "record", content = "data", rename_all = "snake_case")]
pub enum ExportRecord {
    /// First JSONL line identifying the scan and format; separate in JSON.
    Header(ExportHeader),
    /// A resource value, emitted before edges, scopes and failures.
    Resource(ExportResource),
    /// A relationship value, emitted after resources and before coverage.
    Edge(ExportEdge),
    /// Collection coverage, including scopes that returned no resources.
    Scope(ExportScope),
    /// Persisted diagnostic; does not invent an absent resource.
    Failure(CollectionFailure),
    /// Final JSONL line with emitted counts; separate in JSON.
    Summary(ExportCounts),
}

/// Publication receipt for a fully written artifact, including its exact byte digest.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ExportSummary {
    /// Persisted identity of the exported scan.
    pub scan_id: String,
    /// Persisted collection outcome, including opted-in partial/cancelled scans.
    pub status: ScanStatus,
    /// Container format used for the published bytes.
    pub format: ExportFormat,
    /// Observation mode used for the published data records.
    pub provenance: ProvenanceMode,
    /// Emitted data-record counts, flattened into the serialized receipt.
    #[serde(flatten)]
    pub counts: ExportCounts,
    /// Exact output byte count, including envelopes, punctuation and newline.
    pub bytes: u64,
    /// Lowercase SHA-256 hex digest over every published byte.
    pub sha256: String,
}
