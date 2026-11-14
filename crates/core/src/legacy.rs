#![forbid(unsafe_code)]
#![deny(unused_must_use)]

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

mod observations;

mod selection;
pub use selection::{ResourceType, Selection, SelectionError};

pub const MAX_PAGE_RESOURCES: usize = 10_000;

pub const MAX_PAGE_EDGES: usize = 100_000;

pub const MAX_PROPERTY_BYTES: usize = 1_048_576;

pub const MAX_IDENTIFIER_BYTES: usize = 4_096;

pub const MAX_PAGE_BYTES: usize = 16_777_216;

pub(crate) const MAX_TOKEN_BYTES: usize = 8_192;

pub const MAX_PAGES: u32 = 100_000;

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("invalid {0}")]
    InvalidField(&'static str),

    #[error("resource bound exceeded: {0}")]
    Limit(&'static str),

    #[error("JSON serialization: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    Direct,

    CloudControl,

    AwsConfig,

    ResourceExplorer,
}

impl Source {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Direct => "direct",
            Self::CloudControl => "cloud_control",
            Self::AwsConfig => "aws_config",
            Self::ResourceExplorer => "resource_explorer",
        }
    }

    pub const fn priority(self) -> u8 {
        match self {
            Self::Direct => 4,
            Self::CloudControl => 3,
            Self::AwsConfig => 2,
            Self::ResourceExplorer => 1,
        }
    }
}

pub use observations::{
    CollectionEvent, CollectionFailure, CollectionPage, CollectionScope, Edge, Resource,
};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScanStatus {
    Running,

    Complete,

    Partial,

    Cancelled,
}

impl ScanStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Complete => "complete",
            Self::Partial => "partial",
            Self::Cancelled => "cancelled",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ScanSummary {
    pub scan_id: String,

    pub resources: u64,

    pub edges: u64,

    pub completed_scopes: u64,

    pub failed_scopes: u64,
}

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
