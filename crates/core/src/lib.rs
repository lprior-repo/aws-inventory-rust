//! Synchronous inventory model. No runtime, network, database, or ambient clock.
#![forbid(unsafe_code)]
#![deny(unused_must_use)]

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use thiserror::Error;

mod selection;
pub use selection::{ResourceType, Selection, SelectionError};

pub const MAX_PAGE_RESOURCES: usize = 10_000;
pub const MAX_PAGE_EDGES: usize = 100_000;
pub const MAX_PROPERTY_BYTES: usize = 1_048_576;
pub const MAX_IDENTIFIER_BYTES: usize = 4_096;
pub const MAX_PAGE_BYTES: usize = 16_777_216;
pub const MAX_TOKEN_BYTES: usize = 8_192;
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

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Resource {
    pub id: String,
    pub account_id: String,
    pub region: String,
    pub kind: String,
    pub name: Option<String>,
    pub properties: Value,
    pub tags: BTreeMap<String, String>,
    pub source: Source,
    pub observed_at: i64,
}

impl Resource {
    pub fn validate(&self) -> Result<(), ValidationError> {
        validate_identifier(&self.id, "resource id")?;
        validate_identifier(&self.account_id, "account id")?;
        validate_identifier(&self.region, "region")?;
        validate_identifier(&self.kind, "resource kind")?;
        if let Some(name) = &self.name {
            validate_text(name, "resource name")?;
        }
        validate_json_size(&self.properties)?;
        if self.tags.len() > 100 {
            return Err(ValidationError::Limit("tags"));
        }
        if self.observed_at < 0 {
            return Err(ValidationError::InvalidField("observed_at"));
        }
        self.tags.iter().try_for_each(|(key, value)| {
            validate_identifier(key, "tag key")?;
            validate_text(value, "tag value")
        })?;
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Edge {
    pub from_id: String,
    pub to_id: String,
    pub relation: String,
    pub source: Source,
    pub observed_at: i64,
    pub properties: Value,
}

impl Edge {
    pub fn validate(&self) -> Result<(), ValidationError> {
        validate_identifier(&self.from_id, "edge source")?;
        validate_identifier(&self.to_id, "edge target")?;
        validate_identifier(&self.relation, "edge relation")?;
        if self.observed_at < 0 {
            return Err(ValidationError::InvalidField("edge observed_at"));
        }
        validate_json_size(&self.properties)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct CollectionScope {
    pub account_id: String,
    pub region: String,
    pub collector: String,
    pub parent_id: Option<String>,
    pub source: Source,
}

impl CollectionScope {
    pub fn validate(&self) -> Result<(), ValidationError> {
        validate_identifier(&self.account_id, "scope account")?;
        validate_identifier(&self.region, "scope region")?;
        validate_identifier(&self.collector, "scope collector")?;
        if let Some(parent) = &self.parent_id {
            validate_identifier(parent, "scope parent")?;
        }
        Ok(())
    }

    pub fn key(&self) -> Result<String, ValidationError> {
        self.validate()?;
        Ok(serde_json::to_string(self)?)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct CollectionPage {
    pub scope: CollectionScope,
    pub page_number: u32,
    pub next_token: Option<String>,
    pub resources: Vec<Resource>,
    pub edges: Vec<Edge>,
}

impl CollectionPage {
    pub fn validate(&self) -> Result<(), ValidationError> {
        self.scope.validate()?;
        if self.resources.len() > MAX_PAGE_RESOURCES || self.edges.len() > MAX_PAGE_EDGES {
            return Err(ValidationError::Limit("page records"));
        }
        if self.page_number == 0 || self.page_number > MAX_PAGES {
            return Err(ValidationError::Limit("page number"));
        }
        if self.next_token.as_ref().is_some_and(|token| {
            token.is_empty() || token.len() > MAX_TOKEN_BYTES || token.contains('\0')
        }) {
            return Err(ValidationError::InvalidField("continuation token"));
        }
        self.resources.iter().try_for_each(Resource::validate)?;
        self.edges.iter().try_for_each(Edge::validate)?;
        validate_serialized_size(self, MAX_PAGE_BYTES, "aggregate page bytes")
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct CollectionFailure {
    pub scope: CollectionScope,
    pub category: String,
    pub message: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "event", content = "data", rename_all = "snake_case")]
pub enum CollectionEvent {
    Page(CollectionPage),
    Failure(CollectionFailure),
}

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

/// Check a serialized byte budget without first allocating the serialized output.
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
