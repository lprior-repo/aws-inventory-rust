//! Typed failures at the snapshot, filter and graph boundaries.

use thiserror::Error;

/// Everything the read model can refuse.
///
/// Parsing never panics: malformed input, unsupported artifacts and bound
/// violations all surface here.
#[derive(Debug, Error)]
pub enum ViewError {
    /// The whole input exceeded the named byte bound before JSON parsing.
    #[error("snapshot input exceeds the {0} limit")]
    InputLimit(&'static str),
    /// Malformed JSON, an unknown record tag or a missing/ill-typed wire field.
    #[error("snapshot JSON: {0}")]
    Json(#[from] serde_json::Error),
    /// The header artifact is not the supported snapshot identity.
    #[error("unsupported snapshot artifact {0:?}; expected aws-inventory.snapshot")]
    UnsupportedArtifact(String),
    /// The header version is not the supported version.
    #[error("unsupported snapshot version {0}; expected 1")]
    UnsupportedVersion(u16),
    /// Raw resource or edge records exceeded the named count bound.
    #[error("snapshot record bound exceeded: {0}")]
    RecordLimit(&'static str),
    /// A requested page size is zero or greater than 10,000.
    #[error("page limit must be in 1..=10000")]
    InvalidPageLimit,
    /// A cursor is empty, longer than 20 bytes or not a platform-sized offset.
    #[error("page cursor is not a valid offset")]
    InvalidCursor,
    /// The requested neighborhood depth is outside `1..=2`.
    #[error("graph depth must be in 1..=2")]
    InvalidGraphDepth,
    /// A node or edge cap is zero or above its global maximum.
    #[error("graph node or edge cap is out of range")]
    InvalidGraphLimit,
    /// A kind, account or top-degree cap is zero or above its maximum.
    #[error("LOD bucket cap is out of range")]
    InvalidLodLimit,
    /// Checked property-byte, count or pagination arithmetic overflowed.
    #[error("snapshot size accounting overflowed")]
    SizeOverflow,
}
