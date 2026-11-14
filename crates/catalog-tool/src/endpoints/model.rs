use super::{Error, Fault, MAX_DEPTH, MAX_STEPS, MAX_STRING, functions::Function};
use serde::Deserialize;
use serde_json::Value;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

#[derive(Debug, Deserialize)]
pub(super) struct Document {
    pub version: String,
    pub parameters: BTreeMap<String, Parameter>,
    pub rules: Vec<Rule>,
}

#[derive(Debug, Deserialize)]
pub(super) struct Parameter {
    #[serde(rename = "type")]
    pub kind: ParameterType,
    #[serde(default)]
    pub required: bool,
    pub default: Option<Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) enum ParameterType {
    // The pinned models spell these both ways (13 `String`, 4 `Boolean`).
    #[serde(alias = "String")]
    String,
    #[serde(alias = "Boolean")]
    Boolean,
    StringArray,
}

impl ParameterType {
    pub fn accepts(&self, value: &Value) -> bool {
        match self {
            Self::String => value.is_string(),
            Self::Boolean => value.is_boolean(),
            Self::StringArray => value
                .as_array()
                .is_some_and(|a| a.iter().all(Value::is_string)),
        }
    }
}

#[derive(Debug, Deserialize)]
pub(super) struct Rule {
    pub conditions: Vec<Condition>,
    #[serde(flatten)]
    pub kind: RuleKind,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub(super) enum RuleKind {
    Tree { rules: Vec<Rule> },
    Endpoint { endpoint: EndpointExpr },
    Error { error: Expr },
}

#[derive(Debug, Deserialize)]
pub(super) struct EndpointExpr {
    pub url: Expr,
    #[serde(default)]
    pub properties: serde_json::Map<String, Value>,
    #[serde(default)]
    pub headers: BTreeMap<String, Vec<Expr>>,
}

#[derive(Debug, Deserialize)]
pub(super) struct Condition {
    #[serde(flatten)]
    pub call: Call,
    pub assign: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(super) struct Call {
    #[serde(rename = "fn")]
    pub function: String,
    pub argv: Vec<Expr>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(super) enum Expr {
    Ref { r#ref: String },
    Call(Call),
    Literal(Value),
}

/// Parsed rules with a filename retained for every subsequent failure.
#[derive(Debug)]
pub struct RuleSet {
    pub(super) file: PathBuf,
    pub(super) document: Document,
}

impl RuleSet {
    /// Parse an endpoint trait, checking depth, node, string and binding bounds.
    ///
    /// # Errors
    /// Returns filename-bearing schema or resource-bound errors.
    pub fn parse(file: &Path, value: &Value) -> Result<Self, Error> {
        let result = Self::parse_document(value);
        result
            .map(|document| Self {
                file: file.to_path_buf(),
                document,
            })
            .map_err(|kind| Error::Model {
                file: file.to_path_buf(),
                kind,
            })
    }

    fn parse_document(value: &Value) -> Result<Document, Fault> {
        let mut steps = MAX_STEPS;
        guard(value, 0, &mut steps)?;
        let document: Document =
            serde_json::from_value(value.clone()).map_err(|e| Fault::Malformed(e.to_string()))?;
        if document.version != "1.0" || document.rules.is_empty() {
            return Err(Fault::Malformed(
                "version 1.0 and nonempty rules required".into(),
            ));
        }
        validate_parameters(&document)?;
        super::scope::validate(&document)?;
        Ok(document)
    }

    pub(super) fn error(&self, kind: Fault) -> Error {
        Error::Model {
            file: self.file.clone(),
            kind,
        }
    }
}

pub(super) fn guard(value: &Value, depth: usize, steps: &mut usize) -> Result<(), Fault> {
    tick(depth, steps)?;
    match value {
        Value::String(s) if s.len() > MAX_STRING => Err(Fault::Bound("string bytes")),
        Value::Array(a) => a
            .iter()
            .try_for_each(|v| guard(v, depth.saturating_add(1), steps)),
        Value::Object(o) => {
            if o.keys().any(|key| key.len() > MAX_STRING) {
                return Err(Fault::Bound("string bytes"));
            }
            validate_call(o)?;
            o.values()
                .try_for_each(|v| guard(v, depth.saturating_add(1), steps))
        }
        _ => Ok(()),
    }
}

fn validate_call(object: &serde_json::Map<String, Value>) -> Result<(), Fault> {
    let Some(name) = object.get("fn") else {
        return Ok(());
    };
    let name = name
        .as_str()
        .ok_or_else(|| Fault::Malformed("fn must be a string".into()))?;
    let args = object
        .get("argv")
        .and_then(Value::as_array)
        .ok_or_else(|| Fault::Arguments(name.into()))?;
    if Function::parse(name).is_some_and(|function| args.len() != function.arity()) {
        return Err(Fault::Arguments(name.into()));
    }
    Ok(())
}

pub(super) fn tick(depth: usize, steps: &mut usize) -> Result<(), Fault> {
    if depth > MAX_DEPTH {
        return Err(Fault::Bound("depth"));
    }
    *steps = steps.checked_sub(1).ok_or(Fault::Bound("steps"))?;
    Ok(())
}

fn validate_parameters(document: &Document) -> Result<(), Fault> {
    if document.parameters.len() > 256 {
        return Err(Fault::Bound("parameters"));
    }
    document
        .parameters
        .iter()
        .try_for_each(|(name, parameter)| {
            let invalid = parameter
                .default
                .as_ref()
                .is_some_and(|value| !parameter.required || !parameter.kind.accepts(value));
            if invalid {
                return Err(Fault::Malformed(format!("invalid default for {name}")));
            }
            Ok(())
        })
}
