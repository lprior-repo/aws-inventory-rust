use std::path::PathBuf;
use thiserror::Error;

/// Invalid standalone member path or operation name.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("invalid name: {value}")]
pub struct NameError {
    /// Original rejected spelling.
    pub value: String,
}

/// Syntactic and cross-field rejection reasons; context lives in `OverlayError`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Rule {
    /// Service is not `[a-z0-9][a-z0-9-]*`.
    Service,
    /// Required key is absent.
    Missing,
    /// Key is not in this table's schema.
    UnknownKey,
    /// Value has an incompatible TOML type.
    WrongType { expected: &'static str },
    /// Format is not one.
    Format,
    /// Date is not a real YYYY-MM-DD Gregorian date.
    Date,
    /// Reviewer is empty or exceeds 64 bytes.
    Reviewer,
    /// Unknown enum spelling or malformed quota subgroup.
    Value,
    /// Invalid member path.
    MemberPath,
    /// Invalid operation name.
    OperationName,
    /// Numeric value is outside the inclusive range.
    Range { min: i64, max: i64 },
    /// Missing or forbidden deny reason.
    DenyReason,
    /// Denied/deferred operations cannot carry plans.
    NoPlan,
    /// Identity needs exactly one strategy.
    Identity,
    /// Template has invalid braces/placeholders or no placeholder.
    Template,
    /// Key does not belong to the selected paginator kind.
    PaginatorKey,
    /// Multi-token paginator has repeated input paths.
    DuplicateInput,
    /// Binding must select exactly one valid source, with true context flags.
    Binding,
    /// Operation refers to itself via bind, skip or batch replacement.
    SelfReference,
    /// Remaining operation belongs to or depends on an in-file bind cycle.
    BindCycle,
    /// Collection length is outside an inclusive range.
    Count { min: usize, max: usize },
    /// Array repeats a value.
    Duplicate,
    /// Collection count or byte length exceeds its bound.
    Limit { max: usize },
    /// Fallible allocation reservation failed.
    Resource,
}

/// Overlay rejection with service, optional operation, key and offending value.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum OverlayError {
    /// Rule violation. Missing values are empty; containers/datetimes use type labels.
    #[error("overlay {service}/{operation:?} {key}: {rule:?}: {value}")]
    Rule {
        service: String,
        operation: Option<String>,
        key: String,
        value: Box<str>,
        rule: Rule,
    },
    /// Original TOML syntax error, retaining its source text, span, line and column.
    #[error("overlay {service}: {source}")]
    Syntax {
        service: String,
        #[source]
        source: toml::de::Error,
    },
    /// Filesystem failure, including non-UTF-8 file contents.
    #[error("overlay file {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

#[derive(Clone)]
pub(super) struct Context<'a> {
    pub service: &'a str,
    pub operation: Option<&'a str>,
    pub prefix: String,
}
impl Context<'_> {
    pub fn error(&self, key: &str, value: impl ToString, rule: Rule) -> OverlayError {
        OverlayError::Rule {
            service: self.service.to_owned(),
            operation: self.operation.map(str::to_owned),
            key: self.key(key),
            value: value.to_string().into_boxed_str(),
            rule,
        }
    }
    pub fn key(&self, key: &str) -> String {
        if self.prefix.is_empty() {
            return key.to_owned();
        }
        if key.is_empty() {
            return self.prefix.clone();
        }
        format!("{}.{key}", self.prefix)
    }
    pub fn child(&self, key: &str) -> Self {
        Self {
            prefix: self.key(key),
            ..self.clone()
        }
    }
}
