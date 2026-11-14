//! Legacy JSON collection observations and explicit boundary checks.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

use super::{
    MAX_PAGE_BYTES, MAX_PAGE_EDGES, MAX_PAGE_RESOURCES, MAX_PAGES, MAX_TOKEN_BYTES, Source,
    ValidationError, validate_identifier, validate_json_size, validate_serialized_size,
    validate_text,
};

/// A legacy JSON observation; construction and deserialization do not run [`Self::validate`].
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Resource {
    /// Opaque resource identity; validation requires nonempty, NUL-free text of at most
    /// [`super::MAX_IDENTIFIER_BYTES`] bytes.
    pub id: String,
    /// Account label; validation requires nonempty, NUL-free text of at most
    /// [`super::MAX_IDENTIFIER_BYTES`] bytes.
    pub account_id: String,
    /// Region label; validation requires nonempty, NUL-free text of at most
    /// [`super::MAX_IDENTIFIER_BYTES`] bytes.
    pub region: String,
    /// Kind label; validation requires nonempty, NUL-free text of at most
    /// [`super::MAX_IDENTIFIER_BYTES`] bytes.
    pub kind: String,
    /// Optional display name; `None` is absent, while empty text is allowed by validation.
    /// When present, validation rejects NUL or more than [`super::MAX_IDENTIFIER_BYTES`] bytes.
    pub name: Option<String>,
    /// Arbitrary JSON value, limited by validation to [`super::MAX_PROPERTY_BYTES`] serialized bytes.
    pub properties: Value,
    /// Tags ordered by key; empty means no tags. Validation permits at most 100 entries,
    /// nonempty keys, and empty values; keys and values are NUL-free and at most
    /// [`super::MAX_IDENTIFIER_BYTES`] bytes.
    pub tags: BTreeMap<String, String>,
    /// Legacy provenance label, not an authorization to use hints as direct facts.
    pub source: Source,
    /// Producer-supplied observation time; the collectors write Unix milliseconds,
    /// but validation only rejects negative values.
    pub observed_at: i64,
}

impl Resource {
    /// Check the legacy text, tag, timestamp and serialized-property bounds explicitly.
    ///
    /// # Errors
    /// Returns `InvalidField` for empty ID/account/region/kind/tag keys; for NUL or more
    /// than [`super::MAX_IDENTIFIER_BYTES`] bytes in those fields, a present name or tag values;
    /// or for negative time.
    /// Returns `Limit` for more than 100 tags or properties exceeding `MAX_PROPERTY_BYTES`,
    /// and `Json` if property serialization fails.
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

/// A directed legacy relationship; construction and deserialization do not check its bounds.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Edge {
    /// Source resource identity; validation requires nonempty, NUL-free text of at most
    /// [`super::MAX_IDENTIFIER_BYTES`] bytes.
    pub from_id: String,
    /// Target resource identity; validation requires nonempty, NUL-free text of at most
    /// [`super::MAX_IDENTIFIER_BYTES`] bytes.
    pub to_id: String,
    /// Relation label; validation requires nonempty, NUL-free text of at most
    /// [`super::MAX_IDENTIFIER_BYTES`] bytes.
    pub relation: String,
    /// Legacy provenance label for the relationship observation.
    pub source: Source,
    /// Producer-supplied observation time; the collectors write Unix milliseconds,
    /// but validation only rejects negative values.
    pub observed_at: i64,
    /// Arbitrary JSON value, limited by validation to [`super::MAX_PROPERTY_BYTES`] serialized bytes.
    pub properties: Value,
}

