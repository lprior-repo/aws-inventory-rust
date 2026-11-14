use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

use super::{
    MAX_PAGE_BYTES, MAX_PAGE_EDGES, MAX_PAGE_RESOURCES, MAX_PAGES, MAX_TOKEN_BYTES, Source,
    ValidationError, validate_identifier, validate_json_size, validate_serialized_size,
    validate_text,
};

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
