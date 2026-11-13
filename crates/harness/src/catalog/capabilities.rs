//! The full capability matrix for every retained provider schema.

use std::collections::BTreeMap;

use serde_json::Value;

use crate::HarnessError;
use crate::catalog::archive::{Archive, Entry, handlers};
use crate::catalog::json::Json;
use crate::catalog::rules::MODEL_REQUIRED;
use crate::catalog::schema::{empty_model_valid, snake};
use crate::provenance::sha256_hex;

mod evidence;
mod summary;

use evidence::document;
use summary::summary;

pub use evidence::{not_run, without_executed_evidence};

struct Facts {
    name: String,
    family: String,
    read: bool,
    listing: bool,
    exclusion: Option<&'static str>,
    validation: &'static str,
    schema_file: String,
    schema_sha256: String,
}

/// Build the deterministic capability matrix for the pinned schemas.
pub fn build(
    archive: &Archive,
    resources: &Json,
    resources_sha256: &str,
) -> Result<Json, HarnessError> {
    let selected = selected_map(resources);
    let mut rows = Vec::new();
    let mut exclusions: BTreeMap<String, usize> = BTreeMap::new();
    for (name, entry) in &archive.entries {
        let facts = classify(name, entry, selected.contains_key(name.as_str()))?;
        if let Some(reason) = facts.exclusion {
            let slot = exclusions.entry(reason.to_string()).or_insert(0);
            *slot = slot.saturating_add(1);
        }
        rows.push(row(&facts, selected.get(name.as_str()).copied()));
    }
    let summary = summary(resources, archive, &rows, &exclusions);
    Ok(document(rows, resources_sha256, summary))
}

fn selected_map(resources: &Json) -> BTreeMap<&str, &Json> {
    resources
        .get("resource_types")
        .and_then(Json::as_array)
        .unwrap_or_default()
        .iter()
        .filter_map(|row| {
            row.get("type_name")
                .and_then(Json::as_str)
                .map(|name| (name, row))
        })
        .collect()
}

fn classify(name: &str, entry: &Entry, selected: bool) -> Result<Facts, HarnessError> {
    let declared = handlers(&entry.schema).unwrap_or_default();
    let read = declared.iter().any(|key| key == "read");
    let listing = declared.iter().any(|key| key == "list");
    let empty = Value::Object(serde_json::Map::new());
    let handler = handler_schema(&entry.schema).unwrap_or(&empty);
    let empty_model = listing && empty_model_valid(handler)?;
    let present = truthy(handler);
    let exclusion = exclusion_of(name, read, listing, empty_model, selected);
    Ok(Facts {
        name: name.to_string(),
        family: name.split("::").nth(1).unwrap_or("").to_string(),
        read,
        listing,
        exclusion,
        validation: validation_of(listing, empty_model, present),
        schema_file: entry.file.clone(),
        schema_sha256: sha256_hex(&entry.raw),
    })
}

fn exclusion_of(
    name: &str,
    read: bool,
    listing: bool,
    empty: bool,
    selected: bool,
) -> Option<&'static str> {
    if !(read && listing) {
        Some("missing_read_or_list_handler")
    } else if MODEL_REQUIRED.contains(&name) {
        Some("documented_parent_model_required")
    } else if !empty {
        Some("schema_parent_model_required")
    } else if !selected {
        Some("scope_unreviewed")
    } else {
        None
    }
}

fn validation_of(listing: bool, empty: bool, handler: bool) -> &'static str {
    if !listing {
        "no_list_handler"
    } else if !empty {
        "empty_object_rejected"
    } else if handler {
        "empty_object_accepted"
    } else {
        "no_handler_schema"
    }
}

fn row(facts: &Facts, descriptor: Option<&Json>) -> Json {
    let mut entries = identity_fields(facts, descriptor);
    entries.extend(review_fields(facts, descriptor));
    entries.extend(support_fields(facts));
    entries.extend(source_fields(facts, descriptor));
    entries.push(("emulator_verification".to_owned(), not_run()));
    Json::Object(entries)
}

fn identity_fields(facts: &Facts, descriptor: Option<&Json>) -> Vec<(String, Json)> {
    vec![
        ("type_name".to_owned(), Json::string(facts.name.as_str())),
        (
            "service".to_owned(),
            Json::string(snake(facts.family.as_str())),
        ),
        (
            "service_name".to_owned(),
            Json::string(facts.family.as_str()),
        ),
        ("selected".to_owned(), Json::Bool(descriptor.is_some())),
    ]
}

fn review_fields(facts: &Facts, descriptor: Option<&Json>) -> Vec<(String, Json)> {
    vec![
        (
            "exclusion".to_owned(),
            match facts.exclusion {
                Some(reason) => Json::string(reason),
                None => Json::Null,
            },
        ),
        (
            "scope_review".to_owned(),
            Json::string(if descriptor.is_some() {
                "reviewed_regional"
            } else {
                "unreviewed"
            }),
        ),
    ]
}

fn support_fields(facts: &Facts) -> Vec<(String, Json)> {
    vec![
        ("read_supported".to_owned(), Json::Bool(facts.read)),
        ("list_supported".to_owned(), Json::Bool(facts.listing)),
        (
            "list_model_validation".to_owned(),
            Json::string(facts.validation),
        ),
    ]
}

fn source_fields(facts: &Facts, descriptor: Option<&Json>) -> Vec<(String, Json)> {
    vec![
        (
            "schema_file".to_owned(),
            Json::string(facts.schema_file.as_str()),
        ),
        (
            "schema_sha256".to_owned(),
            Json::string(facts.schema_sha256.as_str()),
        ),
        (
            "relationship_rule_count".to_owned(),
            Json::count(relationship_count(descriptor)),
        ),
        (
            "view_name".to_owned(),
            descriptor
                .and_then(|row| row.get("view_name"))
                .and_then(Json::as_str)
                .map(Json::string)
                .unwrap_or(Json::Null),
        ),
    ]
}

/// The archive base name embedded in evidence documents.
pub fn archive_name() -> &'static str {
    crate::catalog::ARCHIVE
        .rsplit('/')
        .next()
        .unwrap_or(crate::catalog::ARCHIVE)
}

fn handler_schema(schema: &Value) -> Option<&Value> {
    schema.get("handlers")?.get("list")?.get("handlerSchema")
}

fn relationship_count(descriptor: Option<&Json>) -> usize {
    descriptor
        .and_then(|row| row.get("relationships"))
        .and_then(Json::as_array)
        .map_or(0, <[Json]>::len)
}

fn truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(flag) => *flag,
        Value::Number(number) => number.as_f64().is_some_and(|value| value != 0.0),
        Value::String(text) => !text.is_empty(),
        Value::Array(items) => !items.is_empty(),
        Value::Object(object) => !object.is_empty(),
    }
}