impl Edge {
    /// Check text, timestamp and serialized-property bounds; endpoint existence is not checked.
    ///
    /// # Errors
    /// Returns `InvalidField` for empty or NUL-containing endpoint/relation text, or text
    /// exceeding [`super::MAX_IDENTIFIER_BYTES`] bytes, or negative time; `Limit` for properties
    /// exceeding `MAX_PROPERTY_BYTES`;
    /// `Json` if property serialization fails.
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

/// A legacy collection boundary; public fields and deserialization bypass [`Self::validate`].
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct CollectionScope {
    /// Account label; validation requires nonempty, NUL-free text of at most
    /// [`super::MAX_IDENTIFIER_BYTES`] bytes.
    pub account_id: String,
    /// Region label; validation requires nonempty, NUL-free text of at most
    /// [`super::MAX_IDENTIFIER_BYTES`] bytes.
    pub region: String,
    /// Collector label, not necessarily a service name; the same text bounds apply.
    pub collector: String,
    /// Optional parent label; `None` means no parent, otherwise the same text bounds apply.
    pub parent_id: Option<String>,
    /// Legacy provenance component of the scope key.
    pub source: Source,
}

impl CollectionScope {
    /// Check only the bounded-text contract of scope labels.
    ///
    /// # Errors
    /// Returns `InvalidField` if account, region, collector or a present parent is empty,
    /// contains NUL, or exceeds [`super::MAX_IDENTIFIER_BYTES`] bytes.
    /// No domain-specific syntax is checked.
    pub fn validate(&self) -> Result<(), ValidationError> {
        validate_identifier(&self.account_id, "scope account")?;
        validate_identifier(&self.region, "scope region")?;
        validate_identifier(&self.collector, "scope collector")?;
        if let Some(parent) = &self.parent_id {
            validate_identifier(parent, "scope parent")?;
        }
        Ok(())
    }

    /// Return the scope's JSON serialization after checking its text bounds.
    /// All fields, including source and an absent parent, participate in the key.
    ///
    /// # Errors
    /// Returns the errors of [`Self::validate`], or `Json` if serialization fails.
    pub fn key(&self) -> Result<String, ValidationError> {
        self.validate()?;
        Ok(serde_json::to_string(self)?)
    }
}

/// A legacy page whose bounds are checked only by explicitly calling [`Self::validate`].
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct CollectionPage {
    /// Collection boundary shared by this page; validation checks its text bounds only.
    pub scope: CollectionScope,
    /// One-based page number, accepted by validation through [`super::MAX_PAGES`] inclusive.
    pub page_number: u32,
    /// Continuation cursor; `None` marks the last page. Validation requires a present
    /// cursor to be nonempty, NUL-free and at most 8,192 bytes.
    pub next_token: Option<String>,
    /// Resource observations in supplied order; empty is allowed, even with a continuation.
    pub resources: Vec<Resource>,
    /// Relationship observations in supplied order; empty is allowed.
    pub edges: Vec<Edge>,
}

impl CollectionPage {
    /// Check scope, records, page number, cursor and the aggregate serialized JSON budget.
    /// Record identities need not be unique, and records need not match the scope.
    ///
    /// # Errors
    /// Returns scope/resource/edge validation errors; `Limit` if record counts exceed
    /// `MAX_PAGE_RESOURCES`/`MAX_PAGE_EDGES`, page number is outside `1..=MAX_PAGES`, or
    /// serialized bytes exceed `MAX_PAGE_BYTES`; `InvalidField` for an empty, NUL-containing
    /// or oversized cursor; `Json` if aggregate serialization fails.
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

/// A legacy failure report with no core validation method or construction-time bounds.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct CollectionFailure {
    /// Scope to which the report applies; construction does not check it.
    pub scope: CollectionScope,
    /// Producer-supplied failure classification; core imposes no vocabulary or text bounds.
    pub category: String,
    /// Producer-supplied diagnostic text; core imposes no text bounds here.
    pub message: String,
}

/// A legacy collection result; deserialization does not validate the contained value.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "event", content = "data", rename_all = "snake_case")]
pub enum CollectionEvent {
    /// One page, including an empty page that can still carry a continuation cursor.
    Page(CollectionPage),
    /// An explicit failure report rather than evidence of an empty scope.
    Failure(CollectionFailure),
}
