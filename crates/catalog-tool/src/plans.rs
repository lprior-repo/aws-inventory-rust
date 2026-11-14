//! Model-only collection derivations; nothing here makes an operation callable (VISION 5.1).
//! Suspected pagination requires review: mislabeling it unpaginated silently truncates inventory.
//! Output identifier paths, redundant describes, hash exclusions, endpoint-dependent scope and
//! fixed request parameters belong to overlays or later packages. The historical 2,209 roots
//! preceded List/Describe denials; the current classifier yields 2,197 roots and 2,940 expansions.
//! Two streaming passes retain only bounded vocabularies and counters, never parsed models.

mod counts;
mod expansion;
pub(crate) mod model;
mod pagination;
mod report;
mod tally;
mod types;

pub use expansion::{Expansion, ExpansionReason, expansion};
pub use model::Model;
pub use pagination::{LargestPageSize, Pagination, PaginationReason, Vocabulary, pagination};
use std::path::Path;
use thiserror::Error;

/// Invalid modeled pagination; no partial report is produced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum PaginatorError {
    /// A merged required token field is absent or not a string.
    MissingToken,
    /// Input token or page-size member does not exist.
    InputMember,
    /// The first output-token segment does not exist.
    OutputMember,
    /// A path is empty, has empty segments, or exceeds 32 segments.
    Path,
    /// A range maximum is not a nonnegative integer fitting u32.
    Range,
}

/// Typed input, resource, aggregate and publication failures.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    /// Shared bounded model reader failed.
    #[error(transparent)]
    Model(#[from] crate::smithy::Error),
    /// The parsed model cannot supply the classifier's required fields.
    #[error(transparent)]
    Decode(#[from] serde_json::Error),
    /// Classifier input or deny-list validation failed.
    #[error(transparent)]
    Classes(#[from] crate::classes::Error),
    /// A derivation requires a model shape or field of the expected kind.
    #[error("{service}/{operation}: invalid model: {detail}")]
    InvalidModel {
        service: String,
        operation: String,
        detail: &'static str,
    },
    /// Invalid effective modeled paginator, naming the offending operation.
    #[error("{service}/{operation}: invalid paginator: {kind:?}")]
    InvalidPaginator {
        service: String,
        operation: String,
        kind: PaginatorError,
    },
    /// A bounded input, vocabulary, path, or allocation exceeds its budget.
    #[error("collection plan resource limit exceeded")]
    Limit,
    /// Checked counters overflowed or required sums disagree.
    #[error("collection plan counts do not sum")]
    CountMismatch,
    /// Byte-sensitive report comparison failed.
    #[error("{0}")]
    ReportMismatch(String),
    /// Canonical formatting failed.
    #[error(transparent)]
    Formatting(#[from] std::fmt::Error),
}

/// Derive and render all plans using the root's deny list and `source/models`.
/// At most 4,096 files of 64 MiB each; models are released between reads.
/// # Errors
/// Returns typed model, deny-list, bound, paginator or aggregate failures.
pub fn build_report(root: &Path, source: &Path) -> Result<String, Error> {
    report::build(root, source)
}

/// Print, check exact bytes, or atomically replace `catalog/aws-plans.report`.
/// # Errors
/// Returns derivation, filesystem, stdout or comparison failures.
pub fn run(root: &Path, source: &Path, mode: crate::smithy::ReportMode) -> Result<(), Error> {
    report::run(root, source, mode)
}
