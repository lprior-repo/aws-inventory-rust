use std::collections::BTreeSet;

use serde_json::Value;

use crate::HarnessError;
use crate::catalog::archive::{Entry, handlers};
use crate::catalog::cloudcontrol::{handler_schema, permissions, strings, truthy};
use crate::catalog::json::Json;
use crate::catalog::rules::{
    ARN_TEMPLATES, Descriptor, MODEL_REQUIRED, NATIVE, Rule, enrich_rules, reviewed_rules,
    rules_json,
};
use crate::catalog::schema::{empty_model_valid, snake};
use crate::provenance::sha256_hex;

mod fields;

use fields::{Paths, arn_paths, name_pointers, paths};

pub const MAX_COLUMNS: usize = 200;

struct Parts {
    list_validation: &'static str,
    kind: String,
    paths: Paths,
    arns: BTreeSet<String>,
    name_pointers: Vec<String>,
    rules: Vec<Rule>,
}

pub(super) fn descriptor(
    service: &str,
    resource: &str,
    name: &str,
    entry: &Entry,
    schema: &Value,
    is_new: bool,
    global: &BTreeSet<String>,
) -> Result<Json, HarnessError> {
    eligibility(name, schema)?;
    let parts = describe(service, resource, name, schema, is_new, global)?;
    Ok(document_row(service, entry, schema, resource, name, &parts))
}

fn eligibility(name: &str, schema: &Value) -> Result<(), HarnessError> {
    let declared = handlers(schema).unwrap_or_default();
    if !(declared.iter().any(|key| key == "read") && declared.iter().any(|key| key == "list"))
        || MODEL_REQUIRED.contains(&name)
    {
        return Err(HarnessError::invalid(format!("not eligible: {name}")));
    }
    Ok(())
}

fn list_validation(name: &str, schema: &Value) -> Result<&'static str, HarnessError> {
    let empty = Value::Object(serde_json::Map::new());
    let handler = handler_schema(schema, "list").unwrap_or(&empty);
    if !empty_model_valid(handler)? {
        return Err(HarnessError::invalid(format!(
            "list model required: {name}"
        )));
    }
    Ok(if truthy(handler) {
        "empty_object_accepted"
    } else {
        "no_handler_schema"
    })
}

fn describe(
    service: &str,
    resource: &str,
    name: &str,
    schema: &Value,
    is_new: bool,
    global: &BTreeSet<String>,
) -> Result<Parts, HarnessError> {
    let list_validation = list_validation(name, schema)?;
    let paths = paths(name, schema, global, is_new)?;
    let arns = arn_paths(name, resource, &paths, is_new, schema)?;
    let kind = native_kind(service, resource, name);
    let descriptor = descriptor_of(name, &kind, &arns, &paths);
    Ok(Parts {
        name_pointers: name_pointers(resource, &paths.properties, &paths.primary),
        rules: enrich_rules(&descriptor, schema, global)?,
        list_validation,
        kind,
        paths,
        arns,
    })
}

fn descriptor_of(name: &str, kind: &str, arns: &BTreeSet<String>, paths: &Paths) -> Descriptor {
    Descriptor {
        type_name: name.to_string(),
        kind: kind.to_string(),
        primary_identifier: paths.primary.clone(),
        arn_pointers: arns.iter().cloned().collect(),
        write_only_properties: paths.write_only.clone(),
        sensitive_properties: paths.sensitive.clone(),
        relationships: reviewed_rules(name),
    }
}

fn native_kind(service: &str, resource: &str, name: &str) -> String {
    NATIVE
        .iter()
        .find(|(key, _)| *key == name)
        .map(|(_, value)| (*value).to_string())
        .unwrap_or_else(|| format!("aws_{}_{}", snake(service), snake(resource)))
}

fn document_row(
    service: &str,
    entry: &Entry,
    schema: &Value,
    resource: &str,
    name: &str,
    parts: &Parts,
) -> Json {
    let mut entries = identity_fields(service, resource, name, parts);
    entries.extend(schema_fields(name, parts));
    entries.extend(redaction_fields(parts));
    entries.extend(source_fields(entry, schema, &parts.paths.columns));
    Json::Object(entries)
}

fn identity_fields(
    service: &str,
    resource: &str,
    name: &str,
    parts: &Parts,
) -> Vec<(String, Json)> {
    vec![
        ("type_name".to_owned(), Json::string(name)),
        ("kind".to_owned(), Json::string(parts.kind.as_str())),
        (
            "view_name".to_owned(),
            Json::string(format!("aws_cc_{}_{}", snake(service), snake(resource))),
        ),
        ("service".to_owned(), Json::string(snake(service))),
        ("scope".to_owned(), Json::string("regional")),
        ("home_region".to_owned(), Json::Null),
        ("read_supported".to_owned(), Json::Bool(true)),
        ("list_supported".to_owned(), Json::Bool(true)),
        ("list_requires_model".to_owned(), Json::Bool(false)),
        (
            "list_model_validation".to_owned(),
            Json::string(parts.list_validation),
        ),
    ]
}

fn schema_fields(name: &str, parts: &Parts) -> Vec<(String, Json)> {
    let primary = &parts.paths.primary;
    let arns = &parts.arns;
    vec![
        ("primary_identifier".to_owned(), strings(primary)),
        (
            "identifier_is_arn".to_owned(),
            Json::Bool(identifier_is_arn(primary, arns)),
        ),
        ("arn_pointers".to_owned(), strings(&arn_list(arns))),
        ("arn_template".to_owned(), arn_template(name)),
        (
            "arn_identifier_pointer".to_owned(),
            arn_identifier_pointer(name, primary),
        ),
        ("name_pointers".to_owned(), strings(&parts.name_pointers)),
    ]
}

fn arn_list(arns: &BTreeSet<String>) -> Vec<String> {
    arns.iter().cloned().collect()
}

fn identifier_is_arn(primary: &[String], arns: &BTreeSet<String>) -> bool {
    primary.len() == 1 && primary.first().is_some_and(|value| arns.contains(value))
}

fn arn_template(name: &str) -> Json {
    ARN_TEMPLATES
        .iter()
        .find(|(key, _)| *key == name)
        .map(|(_, value)| Json::string(*value))
        .unwrap_or(Json::Null)
}

fn arn_identifier_pointer(name: &str, primary: &[String]) -> Json {
    if name == "AWS::EC2::EIP" {
        Json::string("/AllocationId")
    } else if primary.len() == 1 && ARN_TEMPLATES.iter().any(|(key, _)| *key == name) {
        primary
            .first()
            .map_or(Json::Null, |value| Json::string(value.as_str()))
    } else {
        Json::Null
    }
}

fn redaction_fields(parts: &Parts) -> Vec<(String, Json)> {
    vec![
        (
            "write_only_properties".to_owned(),
            strings(&parts.paths.write_only),
        ),
        (
            "sensitive_properties".to_owned(),
            strings(&parts.paths.sensitive),
        ),
        ("relationships".to_owned(), rules_json(&parts.rules)),
    ]
}

fn source_fields(entry: &Entry, schema: &Value, columns: &Json) -> Vec<(String, Json)> {
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
        ("columns".to_owned(), columns.clone()),
    ]
}
