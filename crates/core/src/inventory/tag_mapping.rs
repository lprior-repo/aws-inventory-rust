use super::{InventoryError, tag::check_key};
use thiserror::Error;

/// Upper bound on tag keys listed for one role; bounds the derived view's candidate set.
pub const MAX_MAPPING_KEYS: usize = 16;

/// The three derived ownership roles. Their names are the persisted `role` values.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TagRole {
    Owner,
    Team,
    Application,
}

impl TagRole {
    pub const ALL: [Self; 3] = [Self::Owner, Self::Team, Self::Application];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Owner => "owner",
            Self::Team => "team",
            Self::Application => "application",
        }
    }

    fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|role| role.as_str() == name)
    }
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum TagMappingError {
    #[error("tag mapping is not valid TOML")]
    Syntax,
    #[error("tag mapping role {0:?} is not owner, team or application")]
    UnknownRole(String),
    #[error("tag mapping role {0} must be an array of tag-key strings")]
    NotKeyList(&'static str),
    #[error("tag mapping role {0} lists the same tag key twice")]
    DuplicateKey(&'static str),
    #[error("tag mapping role {0} exceeds {MAX_MAPPING_KEYS} tag keys")]
    TooManyKeys(&'static str),
    #[error(transparent)]
    Key(#[from] InventoryError),
}

/// Ordered tag keys per derived role. Position is priority: the first key whose
/// tag carries a nonempty value supplies the role's value.
///
/// The mapping is operator input. Sweeps never read or write it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TagMapping {
    owner: Vec<String>,
    team: Vec<String>,
    application: Vec<String>,
}

impl TagMapping {
    /// Parses `owner`, `team` and `application` arrays of tag keys. Missing roles are
    /// empty; any other top-level name, non-string item, invalid key or duplicate fails.
    pub fn parse(text: &str) -> Result<Self, TagMappingError> {
        let table: toml::Table = text.parse().map_err(|_| TagMappingError::Syntax)?;
        let mut mapping = Self::default();
        for (name, value) in &table {
            let role = TagRole::from_name(name)
                .ok_or_else(|| TagMappingError::UnknownRole(name.clone()))?;
            *mapping.slot_mut(role) = key_list(role, value)?;
        }
        Ok(mapping)
    }

    pub fn keys(&self, role: TagRole) -> &[String] {
        match role {
            TagRole::Owner => &self.owner,
            TagRole::Team => &self.team,
            TagRole::Application => &self.application,
        }
    }

    fn slot_mut(&mut self, role: TagRole) -> &mut Vec<String> {
        match role {
            TagRole::Owner => &mut self.owner,
            TagRole::Team => &mut self.team,
            TagRole::Application => &mut self.application,
        }
    }
}

fn key_list(role: TagRole, value: &toml::Value) -> Result<Vec<String>, TagMappingError> {
    let items = value
        .as_array()
        .ok_or(TagMappingError::NotKeyList(role.as_str()))?;
    if items.len() > MAX_MAPPING_KEYS {
        return Err(TagMappingError::TooManyKeys(role.as_str()));
    }
    let mut keys: Vec<String> = Vec::with_capacity(items.len());
    for item in items {
        let key = item
            .as_str()
            .ok_or(TagMappingError::NotKeyList(role.as_str()))?;
        check_key(key)?;
        if keys.iter().any(|existing| existing == key) {
            return Err(TagMappingError::DuplicateKey(role.as_str()));
        }
        keys.push(key.to_owned());
    }
    Ok(keys)
}

#[cfg(test)]
mod tests;
