mod format;
mod records;
#[cfg(test)]
mod tests;

use crate::store::{MAX_STREAM_RECORDS, Store, StoreError};
use format::ArtifactWriter;
use inventory_core::{
    CollectionFailure, CollectionScope, Edge, Resource, ScanStatus, ScanSummary, ValidationError,
    validate_identifier,
};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::future::Future;
use std::path::{Path, PathBuf};
use thiserror::Error;
use tokio::fs::File;
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio_util::sync::CancellationToken;

pub const EXPORT_ARTIFACT: &str = "aws-inventory.snapshot";

pub const EXPORT_VERSION: u16 = 1;

pub const MAX_EXPORT_RECORDS: usize = MAX_STREAM_RECORDS;

pub const MAX_EXPORT_BYTES: u64 = 1_099_511_627_776;
pub(crate) const MAX_EXPORT_RECORD_BYTES: usize = 2_359_296;
const FILE_BUFFER_BYTES: usize = 65_536;

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExportFormat {
    Json,

    #[default]
    Jsonl,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProvenanceMode {
    #[default]
    Normalized,

    AllObservations,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ReferenceMode {
    #[default]
    Disabled,

    Derive,
}

impl ReferenceMode {
    pub const fn from_flag(enabled: bool) -> Self {
        if enabled {
            Self::Derive
        } else {
            Self::Disabled
        }
    }

    pub const fn is_enabled(self) -> bool {
        matches!(self, Self::Derive)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ExportOptions {
    pub format: ExportFormat,

    pub provenance: ProvenanceMode,

    pub allow_partial: bool,

    pub reference_mode: ReferenceMode,

    pub max_records: usize,

    pub max_bytes: u64,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            format: ExportFormat::Jsonl,
            provenance: ProvenanceMode::Normalized,
            allow_partial: false,
            reference_mode: ReferenceMode::Disabled,
            max_records: MAX_EXPORT_RECORDS,
            max_bytes: MAX_EXPORT_BYTES,
        }
    }
}

impl ExportOptions {
    fn validate(self) -> Result<(), ExportError> {
        if !(1..=MAX_EXPORT_RECORDS).contains(&self.max_records) {
            return Err(ExportError::Invalid("max_records must be in 1..=10000000"));
        }
        if !(1..=MAX_EXPORT_BYTES).contains(&self.max_bytes) {
            return Err(ExportError::Invalid(
                "max_bytes must be in 1..=1099511627776",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum ExportError {
    #[error("inventory: {0}")]
    Store(#[from] StoreError),

    #[error("SQLx inventory read: {0}")]
    Sql(#[from] sqlx::Error),

    #[error("file I/O: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON: {0}")]
    Json(#[from] serde_json::Error),

    #[error("inventory validation: {0}")]
    Validation(#[from] ValidationError),

    #[error("export validation: {0}")]
    Invalid(&'static str),

    #[error("export bound exceeded: {0}")]
    Limit(&'static str),

    #[error("export cancelled before publication")]
    Cancelled,

    #[error("{cause}; partial-file cleanup failed (drain: {drain:?}, removal: {removal:?})")]
    Cleanup {
        #[source]
        cause: Box<Self>,

        drain: Option<std::io::Error>,

        removal: Option<std::io::Error>,
    },

    #[error(
        "export was published at {output:?}, but its partial-file link could not be removed: {source}"
    )]
    PublishedCleanup {
        output: PathBuf,

        #[source]
        source: std::io::Error,
    },

    #[error("{cause}; restoring the pre-export reference edges failed: {restore}")]
    Restore {
        #[source]
        cause: Box<Self>,

        restore: Box<StoreError>,
    },
}

mod model;
pub use model::*;
mod publication;
pub use publication::*;
