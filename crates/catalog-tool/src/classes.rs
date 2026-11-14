//! Fail-closed operation classification and deterministic, build-time model reports.
//! Denials win; only case-sensitive `List`/`Describe` prefixes may be allowed.
//! `readonly` is diagnostic, never permission. Expand is a candidate classification:
//! parent bindings and reviewed service overlays remain later callable-handle gates.
//! The ROADMAP's 2,209 roots and 2,998 expansions precede List/Describe denials.
//! The pinned models with the reviewed deny policy yield 2,197 enumerate and
//! 2,940 expand candidates; neither class alone grants callability.

mod counts;
mod deny;
pub(crate) mod model;
pub(crate) mod report;

use std::path::{Path, PathBuf};

pub use crate::smithy::ReportMode;
pub use deny::{DenyList, ParseError, ParseErrorKind};
use thiserror::Error;

/// A reason an operation must never be called.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum DenyReason {
    /// Customer content rather than resource configuration.
    Content,
    /// Credentials, tokens, passwords, or secret values.
    Secret,
    /// A billed data query.
    Paid,
    /// Changes state or lacks a recognized read prefix.
    Mutating,
    /// AWS-owned reference data rather than account resources.
    ReferenceData,
}

/// Classification is not itself permission to call an operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Class {
    /// List/Describe candidate without required input members.
    Enumerate,
    /// List/Describe candidate requiring parent input bindings.
    Expand,
    /// Forbidden, with the winning reason.
    Denied(DenyReason),
    /// No reviewed allow rule; never callable.
    Unclassified,
}

/// Borrowed facts; flags describe the model, not authorization decisions.
#[derive(Debug, Clone, Copy)]
pub struct OperationFacts<'a> {
    /// Model directory name, not the Smithy service shape name.
    pub service: &'a str,
    /// Case-sensitive operation name without its namespace.
    pub name: &'a str,
    /// Whether the input structure contains any required member.
    pub has_required_input: bool,
    /// Presence of the Smithy readonly trait (diagnostic only).
    pub readonly: bool,
}

/// Classify without allocation or I/O. Non-read names are always mutating;
/// scoped denials override global denials, and neither can grant permission.
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

/// Typed failures; incomplete input never produces or overwrites a report.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    /// Invalid must-deny syntax, with a one-based line number.
    #[error(transparent)]
    DenyList(#[from] ParseError),
    /// Bounded model discovery or decoding failed.
    #[error(transparent)]
    Model(#[from] crate::smithy::Error),
    /// A filesystem or stdout write failed.
    #[error("{file}: {source}")]
    Io {
        file: PathBuf,
        source: std::io::Error,
    },
    /// A deny entry matched no operation in any loaded model.
    #[error("must-deny line {line} matches no operation: {key}")]
    UnmatchedEntry { line: usize, key: String },
    /// A model is not inside a service directory.
    #[error("invalid model service path: {0}")]
    InvalidModelPath(PathBuf),
    /// A present, non-Unit input target is missing or is not a structure.
    #[error("{service}/{operation}: input {target} does not resolve to a structure")]
    InvalidInput {
        service: String,
        operation: String,
        target: String,
    },
    /// Required members inherited through input mixins cannot yet be resolved.
    #[error("{service}/{operation}: mixins are not resolved")]
    MixinsNotResolved { service: String, operation: String },
    /// A bounded input or allocation limit was exceeded.
    #[error("classification resource limit exceeded")]
    Limit,
    /// Arithmetic overflow or inconsistent aggregate counts.
    #[error("classification counts do not sum to the operation total")]
    CountMismatch,
    /// Every byte-sensitive report difference.
    #[error("{0}")]
    ReportMismatch(String),
    /// Canonical text formatting failed.
    #[error(transparent)]
    Formatting(#[from] std::fmt::Error),
}

/// Build a deterministic report from `source/models` and the root's deny list.
/// Models are read one at a time (64 MiB each), at most 4,096 files.
/// # Errors
/// Rejects invalid input, unmatched deny entries, resource limits or bad totals.
pub fn build_report(root: &Path, source: &Path) -> Result<String, Error> {
    report::build(root, source)
}

/// Print the complete report, then optionally check or overwrite its committed file.
/// # Errors
/// Returns build, stdout, filesystem or byte-sensitive comparison errors.
pub fn run(root: &Path, source: &Path, mode: ReportMode) -> Result<(), Error> {
    report::run(root, source, mode)
}
