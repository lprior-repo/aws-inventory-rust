//! Bounded, schema-backed relationship compilation.
//!
//! A rule is only emitted from a declared string property. Opaque maps are not
//! searched, identifier prefixes are bound to the reviewed EC2 grammar, and a
//! type-specific path override always outranks lexical resemblance.

mod compile;
mod enrich_tables;
mod model;
mod native;
mod relationships;
mod selection;
mod sensitive;

use std::collections::BTreeSet;

use serde_json::Value;

use crate::HarnessError;
use crate::catalog::schema::{escape, normalize, schema_nodes};

pub(crate) use compile::{enrich_rules, rules_json};
pub(crate) use enrich_tables::{
    ADDITIONAL_SENSITIVE, ARN_PATH_OVERRIDES, EC2_IDENTIFIERS, IDENTIFIER_PATH_EXCLUSIONS,
    SENSITIVE_NODE_NAMES,
};
pub(crate) use model::{Descriptor, Rule};
pub(crate) use native::{ARN_TEMPLATES, NATIVE, SELF_ARNS};
pub(crate) use relationships::RELATIONSHIPS;
pub(crate) use selection::{GLOBAL_SENSITIVE, MODEL_REQUIRED, RESERVED, SELECTED};
pub(crate) use sensitive::SENSITIVE;

/// Reviewed sensitive paths for one type.
pub(crate) fn sensitive_paths(name: &str) -> &'static [&'static str] {
    SENSITIVE
        .iter()
        .find(|(key, _)| *key == name)
        .map(|(_, paths)| *paths)
        .unwrap_or(&[])
}

/// Reviewed seed rules for one type.
pub(crate) fn reviewed_rules(name: &str) -> Vec<Rule> {
    RELATIONSHIPS
        .iter()
        .find(|(key, _)| *key == name)
        .map(|(_, rules)| {
            rules
                .iter()
                .map(|(path, relation, kind, arn)| Rule::reviewed(path, relation, kind, *arn))
                .collect()
        })
        .unwrap_or_default()
}

/// Additional conservative sensitivity for a newly admitted type.
pub fn sensitive_paths_for(name: &str, schema: &Value) -> Result<Vec<String>, HarnessError> {
    let mut paths: BTreeSet<String> = ADDITIONAL_SENSITIVE
        .iter()
        .find(|(key, _)| *key == name)
        .map(|(_, values)| values.iter().map(|value| (*value).to_string()).collect())
        .unwrap_or_default();
    for path in schema_nodes(schema)?.keys() {
        let field = path.rsplit('/').next().unwrap_or("");
        if SENSITIVE_NODE_NAMES.contains(&normalize(field).as_str()) {
            paths.insert(path.clone());
        }
    }
    Ok(paths.into_iter().collect())
}

/// Escape helper re-exported for descriptor builders.
pub(crate) fn escaped_pointer(key: &str) -> String {
    format!("/{}", escape(key))
}
