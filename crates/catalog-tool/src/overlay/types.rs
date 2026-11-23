use super::FixedValue;
use super::NameError;
use super::ResourceEmission;
use super::{Batch, Binding, Origin};
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

    pub fn selected(value: &str) -> Result<Self, NameError> {
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

impl std::borrow::Borrow<str> for OperationName {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ServiceOverlay {
    pub(super) service: String,
    pub(super) format: i64,
    pub(super) reviewed: String,
    pub(super) reviewer: String,
    pub(super) evidence: ReviewEvidence,
    pub(super) operations: BTreeMap<OperationName, OperationOverlay>,
    pub(super) origin: BTreeMap<OperationName, Origin>,
}

/// Evidence kinds are separate: a machine review never implies a human, fixture or emulator check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReviewEvidence {
    pub machine_reviewed: bool,
    pub manually_reviewed: bool,
    pub fixture_tested: bool,
    pub emulator_tested: bool,
}

impl ServiceOverlay {
    #[must_use]
    pub fn origin_of(&self, operation: &OperationName) -> Origin {
        match self.origin.get(operation) {
            Some(origin) => *origin,
            None => Origin::Hand,
        }
    }

    #[must_use]
    pub fn evidence(&self) -> &ReviewEvidence {
        &self.evidence
    }

    pub fn new(
        service: &str,
        operations: Vec<(&str, OperationOverlay)>,
    ) -> Result<Self, NameError> {
        super::builtin::service(service, operations)
    }

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

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[non_exhaustive]
pub struct OperationOverlay {
    pub row_policy: super::RowPolicy,
    pub tag_endpoint: Option<super::TagEndpoint>,
    pub emission: super::Emission,
    pub projection: Option<super::ResourceProjection>,
    pub classification: Option<super::KindClassification>,
    pub routing: super::RequestRouting,
    pub branches: Vec<super::ExecutionBranch>,
    pub endpoint_discovery: bool,
    pub class: Option<OverlayClass>,
    pub defer_reason: Option<String>,

    pub scope: Option<Scope>,

    pub items: Option<ItemSelector>,

    pub page_size: Option<i64>,

    pub exclude_from_hash: Vec<MemberPath>,
    pub exclude_map_keys: Vec<super::MapKeyExclusion>,

    pub skip: Vec<OperationName>,

    pub quota_group: Option<String>,

    pub queryable: Vec<MemberPath>,

    pub tags: Option<MemberPath>,

    pub fixed: BTreeMap<MemberPath, FixedValue>,

    pub paginator: Option<Paginator>,

    pub identity: Option<Identity>,

    pub bind: BTreeMap<MemberPath, Binding>,

    pub batch: Option<Batch>,
    pub additional_resources: Vec<ResourceEmission>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ItemSelector {
    Root,
    Member(MemberPath),
    Explicit {
        path: MemberPath,
        kind: super::SelectionKind,
    },
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
pub struct TokenPair {
    pub input: MemberPath,

    pub output: MemberPath,

    /// Only a typed boolean is accepted; textual and numeric flags cannot construct policy.
    /// ```compile_fail
    /// # use catalog_tool::overlay::{MemberPath, TokenPair};
    /// # fn policy() -> Result<(), catalog_tool::overlay::NameError> {
    /// let mut pair = TokenPair::new(MemberPath::new("Start")?, MemberPath::new("Next")?);
    /// pair.optional = "true";
    /// # Ok(()) }
    /// ```
    /// ```compile_fail
    /// # use catalog_tool::overlay::{MemberPath, TokenPair};
    /// # fn policy() -> Result<(), catalog_tool::overlay::NameError> {
    /// let mut pair = TokenPair::new(MemberPath::new("Start")?, MemberPath::new("Next")?);
    /// pair.optional = 1;
    /// # Ok(()) }
    /// ```
    pub optional: bool,
}

impl TokenPair {
    #[must_use]
    pub const fn new(input: MemberPath, output: MemberPath) -> Self {
        Self {
            input,
            output,
            optional: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Paginator {
    Token(TokenPair),

    Tokens {
        pairs: Vec<TokenPair>,
        truncated: Option<MemberPath>,
    },

    LastListItem {
        input: MemberPath,
        output: MemberPath,
        truncated: MemberPath,
    },

    None,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Identity {
    Arn(MemberPath),

    Template(String),

    Key(Vec<MemberPath>),
    Choice(super::IdentityChoice),
    CorrelatedKey(Vec<super::IdentityField>),
    MapArn {
        path: MemberPath,
        key: String,
        resource_kind: super::ArnResourceKind,
        request_url: Option<MemberPath>,
    },
}
