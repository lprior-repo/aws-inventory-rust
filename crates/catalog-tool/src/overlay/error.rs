use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("invalid name: {value}")]
pub struct NameError {
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Rule {
    Service,

    Missing,

    UnknownKey,

    WrongType { expected: &'static str },

    Format,

    Date,

    Reviewer,

    Value,

    MemberPath,

    OperationName,

    Range { min: i64, max: i64 },

    DenyReason,

    NoPlan,

    Identity,

    Template,

    PaginatorKey,

    DuplicateInput,

    Binding,

    SelfReference,

    BindCycle,

    Count { min: usize, max: usize },

    Duplicate,

    Limit { max: usize },

    Resource,
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum OverlayError {
    #[error("overlay {service}/{operation:?} {key}: {rule:?}: {value}")]
    Rule {
        service: String,
        operation: Option<String>,
        key: String,
        value: Box<str>,
        rule: Rule,
    },

    #[error("overlay {service}: {source}")]
    Syntax {
        service: String,
        #[source]
        source: toml::de::Error,
    },

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
