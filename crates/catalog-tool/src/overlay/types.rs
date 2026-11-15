use super::NameError;
use crate::classes::DenyReason;
use std::{collections::BTreeMap, fmt};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct MemberPath(String);

impl MemberPath {
    pub fn new(value: &str) -> Result<Self, NameError> {
        if !super::valid_path(value) {
            return Err(NameError {
                value: value.to_owned(),
            });
        }
        Ok(Self(value.to_owned()))
    }

    pub(super) fn selected(value: &str) -> Result<Self, NameError> {
        if value == "$" {
            return Ok(Self(value.to_owned()));
        }
        Self::new(value)
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl fmt::Display for MemberPath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct OperationName(String);
impl OperationName {
    pub fn new(value: &str) -> Result<Self, NameError> {
        if value.len() > super::MAX_FILE_BYTES
            || !value.starts_with(|c: char| c.is_ascii_alphabetic())
            || !value.bytes().all(|b| b.is_ascii_alphanumeric())
        {
            return Err(NameError {
                value: value.to_owned(),
            });
        }
        Ok(Self(value.to_owned()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl fmt::Display for OperationName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ServiceOverlay {
    pub(super) service: String,
    pub(super) format: i64,
    pub(super) reviewed: String,
    pub(super) reviewer: String,
    pub(super) operations: BTreeMap<OperationName, OperationOverlay>,
}

impl ServiceOverlay {
    #[must_use]
    pub fn service(&self) -> &str {
        &self.service
    }

    #[must_use]
    pub fn format(&self) -> i64 {
        self.format
    }

    #[must_use]
    pub fn reviewed(&self) -> &str {
        &self.reviewed
    }

    #[must_use]
    pub fn reviewer(&self) -> &str {
        &self.reviewer
    }

    #[must_use]
    pub fn operations(&self) -> &BTreeMap<OperationName, OperationOverlay> {
        &self.operations
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum OverlayClass {
    Enumerate,

    Expand,

    Deny(DenyReason),

    Defer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Scope {
    Region,

    Account,

    HomeRegion,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct OperationOverlay {
    pub class: Option<OverlayClass>,
    pub defer_reason: Option<String>,

    pub scope: Option<Scope>,

    pub items: Option<ItemSelector>,

    pub page_size: Option<i64>,

    pub exclude_from_hash: Vec<MemberPath>,

    pub skip: Vec<OperationName>,

    pub quota_group: Option<String>,

    pub queryable: Vec<MemberPath>,

    pub tags: Option<MemberPath>,

    pub fixed: BTreeMap<MemberPath, FixedValue>,

    pub paginator: Option<Paginator>,

    pub identity: Option<Identity>,

    pub bind: BTreeMap<MemberPath, Binding>,

    pub batch: Option<Batch>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ItemSelector {
    Root,
    Member(MemberPath),
}

impl ItemSelector {
    pub fn parse(value: &str) -> Result<Self, NameError> {
        if value == "$" {
            return Ok(Self::Root);
        }
        MemberPath::new(value).map(Self::Member)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum FixedValue {
    String(String),

    Integer(i64),

    Boolean(bool),

    Strings(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct TokenPair {
    pub input: MemberPath,

    pub output: MemberPath,

    pub optional: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Paginator {
    Token(TokenPair),

    Tokens {
        pairs: Vec<TokenPair>,
        truncated: Option<MemberPath>,
    },

    None,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Identity {
    Arn(MemberPath),

    Template(String),

    Key(Vec<MemberPath>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Binding {
    From {
        operation: OperationName,
        path: MemberPath,
    },

    Account,

    Region,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Batch {
    pub member: MemberPath,

    pub max: i64,

    pub replaces: OperationName,
}
