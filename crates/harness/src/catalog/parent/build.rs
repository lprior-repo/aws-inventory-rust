//! Validate the reviewed parent groups and collect their descriptor rows.

use std::collections::BTreeSet;
use std::path::Path;

use serde_json::Value;

use crate::HarnessError;
use crate::catalog::archive::{Archive, handlers};
use crate::catalog::cloudcontrol::properties;
use crate::catalog::json::Json;
use crate::catalog::rules::GLOBAL_SENSITIVE;
use crate::catalog::schema::{basic_type, empty_model_valid};
use crate::catalog::tables::GROUPS;
use crate::catalog::{ARCHIVE, ARCHIVE_SHA256, ROOT_CATALOG};
use crate::provenance::{file_sha256, sha256_hex};

use super::descriptor::make_descriptor;
use super::model::{required_list, schema_member, type_name};

pub(super) type Resources = Vec<(String, Json)>;
pub(super) type Plans = Vec<(String, Json)>;

struct Group<'a> {
    archive: &'a Archive,
    global: &'a BTreeSet<String>,
    parent_name: String,
    family: &'a str,
    source: &'a str,
    target: &'a str,
}

pub(super) fn verify_archive(root: &Path) -> Result<(), HarnessError> {
    if file_sha256(&root.join(ARCHIVE))? != ARCHIVE_SHA256 {
        return Err(HarnessError::invalid(
            "pinned CloudFormation schema archive checksum differs",
        ));
    }
    Ok(())
}

pub(super) fn root_catalog(root: &Path) -> Result<(Json, String), HarnessError> {
    let root_bytes = std::fs::read(root.join(ROOT_CATALOG))?;
    let root_sha = sha256_hex(&root_bytes);
    let root_text = std::str::from_utf8(&root_bytes)
        .map_err(|_| HarnessError::invalid("root catalog is not UTF-8"))?;
    Ok((Json::parse(root_text)?, root_sha))
}

pub(super) fn known_types(root_catalog: &Json) -> BTreeSet<String> {
    root_catalog
        .get("resource_types")
        .and_then(Json::as_array)
        .unwrap_or_default()
        .iter()
        .filter_map(|row| row.get("type_name").and_then(Json::as_str))
        .map(str::to_string)
        .collect()
}

pub(super) fn global_names() -> BTreeSet<String> {
    GLOBAL_SENSITIVE
        .iter()
        .map(|name| (*name).to_string())
        .collect()
}

pub(super) fn collect(
    archive: &Archive,
    known: &BTreeSet<String>,
    global: &BTreeSet<String>,
) -> Result<(Resources, Plans), HarnessError> {
    let mut resources = Resources::new();
    let mut plans = Plans::new();
    for group in GROUPS {
        collect_group(&mut resources, &mut plans, archive, known, global, group)?;
    }
    Ok((resources, plans))
}

pub(super) fn check_count(resources: &Resources) -> Result<(), HarnessError> {
    let unique: BTreeSet<&str> = resources.iter().map(|(name, _)| name.as_str()).collect();
    if resources.len() != super::EXPECTED_TYPES || unique.len() != super::EXPECTED_TYPES {
        return Err(HarnessError::invalid(
            "expected exactly 24 distinct reviewed parent-dependent types",
        ));
    }
    Ok(())
}

fn collect_group(
    resources: &mut Resources,
    plans: &mut Plans,
    archive: &Archive,
    known: &BTreeSet<String>,
    global: &BTreeSet<String>,
    group: &(&str, &str, &str, &str, &str),
) -> Result<(), HarnessError> {
    let (family, parent, source, target, children) = *group;
    let parent_name = format!("AWS::{family}::{parent}");
    let parent_entry = schema_member(archive, family, parent)?;
    validate_parent(&parent_entry.schema, known, &parent_name, source)?;
    let context = Group {
        archive,
        global,
        parent_name,
        family,
        source,
        target,
    };
    for child in children.split_whitespace() {
        collect_child(&context, child, resources, plans)?;
    }
    Ok(())
}

fn validate_parent(
    schema: &Value,
    known: &BTreeSet<String>,
    parent_name: &str,
    source: &str,
) -> Result<(), HarnessError> {
    let source_ok = properties(schema)
        .get(source)
        .is_some_and(|prop| matches!(basic_type(prop, schema), Ok("string")));
    if !known.contains(parent_name) || !source_ok {
        return Err(HarnessError::invalid(format!(
            "unreviewed or non-string parent source {parent_name}/{source}"
        )));
    }
    Ok(())
}

