use super::NameError;
use crate::classes::DenyReason;
use std::{collections::BTreeMap, fmt};

/// Validated output/input member path: at most eight segments and 256 bytes.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct MemberPath(String);

impl MemberPath {
    /// Construct a path using ASCII member names separated by dots.
    /// # Errors
    /// Rejects empty/invalid segments, more than eight segments or 256 bytes.
    pub fn new(value: &str) -> Result<Self, NameError> {
        if !super::valid_path(value) {
            return Err(NameError {
                value: value.to_owned(),
            });
        }
        Ok(Self(value.to_owned()))
    }
    /// Borrow the validated spelling.
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

/// Exact ASCII operation name, `[A-Za-z][A-Za-z0-9]*`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct OperationName(String);
impl OperationName {
    /// Validate an operation name; total length is bounded by the overlay byte limit.
    /// # Errors
    /// Rejects empty, non-ASCII or non-alphanumeric names and non-letter initials.
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
    /// Borrow the validated spelling.
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

/// Parsed review metadata and syntactically checked operation overrides.
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
    /// Model directory name, validated at the boundary.
    #[must_use]
    pub fn service(&self) -> &str {
        &self.service
    }
    /// Supported format version, always one.
    #[must_use]
    pub fn format(&self) -> i64 {
        self.format
    }
    /// Real Gregorian date, preserved in YYYY-MM-DD form.
    #[must_use]
    pub fn reviewed(&self) -> &str {
        &self.reviewed
    }
    /// Nonempty reviewer name, at most 64 bytes.
    #[must_use]
    pub fn reviewer(&self) -> &str {
        &self.reviewer
    }
    /// At most 4,096 operations, ordered by exact operation name.
    #[must_use]
    pub fn operations(&self) -> &BTreeMap<OperationName, OperationOverlay> {
        &self.operations
    }
}

/// Explicit classification override; absence leaves model-derived classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum OverlayClass {
    /// Root enumeration candidate.
    Enumerate,
    /// Parent-bound expansion candidate.
    Expand,
    /// Never callable; carries the shared deny taxonomy.
    Deny(DenyReason),
    /// Reviewed but intentionally not callable.
    Defer,
}

/// Scope override.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Scope {
    /// Once per swept region.
    Region,
    /// Once per account.
    Account,
    /// Once in the service's home region.
    HomeRegion,
}

/// Optional plan overrides; absent fields retain the documented model defaults.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct OperationOverlay {
    /// Explicit classification, if reviewed here.
    pub class: Option<OverlayClass>,
    /// Execution scope override.
    pub scope: Option<Scope>,
    /// Output record-list path override.
    pub items: Option<MemberPath>,
    /// Page size, 1..=100,000.
    pub page_size: Option<i64>,
    /// Item fields excluded from hashing, at most 64 unique paths.
    pub exclude_from_hash: Vec<MemberPath>,
    /// Operations made unnecessary, at most 64 unique names.
    pub skip: Vec<OperationName>,
    /// Quota subgroup, at most 32 bytes.
    pub quota_group: Option<String>,
    /// Item paths receiving SQL columns, at most 64 unique paths.
    pub queryable: Vec<MemberPath>,
    /// Item tags path override.
    pub tags: Option<MemberPath>,
    /// Fixed request values keyed by input member path.
    pub fixed: BTreeMap<MemberPath, FixedValue>,
    /// Reviewed paginator override.
    pub paginator: Option<Paginator>,
    /// Reviewed item identity override.
    pub identity: Option<Identity>,
    /// Input bindings, keyed by input member path.
    pub bind: BTreeMap<MemberPath, Binding>,
    /// Batch-read override.
    pub batch: Option<Batch>,
}

/// Allowed fixed request value; string arrays have at most 64 unique entries.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum FixedValue {
    /// String scalar.
    String(String),
    /// TOML signed 64-bit integer.
    Integer(i64),
    /// Boolean scalar.
    Boolean(bool),
    /// String-only array.
    Strings(Vec<String>),
}

/// A token pair with validated member paths.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct TokenPair {
    /// Input token member.
    pub input: MemberPath,
    /// Output token member path.
    pub output: MemberPath,
}

/// Paginator variants admit only their own keys.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Paginator {
    /// Exactly one token pair.
    Token(TokenPair),
    /// One to eight pairs with distinct inputs; optional output truncation flag.
    Tokens {
        pairs: Vec<TokenPair>,
        truncated: Option<MemberPath>,
    },
    /// Reviewed as returning everything in one response.
    None,
}

/// Exactly one reviewed identity strategy.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Identity {
    /// Item ARN member path.
    Arn(MemberPath),
    /// Template whose placeholders are context names or item paths.
    Template(String),
    /// One to eight distinct item paths forming a synthesized key.
    Key(Vec<MemberPath>),
}

/// Exactly one source for a required input.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Binding {
    /// Value from a parent operation's item member path.
    From {
        operation: OperationName,
        path: MemberPath,
    },
    /// Caller account ID.
    Account,
    /// Swept region.
    Region,
}

/// Read many resources in one request.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Batch {
    /// Input list member.
    pub member: MemberPath,
    /// Maximum batch size, 1..=10,000.
    pub max: i64,
    /// Per-resource operation replaced by this operation.
    pub replaces: OperationName,
}
