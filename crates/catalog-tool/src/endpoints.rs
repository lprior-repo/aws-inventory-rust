mod cases;
mod derive;
mod eval;
mod exceptions;
mod functions;
mod model;
mod output;
mod partition;
mod report;
mod report_text;
mod routing;
mod scope;
mod static_context;
mod template;

pub use cases::{Case, CaseFailure, CaseRun, SkipReason, UnsupportedCase, classify_case};
pub use derive::{DerivedEndpoint, OverlayNeed, ProbeOutcome, ServiceClass};
pub use exceptions::{DerivedService, EndpointException, VerificationFailure, VerifiedEndpoint};
pub use model::RuleSet;
pub use output::run;
pub use report::{build_report, verified_endpoints};

use serde::Deserialize;
use serde_json::Value;
use std::path::PathBuf;
use thiserror::Error;

pub const MAX_DEPTH: usize = 64;

pub const MAX_STEPS: usize = 100_000;

pub const MAX_STRING: usize = 16_384;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Endpoint {
    pub url: String,

    #[serde(default = "empty_object")]
    pub properties: Value,

    #[serde(default = "empty_object")]
    pub headers: Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Outcome {
    Endpoint(Endpoint),

    Error(String),

    #[serde(skip)]
    UnsupportedFunction {
        name: String,
    },
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Input(#[from] crate::smithy::Error),
    #[error(transparent)]
    Source(#[from] crate::model_source::Error),

    #[error("{file}: {kind}")]
    Model { file: PathBuf, kind: Fault },

    #[error("{file}: {source}")]
    Io {
        file: PathBuf,
        source: std::io::Error,
    },

    #[error(transparent)]
    Report(#[from] crate::report::ReportError),

    #[error(transparent)]
    Formatting(#[from] std::fmt::Error),
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum Fault {
    #[error("malformed rule set: {0}")]
    Malformed(String),

    #[error("unsupported function: {name}")]
    UnsupportedFunction { name: String },

    #[error("invalid arguments: {0}")]
    Arguments(String),

    #[error("resource bound exceeded: {0}")]
    Bound(&'static str),

    #[error("missing parameter: {0}")]
    MissingParameter(String),

    #[error("no matching endpoint rule")]
    Exhausted,

    #[error("outside commercial partition: {0}")]
    Partition(String),

    #[error("invalid template: {0}")]
    Template(String),
}

fn empty_object() -> Value {
    Value::Object(serde_json::Map::new())
}
