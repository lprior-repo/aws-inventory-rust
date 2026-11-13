//! Typed failures at the snapshot, filter and graph boundaries.

use thiserror::Error;

/// Everything the read model can refuse.
///
/// Parsing never panics: malformed input, unsupported artifacts and bound
/// violations all surface here.
#[derive(Debug, Error)]
pub enum ViewError {
    #[error("snapshot input exceeds the {0} limit")]
    InputLimit(&'static str),
    #[error("snapshot JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("unsupported snapshot artifact {0:?}; expected aws-inventory.snapshot")]
    UnsupportedArtifact(String),
    #[error("unsupported snapshot version {0}; expected 1")]
    UnsupportedVersion(u16),
    #[error("snapshot record bound exceeded: {0}")]
    RecordLimit(&'static str),
    #[error("page limit must be in 1..=10000")]
    InvalidPageLimit,
    #[error("page cursor is not a valid offset")]
    InvalidCursor,
    #[error("graph depth must be in 1..=2")]
    InvalidGraphDepth,
    #[error("graph node or edge cap is out of range")]
    InvalidGraphLimit,
    #[error("LOD bucket cap is out of range")]
    InvalidLodLimit,
    #[error("snapshot size accounting overflowed")]
    SizeOverflow,
}
