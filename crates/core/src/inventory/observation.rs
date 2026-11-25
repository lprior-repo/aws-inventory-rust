use super::{InventoryError, Relation, ResourceKey};
use crate::record::{ContentHash, Exclusions, Record};
use std::sync::Arc;

#[derive(Debug)]
pub struct Observation {
    key: ResourceKey,
    record: Record,
    content_hash: ContentHash,
    name: Option<Arc<str>>,
    exclusions: Exclusions,
    facet_precedence: u16,
}

impl Observation {
    pub fn new(
        key: ResourceKey,
        record: Record,
        exclusions: &Exclusions,
        name: Option<Arc<str>>,
    ) -> Result<Self, InventoryError> {
        if name.as_ref().is_some_and(|name| name.len() > 4096) {
            return Err(InventoryError::Name);
        }
        let content_hash = record.content_hash(exclusions)?;
        Ok(Self {
            key,
            record,
            content_hash,
            name,
            exclusions: exclusions.clone(),
            facet_precedence: 0,
        })
    }

    pub fn key(&self) -> &ResourceKey {
        &self.key
    }
    pub fn record(&self) -> &Record {
        &self.record
    }
    pub fn content_hash(&self) -> ContentHash {
        self.content_hash
    }
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Assigns the catalog-validated precedence of this operation's replaceable facet.
    pub fn with_facet_precedence(mut self, precedence: u16) -> Self {
        self.facet_precedence = precedence;
        self
    }

    pub fn facet_precedence(&self) -> u16 {
        self.facet_precedence
    }

    pub fn hash_exclusions(&self) -> &Exclusions {
        &self.exclusions
    }
}

impl AsRef<ResourceKey> for Observation {
    fn as_ref(&self) -> &ResourceKey {
        self.key()
    }
}

impl AsRef<ResourceKey> for ResourceKey {
    fn as_ref(&self) -> &ResourceKey {
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EdgeSource {
    Model,
    ReviewedRule,
    InferredExact,
}

impl EdgeSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Model => "model",
            Self::ReviewedRule => "reviewed-rule",
            Self::InferredExact => "inferred-exact",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Edge {
    pub from: ResourceKey,
    pub to: ResourceKey,
    pub relation: Relation,
    pub source: EdgeSource,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScopeOutcome {
    Complete,
    Partial,
    Inaccessible,
    Unsupported,
    Failed,
    Unobserved,
}

impl ScopeOutcome {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Complete => "complete",
            Self::Partial => "partial",
            Self::Inaccessible => "inaccessible",
            Self::Unsupported => "unsupported",
            Self::Failed => "failed",
            Self::Unobserved => "unobserved",
        }
    }
}

#[cfg(test)]
mod tests;
