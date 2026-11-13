//! One reviewed parent-dependent descriptor row.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

use crate::HarnessError;
use crate::catalog::archive::Entry;
use crate::catalog::cloudcontrol::{permissions, properties};
use crate::catalog::json::Json;
use crate::catalog::parent::model::{pointers, sorted_pointers, strings, type_name};
use crate::catalog::rules::{Descriptor, Rule, enrich_rules, rules_json};
use crate::catalog::schema::snake;
use crate::provenance::sha256_hex;

mod fields;

use fields::{
    arn_paths, columns, name_pointers, native_kind, redacted_paths, seeds, sensitive_set,
};

/// Build one parent-only descriptor with reviewed redaction.
pub(super) fn make_descriptor(
    family: &str,
    resource: &str,
    entry: &Entry,
    schema: &Value,
    global: &BTreeSet<String>,
) -> Result<Json, HarnessError> {
    let name = type_name(schema);
    let properties = properties(schema);
    let primary = pointers(schema, "primaryIdentifier")?;
    let write_only = sorted_pointers(schema, "writeOnlyProperties")?;
    let sensitive = sensitive_set(name, &properties, global, schema)?;
    let redacted = redacted_paths(&write_only, &sensitive);
    let columns = columns(name, &properties, schema, &redacted)?;
    let arns = arn_paths(resource, &properties, &primary);
    let kind = native_kind(family, resource, name);
    let descriptor = descriptor_of(name, &kind, &primary, &arns, &write_only, &sensitive);
    let rules = enrich_rules(&descriptor, schema, global)?;
    let mut entries = identity_fields(family, resource, name, &kind);
    entries.extend(schema_fields(resource, &properties, &primary, &arns));
    entries.extend(redaction_fields(&write_only, &sensitive, &rules));
    entries.extend(source_fields(entry, schema, columns));
    Ok(Json::Object(entries))
}

fn descriptor_of(
    name: &str,
    kind: &str,
    primary: &[String],
    arns: &BTreeSet<String>,
    write_only: &[String],
    sensitive: &[String],
) -> Descriptor {
    Descriptor {
        type_name: name.to_string(),
        kind: kind.to_string(),
        primary_identifier: primary.to_vec(),
        arn_pointers: arns.iter().cloned().collect(),
        write_only_properties: write_only.to_vec(),
        sensitive_properties: sensitive.to_vec(),
        relationships: seeds(name),
    }
}

fn identity_fields(family: &str, resource: &str, name: &str, kind: &str) -> Vec<(String, Json)> {
    vec![
        ("type_name".to_owned(), Json::string(name)),
        ("kind".to_owned(), Json::string(kind)),
        (
            "view_name".to_owned(),
            Json::string(format!("aws_cc_{}_{}", snake(family), snake(resource))),
        ),
        ("service".to_owned(), Json::string(snake(family))),
        ("scope".to_owned(), Json::string("regional")),
        ("home_region".to_owned(), Json::Null),
        ("read_supported".to_owned(), Json::Bool(true)),
        ("list_supported".to_owned(), Json::Bool(true)),
        ("list_requires_model".to_owned(), Json::Bool(true)),
        (
            "list_model_validation".to_owned(),
            Json::string("reviewed_parent_model"),
        ),
    ]
}

fn schema_fields(
    resource: &str,
    properties: &BTreeMap<String, Value>,
    primary: &[String],
    arns: &BTreeSet<String>,
) -> Vec<(String, Json)> {
    let identifier_is_arn =
        primary.len() == 1 && primary.first().is_some_and(|value| arns.contains(value));
    vec![
        ("primary_identifier".to_owned(), strings(primary)),
        (
            "identifier_is_arn".to_owned(),
            Json::Bool(identifier_is_arn),
        ),
        ("arn_pointers".to_owned(), strings(&arn_list(arns))),
        ("arn_template".to_owned(), Json::Null),
        ("arn_identifier_pointer".to_owned(), Json::Null),
        (
            "name_pointers".to_owned(),
            strings(&name_pointers(resource, properties)),
        ),
    ]
}

fn arn_list(arns: &BTreeSet<String>) -> Vec<String> {
    arns.iter().cloned().collect()
}

fn redaction_fields(
    write_only: &[String],
    sensitive: &[String],
    rules: &[Rule],
) -> Vec<(String, Json)> {
    vec![
        ("write_only_properties".to_owned(), strings(write_only)),
        ("sensitive_properties".to_owned(), strings(sensitive)),
        ("relationships".to_owned(), rules_json(rules)),
    ]
}

fn source_fields(entry: &Entry, schema: &Value, columns: Vec<Json>) -> Vec<(String, Json)> {
    vec![
        ("schema_file".to_owned(), Json::string(entry.file.as_str())),
        (
            "schema_sha256".to_owned(),
            Json::string(sha256_hex(&entry.raw)),
        ),
        (
            "read_permissions".to_owned(),
            strings(&permissions(schema, "read")),
        ),
        (
            "list_permissions".to_owned(),
            strings(&permissions(schema, "list")),
        ),
        ("columns".to_owned(), Json::array(columns)),
    ]
}
