mod counts;
mod deny;
pub(crate) mod model;
pub(crate) mod report;

use std::path::{Path, PathBuf};

pub use crate::smithy::ReportMode;
pub use deny::{DenyList, ParseError, ParseErrorKind};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum DenyReason {
    Content,

    Secret,

    Paid,

    Mutating,

    ReferenceData,
}

impl std::str::FromStr for DenyReason {
    type Err = ParseErrorKind;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "content" => Ok(Self::Content),
            "secret" => Ok(Self::Secret),
            "paid" => Ok(Self::Paid),
            "mutating" => Ok(Self::Mutating),
            "reference-data" => Ok(Self::ReferenceData),
            _ => Err(ParseErrorKind::UnknownReason),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Class {
    Enumerate,

    Expand,

    Denied(DenyReason),

    Unclassified,
}

#[derive(Debug, Clone, Copy)]
pub struct OperationFacts<'a> {
    pub service: &'a str,

    pub name: &'a str,

    pub has_required_input: bool,

    pub readonly: bool,
}

#[must_use]
pub fn classify(facts: &OperationFacts<'_>, deny: &DenyList<'_>) -> Class {
    if !crate::smithy::READ_PREFIXES
        .iter()
        .any(|p| facts.name.starts_with(p))
    {
        return Class::Denied(DenyReason::Mutating);
    }
    if let Some(reason) = deny.reason(facts) {
        return Class::Denied(reason);
    }
    if !(facts.name.starts_with("List") || facts.name.starts_with("Describe")) {
        return Class::Unclassified;
    }
    if facts.has_required_input {
        Class::Expand
    } else {
        Class::Enumerate
    }
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    #[error(transparent)]
    DenyList(#[from] ParseError),

    #[error(transparent)]
    Model(#[from] crate::smithy::Error),

    #[error("{file}: {source}")]
    Io {
        file: PathBuf,
        source: std::io::Error,
    },

    #[error("must-deny line {line} matches no operation: {key}")]
    UnmatchedEntry { line: usize, key: String },

    #[error("invalid model service path: {0}")]
    InvalidModelPath(PathBuf),

    #[error("{service}/{operation}: input {target} does not resolve to a structure")]
    InvalidInput {
        service: String,
        operation: String,
        target: String,
    },

    #[error("{service}/{operation}: mixins are not resolved")]
    MixinsNotResolved { service: String, operation: String },

    #[error("classification resource limit exceeded")]
    Limit,

    #[error("classification counts do not sum to the operation total")]
    CountMismatch,

    #[error("{0}")]
    ReportMismatch(String),

    #[error(transparent)]
    Formatting(#[from] std::fmt::Error),
}

pub fn build_report(root: &Path, source: &Path) -> Result<String, Error> {
    report::build(root, source)
}

pub fn run(root: &Path, source: &Path, mode: ReportMode) -> Result<(), Error> {
    report::run(root, source, mode)
}
