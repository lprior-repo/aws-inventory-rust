//! Snapshot identity plus wire-level provenance, status and coverage views.

use crate::raw::RawHeader;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt::Write as _;

/// Artifact identity written by `aws-inventory export`.
pub const SNAPSHOT_ARTIFACT: &str = "aws-inventory.snapshot";
/// The only snapshot format version this crate accepts.
pub const SUPPORTED_VERSION: u16 = 1;

/// Wire spelling of `header.provenance`.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Provenance {
    /// One preferred observation per identity.
    Normalized,
    /// Every persisted observation; the read model keeps the first identity.
    AllObservations,
}

impl Provenance {
    /// Lowercase wire spelling, also used in the fingerprint.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Normalized => "normalized",
            Self::AllObservations => "all_observations",
        }
    }
}

/// Wire spelling of `header.scan.status`.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScanStatus {
    /// The wire header says collection is still in progress.
    Running,
    /// The wire header reports a completed scan.
    Complete,
    /// The wire header reports incomplete collection coverage.
    Partial,
    /// The wire header reports collection was cancelled.
    Cancelled,
}

impl ScanStatus {
    /// Lowercase wire spelling, also used in the fingerprint.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Complete => "complete",
            Self::Partial => "partial",
            Self::Cancelled => "cancelled",
        }
    }
}

/// Versioned identity of one parsed snapshot.
///
/// `fingerprint` changes whenever any identity field or the optional
/// catalog/evidence digest changes, so a UI can detect a stale parse without
/// comparing whole inventories.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SnapshotMeta {
    /// Validated artifact spelling copied from `header.artifact`.
    pub artifact: String,
    /// Accepted wire version copied from `header.version`.
    pub version: u16,
    /// Observation mode declared by the header, before read-model deduplication.
    pub provenance: Provenance,
    /// Scan identity copied verbatim from the header.
    pub scan_id: String,
    /// Header-reported collection outcome; not recomputed from retained records.
    pub status: ScanStatus,
    /// Header start timestamp, in the export producer's time units.
    pub started_at: i64,
    /// Header finish timestamp, in the export producer's time units.
    pub finished_at: i64,
    /// Header resource count, not a recount of retained resource views.
    pub normalized_resources: u64,
    /// Header edge count, not a recount of retained edge views.
    pub normalized_edges: u64,
    /// Header-reported count of completed collection scopes.
    pub completed_scopes: u64,
    /// Header-reported count of failed collection scopes.
    pub failed_scopes: u64,
    /// Optional producer catalog digest; `None` means absent in the header.
    pub catalog_digest: Option<String>,
    /// Optional producer evidence digest; `None` means absent in the header.
    pub evidence_digest: Option<String>,
    /// Lowercase SHA-256 hex over header identity, counts and optional digests.
    pub fingerprint: String,
}

impl SnapshotMeta {
    /// SHA-256 hex digest over the canonical identity fields.
    pub fn fingerprint(&self) -> &str {
        self.fingerprint.as_str()
    }

    /// Build a meta view and its fingerprint from a parsed header.
    pub(crate) fn from_header(header: &RawHeader) -> Self {
        let mut meta = Self {
            artifact: header.artifact.clone(),
            version: header.version,
            provenance: header.provenance,
            scan_id: header.scan.scan_id.clone(),
            status: header.scan.status,
            started_at: header.scan.started_at,
            finished_at: header.scan.finished_at,
            normalized_resources: header.scan.resources,
            normalized_edges: header.scan.edges,
            completed_scopes: header.scan.completed_scopes,
            failed_scopes: header.scan.failed_scopes,
            catalog_digest: header.catalog_digest.clone(),
            evidence_digest: header.evidence_digest.clone(),
            fingerprint: String::new(),
        };
        let fingerprint = fingerprint(&meta);
        meta.fingerprint = fingerprint;
        meta
    }
}

fn fingerprint(meta: &SnapshotMeta) -> String {
    let mut canonical = String::with_capacity(256);
    push_identity(&mut canonical, meta);
    push_numbers(&mut canonical, meta);
    let digest = Sha256::digest(canonical.as_bytes());
    format!("{:x}", base16ct::HexDisplay(&digest))
}

fn push_identity(canonical: &mut String, meta: &SnapshotMeta) {
    push_str(canonical, &meta.artifact);
    push_str(canonical, meta.provenance.as_str());
    push_str(canonical, &meta.scan_id);
    push_str(canonical, meta.status.as_str());
    push_str(canonical, meta.catalog_digest.as_deref().unwrap_or("-"));
    push_str(canonical, meta.evidence_digest.as_deref().unwrap_or("-"));
}

fn push_numbers(canonical: &mut String, meta: &SnapshotMeta) {
    let _ = write!(
        canonical,
        "\u{1f}{}\u{1f}{}\u{1f}{}\u{1f}{}\u{1f}{}\u{1f}{}\u{1f}{}",
        meta.version,
        meta.started_at,
        meta.finished_at,
        meta.normalized_resources,
        meta.normalized_edges,
        meta.completed_scopes,
        meta.failed_scopes
    );
}

/// Length-prefixed so adjacent fields can never collide.
fn push_str(canonical: &mut String, value: &str) {
    let _ = write!(canonical, "{}\u{1f}{}\u{1e}", value.len(), value);
}

/// Collection coverage carried by the snapshot.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Coverage {
    /// Up to 500,000 scope records, retained in input order without deduplication.
    pub scopes: Vec<ScopeCoverage>,
    /// Up to 500,000 failure records, retained in input order without deduplication.
    pub failures: Vec<FailureView>,
    /// Additional input scope records were omitted from `scopes`.
    pub scopes_truncated: bool,
    /// Additional input failure records were omitted from `failures`.
    pub failures_truncated: bool,
}

/// One `scope` record from the export.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScopeCoverage {
    /// Account identity copied from the scope record.
    pub account_id: String,
    /// Region spelling copied from the scope record, including global scopes.
    pub region: String,
    /// Collector identity copied from the scope record.
    pub collector: String,
    /// Parent collection identity; `None` denotes an unparented scope.
    pub parent_id: Option<String>,
    /// The record says enumeration reached a terminal page.
    pub complete: bool,
    /// The record says the scope failed; independent of `complete`.
    pub failed: bool,
}

/// One `failure` record from the export.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FailureView {
    /// Account identity copied from the failure's scope.
    pub account_id: String,
    /// Region spelling copied from the failure's scope.
    pub region: String,
    /// Collector identity copied from the failure's scope.
    pub collector: String,
    /// Producer-supplied failure classification, without reinterpretation.
    pub category: String,
    /// Producer-supplied diagnostic text, retained verbatim.
    pub message: String,
}
