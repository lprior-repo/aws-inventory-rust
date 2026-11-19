use super::{InventoryError, ResourceKey};

pub const MAX_RESOURCE_TAGS: usize = 100;
pub const MAX_TAG_KEY_CHARS: usize = 128;
pub const MAX_TAG_VALUE_CHARS: usize = 256;

/// A validated AWS tag pair. Empty values are meaningful and are retained verbatim.
/// Tag limits are measured in Unicode characters, not UTF-8 bytes, per AWS tag
/// documentation.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Tag {
    key: String,
    value: String,
}

impl Tag {
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Result<Self, InventoryError> {
        let key: String = key.into();
        if key.is_empty() || key.chars().count() > MAX_TAG_KEY_CHARS || key.chars().any(char::is_control) {
            return Err(InventoryError::TagKey);
        }
        let value: String = value.into();
        if value.chars().count() > MAX_TAG_VALUE_CHARS || value.contains('\0') {
            return Err(InventoryError::TagValue);
        }
        Ok(Self { key, value })
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TagCoverage {
    Complete,
    Partial,
    Failed,
    Unobserved,
}

impl TagCoverage {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Complete => "complete",
            Self::Partial => "partial",
            Self::Failed => "failed",
            Self::Unobserved => "unobserved",
        }
    }
}

/// Only a complete observation is authoritative enough to replace persisted tags.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TagObservation {
    resource: ResourceKey,
    coverage: TagCoverage,
    tags: Vec<Tag>,
}

impl TagObservation {
    pub fn new(
        resource: ResourceKey,
        coverage: TagCoverage,
        mut tags: Vec<Tag>,
    ) -> Result<Self, InventoryError> {
        if tags.len() > MAX_RESOURCE_TAGS {
            return Err(InventoryError::TagCount);
        }
        tags.sort_unstable_by(|left, right| left.key.cmp(&right.key));
        if tags.windows(2).any(|pair| match pair {
            [left, right] => left.key == right.key,
            _ => false,
        }) {
            return Err(InventoryError::DuplicateTag);
        }
        Ok(Self {
            resource,
            coverage,
            tags,
        })
    }

    pub fn resource(&self) -> &ResourceKey {
        &self.resource
    }

    pub fn coverage(&self) -> TagCoverage {
        self.coverage
    }

    pub fn tags(&self) -> &[Tag] {
        &self.tags
    }
}
