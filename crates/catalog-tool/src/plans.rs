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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum PaginatorError {
    MissingToken,

    InputMember,

    OutputMember,

    Path,

    Range,
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    #[error(transparent)]
    Model(#[from] crate::smithy::Error),

    #[error(transparent)]
    Decode(#[from] serde_json::Error),

    #[error(transparent)]
    Classes(#[from] crate::classes::Error),

    #[error("{service}/{operation}: invalid model: {detail}")]
    InvalidModel {
        service: String,
        operation: String,
        detail: &'static str,
    },

    #[error("{service}/{operation}: invalid paginator: {kind:?}")]
    InvalidPaginator {
        service: String,
        operation: String,
        kind: PaginatorError,
    },

    #[error("collection plan resource limit exceeded")]
    Limit,

    #[error("collection plan counts do not sum")]
    CountMismatch,

    #[error("{0}")]
    ReportMismatch(String),

    #[error(transparent)]
    Formatting(#[from] std::fmt::Error),
}

pub fn build_report(root: &Path, source: &Path) -> Result<String, Error> {
    report::build(root, source)
}

pub fn run(root: &Path, source: &Path, mode: crate::smithy::ReportMode) -> Result<(), Error> {
    report::run(root, source, mode)
}
