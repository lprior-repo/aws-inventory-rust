//! Bounded, commercial-partition Smithy endpoint compilation and model-case reports.
//! This is offline build tooling, never a runtime endpoint interpreter.
//! Preclassified skipped cases are counted, never run. Node/individual-string limits
//! are not cumulative byte or latency limits: environment copies and property expansion
//! can exceed a gigabyte of string payload in an accepted evaluation.

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
mod scope;
mod template;

pub use crate::smithy::ReportMode;
pub use cases::{Case, CaseCounts, SkipReason, UnsupportedCase, classify_case};
pub use derive::{DerivedEndpoint, ServiceClass};
pub use exceptions::{DerivedService, EndpointException};
pub use model::RuleSet;
pub use output::run;
pub use report::build_report;

use serde::Deserialize;
use serde_json::Value;
use std::path::PathBuf;
use thiserror::Error;

/// Maximum nesting for input documents and evaluation; checked before descending.
pub const MAX_DEPTH: usize = 64;
/// Maximum document nodes or evaluation steps in one invocation.
pub const MAX_STEPS: usize = 100_000;
/// Maximum bytes in any input or expanded endpoint string.
pub const MAX_STRING: usize = 16_384;

/// A complete endpoint, including modeled signing overrides and transport headers.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Endpoint {
    /// Resolved endpoint URL, without normalization.
    pub url: String,
    /// Expanded modeled properties, including `authSchemes` when present.
    #[serde(default = "empty_object")]
    pub properties: Value,
    /// Expanded modeled header arrays.
    #[serde(default = "empty_object")]
    pub headers: Value,
}

/// A modeled error is a successful evaluation, not an interpreter failure.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Outcome {
    /// The first matching endpoint rule.
    Endpoint(Endpoint),
    /// The first matching error rule's expanded message.
    Error(String),
    /// A taken branch needs an overlay/request-time function not implemented here.
    #[serde(skip)]
    UnsupportedFunction { name: String },
}

/// Interpreter and persistence failures retain the offending model filename.
#[derive(Debug, Error)]
pub enum Error {
    /// Shared bounded reader or model discovery failed.
    #[error(transparent)]
    Input(#[from] crate::smithy::Error),
    /// Invalid schema, unsupported function, or malformed expression.
    #[error("{file}: {kind}")]
    Model { file: PathBuf, kind: Fault },
    /// Report I/O failed.
    #[error("{file}: {source}")]
    Io {
        file: PathBuf,
        source: std::io::Error,
    },
    /// Exact report bytes differ.
    #[error("endpoint report differs: {0}")]
    ReportMismatch(PathBuf),
    /// Text rendering failed.
    #[error(transparent)]
    Formatting(#[from] std::fmt::Error),
}

/// Enumerable failures distinct from rule-authored error messages.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum Fault {
    /// The rule schema or value shape is invalid.
    #[error("malformed rule set: {0}")]
    Malformed(String),
    /// A taken expression needs a function outside the compile-time subset.
    #[error("unsupported function: {name}")]
    UnsupportedFunction { name: String },
    /// A function received the wrong count or types.
    #[error("invalid arguments: {0}")]
    Arguments(String),
    /// A named bound was exceeded.
    #[error("resource bound exceeded: {0}")]
    Bound(&'static str),
    /// A required input is absent.
    #[error("missing parameter: {0}")]
    MissingParameter(String),
    /// A selected tree or the rule set exhausted its rules.
    #[error("no matching endpoint rule")]
    Exhausted,
    /// This evaluator intentionally has no other partition data.
    #[error("outside commercial partition: {0}")]
    Partition(String),
    /// A string template is malformed or references an absent/non-string value.
    #[error("invalid template: {0}")]
    Template(String),
}

fn empty_object() -> Value {
    Value::Object(serde_json::Map::new())
}
