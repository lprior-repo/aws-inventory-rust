//! Synchronous inventory model. No runtime, network, database, or ambient clock.
//!
//! The JSON collection types and their string identity fields are legacy representations
//! to be replaced by the record-based resource model. Their public fields and derived
//! deserialization do not enforce bounds: callers must invoke the explicit validation
//! methods. Identity text checks enforce only nonempty, NUL-free text of at most
//! [`MAX_IDENTIFIER_BYTES`] bytes, not domain-specific syntax.
#![deny(missing_docs)]
#![forbid(unsafe_code)]
#![deny(unused_must_use)]

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

mod legacy;

pub mod record;
mod selection;
pub use selection::{ResourceType, Selection, SelectionError};

/// Maximum resource observations accepted by [`CollectionPage::validate`], inclusive.
pub const MAX_PAGE_RESOURCES: usize = 10_000;
/// Maximum relationship observations accepted by [`CollectionPage::validate`], inclusive.
pub const MAX_PAGE_EDGES: usize = 100_000;
/// Maximum serialized JSON bytes of the whole `properties` value of one resource or edge, inclusive.
pub const MAX_PROPERTY_BYTES: usize = 1_048_576;
/// Maximum UTF-8 bytes in identifier text and optional display/tag text, inclusive.
pub const MAX_IDENTIFIER_BYTES: usize = 4_096;
/// Maximum serialized JSON bytes for an entire collection page, inclusive.
pub const MAX_PAGE_BYTES: usize = 16_777_216;
/// Maximum UTF-8 bytes in a present continuation cursor, inclusive.
pub(crate) const MAX_TOKEN_BYTES: usize = 8_192;
/// Largest one-based page number accepted by [`CollectionPage::validate`].
pub const MAX_PAGES: u32 = 100_000;

/// A legacy model boundary check failed; string payloads name the rejected field or budget.
#[derive(Debug, Error)]
pub enum ValidationError {
    /// A field violates its text contract or a timestamp is negative.
    #[error("invalid {0}")]
    InvalidField(&'static str),
    /// A count or byte budget is exceeded, or a page number is outside its bounds.
    #[error("resource bound exceeded: {0}")]
    Limit(&'static str),
    /// JSON serialization failed for a reason other than the counting writer's byte budget.
    #[error("JSON serialization: {0}")]
    Json(#[from] serde_json::Error),
}

/// Legacy provenance labels, ordered by declaration rather than [`Self::priority`].
/// Hint labels do not authorize treating hints as direct service facts in the record model.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    /// Observation labelled as coming from a service API.
    Direct,
    /// Observation labelled as coming from the legacy Cloud Control path.
    CloudControl,
    /// Observation labelled as coming from the legacy AWS Config path.
    AwsConfig,
    /// Observation labelled as coming from the legacy Resource Explorer path.
    ResourceExplorer,
}

impl Source {
    /// Return the snake-case label also used by serde serialization.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Direct => "direct",
            Self::CloudControl => "cloud_control",
            Self::AwsConfig => "aws_config",
            Self::ResourceExplorer => "resource_explorer",
        }
    }

    /// Legacy precedence: direct 4, Cloud Control 3, Config 2, Resource Explorer 1.
    /// Larger numbers have higher priority; this is the reverse of derived enum ordering.
    pub const fn priority(self) -> u8 {
        match self {
            Self::Direct => 4,
            Self::CloudControl => 3,
            Self::AwsConfig => 2,
            Self::ResourceExplorer => 1,
        }
    }
}

pub use legacy::{
    CollectionEvent, CollectionFailure, CollectionPage, CollectionScope, Edge, Resource,
};

/// Legacy scan lifecycle label; this enum alone does not enforce state transitions.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScanStatus {
    /// Collection has not been finalized.
    Running,
    /// Collection was finalized as complete.
    Complete,
    /// Collection was finalized as partial: requested by the finisher, or downgraded from complete
    /// because no scope was recorded or a scope was incomplete or failed.
    Partial,
    /// Collection was finalized as cancelled.
    Cancelled,
}

impl ScanStatus {
    /// Return the snake-case label also used by serde serialization.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Complete => "complete",
            Self::Partial => "partial",
            Self::Cancelled => "cancelled",
        }
    }
}

/// Legacy per-scan totals; core imposes no construction-time identity or count checks.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ScanSummary {
    /// Opaque scan label; the summary itself does not check its text bounds.
    pub scan_id: String,
    /// Number of resources associated with the scan, not pages or source observations.
    pub resources: u64,
    /// Number of relationships associated with the scan.
    pub edges: u64,
    /// Number of scopes recorded as complete and not failed.
    pub completed_scopes: u64,
    /// Number of scopes recorded as failed, whether or not their pagination completed.
    pub failed_scopes: u64,
}

/// Check only nonempty, bounded, NUL-free text; `field` labels a rejection.
///
/// # Errors
/// Returns `InvalidField(field)` if `value` is empty, exceeds [`MAX_IDENTIFIER_BYTES`]
/// UTF-8 bytes, or contains NUL. No domain-specific syntax or normalization is applied.
pub fn validate_identifier(value: &str, field: &'static str) -> Result<(), ValidationError> {
    if value.is_empty() || value.len() > MAX_IDENTIFIER_BYTES || value.contains('\0') {
        return Err(ValidationError::InvalidField(field));
    }
    Ok(())
}

fn validate_json_size(value: &Value) -> Result<(), ValidationError> {
    validate_serialized_size(value, MAX_PROPERTY_BYTES, "properties JSON bytes")
}

fn validate_text(value: &str, field: &'static str) -> Result<(), ValidationError> {
    if value.len() > MAX_IDENTIFIER_BYTES || value.contains('\0') {
        return Err(ValidationError::InvalidField(field));
    }
    Ok(())
}

struct CountingWriter {
    used: usize,
    limit: usize,
    exceeded: bool,
}

impl std::io::Write for CountingWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        match self.used.checked_add(bytes.len()) {
            Some(total) if total <= self.limit => {
                self.used = total;
                Ok(bytes.len())
            }
            _ => {
                self.exceeded = true;
                Err(std::io::Error::other("serialized byte limit"))
            }
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// Check a serialized byte budget without first allocating the serialized output.
/// `limit` counts JSON bytes including escaping; equality is accepted and `label` names the budget.
///
/// # Errors
/// Returns `Limit(label)` when serialization would exceed `limit` or the byte counter
/// overflows; otherwise returns `Json` if the serializer fails.
pub fn validate_serialized_size<T: Serialize>(
    value: &T,
    limit: usize,
    label: &'static str,
) -> Result<(), ValidationError> {
    let mut counter = CountingWriter {
        used: 0,
        limit,
        exceeded: false,
    };
    let result = serde_json::to_writer(&mut counter, value);
    if counter.exceeded {
        return Err(ValidationError::Limit(label));
    }
    result.map_err(ValidationError::Json)
}