fn collect_child(
    context: &Group<'_>,
    child: &str,
    resources: &mut Resources,
    plans: &mut Plans,
) -> Result<(), HarnessError> {
    let entry = schema_member(context.archive, context.family, child)?;
    validate_child(&entry.schema, context.target)?;
    let descriptor = make_descriptor(context.family, child, entry, &entry.schema, context.global)?;
    validate_redacted(&descriptor, context.target)?;
    let name = type_name(&entry.schema).to_string();
    resource_push(
        resources,
        plans,
        name,
        descriptor,
        type_name(&entry.schema),
        context,
    );
    Ok(())
}

fn validate_child(schema: &Value, target: &str) -> Result<(), HarnessError> {
    validate_handlers(schema)?;
    let listing = list_model(schema)?;
    validate_list_model(schema, listing, target)?;
    validate_target_property(schema, target)?;
    validate_required_context(listing, target, schema)
}

fn resource_push(
    resources: &mut Resources,
    plans: &mut Plans,
    name: String,
    descriptor: Json,
    schema_name: &str,
    context: &Group<'_>,
) {
    resources.push((name.clone(), descriptor));
    plans.push((
        name,
        super::plan::plan_row(
            schema_name,
            &context.parent_name,
            context.source,
            context.target,
        ),
    ));
}

fn validate_handlers(schema: &Value) -> Result<(), HarnessError> {
    let declared = handlers(schema).unwrap_or_default();
    if !(declared.iter().any(|key| key == "read") && declared.iter().any(|key| key == "list")) {
        return Err(HarnessError::invalid(format!(
            "child lacks read/list handlers: {}",
            type_name(schema)
        )));
    }
    Ok(())
}

fn list_model(schema: &Value) -> Result<&Value, HarnessError> {
    schema
        .get("handlers")
        .and_then(|handlers| handlers.get("list"))
        .and_then(|handler| handler.get("handlerSchema"))
        .ok_or_else(|| {
            HarnessError::invalid(format!("child lacks a list model: {}", type_name(schema)))
        })
}

fn validate_list_model(schema: &Value, listing: &Value, target: &str) -> Result<(), HarnessError> {
    let has_target = listing
        .get("properties")
        .and_then(Value::as_object)
        .is_some_and(|properties| properties.contains_key(target));
    if empty_model_valid(listing)? || !has_target {
        return Err(HarnessError::invalid(format!(
            "child is not a reviewed parent-only list: {}",
            type_name(schema)
        )));
    }
    Ok(())
}

fn validate_target_property(schema: &Value, target: &str) -> Result<(), HarnessError> {
    let child_properties = properties(schema);
    let target_prop = child_properties.get(target).ok_or_else(|| {
        HarnessError::invalid(format!(
            "non-string child parent property {}/{target}",
            type_name(schema)
        ))
    })?;
    if basic_type(target_prop, schema)? != "string" {
        return Err(HarnessError::invalid(format!(
            "non-string child parent property {}/{target}",
            type_name(schema)
        )));
    }
    Ok(())
}

fn validate_required_context(
    listing: &Value,
    target: &str,
    schema: &Value,
) -> Result<(), HarnessError> {
    let required = required_list(listing.get("required"));
    let alternatives: Vec<Vec<String>> = listing
        .get("oneOf")
        .and_then(Value::as_array)
        .map(|parts| {
            parts
                .iter()
                .map(|part| required_list(part.get("required")))
                .collect()
        })
        .unwrap_or_default();
    let expected = vec![target.to_string()];
    if required != expected && !alternatives.contains(&expected) {
        return Err(HarnessError::invalid(format!(
            "list requires an additional unreviewed context: {}",
            type_name(schema)
        )));
    }
    Ok(())
}

fn validate_redacted(descriptor: &Json, target: &str) -> Result<(), HarnessError> {
    let forbidden = format!("/{target}");
    let redacted_binding = descriptor
        .get("write_only_properties")
        .and_then(Json::as_array)
        .into_iter()
        .flatten()
        .chain(
            descriptor
                .get("sensitive_properties")
                .and_then(Json::as_array)
                .into_iter()
                .flatten(),
        )
        .any(|value| value.as_str() == Some(forbidden.as_str()));
    if redacted_binding {
        return Err(HarnessError::invalid(
            "parent binding targets a redacted property",
        ));
    }
    Ok(())
}
