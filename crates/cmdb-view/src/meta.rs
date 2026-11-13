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
    Running,
    Complete,
    Partial,
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
    pub scopes: Vec<ScopeCoverage>,
    pub failures: Vec<FailureView>,
    pub scopes_truncated: bool,
    pub failures_truncated: bool,
}

/// One `scope` record from the export.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScopeCoverage {
    pub account_id: String,
    pub region: String,
    pub collector: String,
    pub parent_id: Option<String>,
    pub complete: bool,
    pub failed: bool,
}

/// One `failure` record from the export.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FailureView {
    pub account_id: String,
    pub region: String,
    pub collector: String,
    pub category: String,
    pub message: String,
}
