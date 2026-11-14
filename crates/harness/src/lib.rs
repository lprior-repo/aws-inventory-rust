//! Development harness for local fixtures, oracle snapshots and packaging.
//!
//! The shipped collector never depends on this crate. Every module here
//! reproduces a previously Python-held development contract in Rust so the
//! verification path and the application share one toolchain.
#![deny(missing_docs)]

pub mod assertions;
pub mod audit;
pub mod catalog;
pub mod emulator;
mod files;
pub mod install;
pub mod json;
pub mod package;
pub mod provenance;
pub mod proxy;
pub mod seed;
pub mod snapshot;

/// Bounded harness failures. Messages are the exact reasons a fixture or
/// receipt was rejected; they are surfaced to the operator, never to the
/// collector.
#[derive(Debug, thiserror::Error)]
pub enum HarnessError {
    /// Input did not satisfy its declared contract.
    #[error("{0}")]
    Invalid(String),
    /// Filesystem failure.
    #[error("{0}")]
    Io(#[from] std::io::Error),
    /// JSON decoding or encoding failure.
    #[error("{0}")]
    Json(#[from] serde_json::Error),
    /// SQLite access failure.
    #[error("{0}")]
    Sqlx(#[from] sqlx::Error),
}

impl HarnessError {
    /// Build an invalid-input failure with an operator-facing reason.
    pub fn invalid(reason: impl Into<String>) -> Self {
        Self::Invalid(reason.into())
    }
}
