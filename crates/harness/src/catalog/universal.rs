//! Universal Cloud Control catalog over every schema in the pinned archive.
//!
//! The reviewed capability matrix stays the source of truth for handler
//! availability, request-model evidence, exclusions and schema digests. This
//! projection keeps all 1,881 archived provider types instead of the reviewed
//! 397, so no schema is silently unreachable. Visibility is emitted from the
//! schema when it declares one; the pinned archive declares none, so every row
//! is `PUBLIC`. A parent type is emitted only where the reviewed parent catalog
//! states one; schema request models never name their parent type.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde_json::Value;

use crate::HarnessError;
use crate::catalog::archive;
use crate::catalog::json::Json;
use crate::catalog::{ARCHIVE, ARCHIVE_SHA256, CAPABILITY_MATRIX, PARENT_CATALOG};
use crate::provenance::sha256_hex;

mod output;

/// Schema members in the pinned archive.
pub const SCHEMA_COUNT: usize = 1881;

/// Exclusions that mean listing needs a parent request context.
const PARENT_REASONS: [&str; 2] = [
    "schema_parent_model_required",
    "documented_parent_model_required",
];

struct Totals {
    total: usize,
    list: usize,
    parent: usize,
    unsupported: usize,
}

/// Build the universal catalog document.
pub fn document(root: &Path) -> Result<Json, HarnessError> {
    let archive = archive::load(root)?;
    let matrix = read(&root.join(CAPABILITY_MATRIX))?;
    let parents = parent_types(root)?;
    let rows = matrix_rows(&matrix, &archive)?;
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut counts: BTreeMap<&'static str, usize> = BTreeMap::new();
    let mut resource_types: Vec<(String, Json)> = Vec::with_capacity(rows.len());
    for row in rows {
        resource_types.push(entry_row(row, &archive, &parents, &mut seen, &mut counts)?);
    }
    check_coverage(seen.len(), &archive)?;
    let totals = totals(&mut resource_types, &counts)?;
    Ok(document_json(&archive, totals, resource_types))
}

fn matrix_rows<'a>(
    matrix: &'a Json,
    archive: &archive::Archive,
) -> Result<&'a [Json], HarnessError> {
    let rows = matrix
        .get("resource_types")
        .and_then(Json::as_array)
        .ok_or_else(|| HarnessError::invalid("capability matrix lacks resource_types"))?;
    if rows.len() != SCHEMA_COUNT || archive.all_count != SCHEMA_COUNT {
        return Err(HarnessError::invalid(format!(
            "expected {SCHEMA_COUNT} schemas, matrix has {} and archive has {}",
            rows.len(),
            archive.all_count
        )));
    }
    if archive.entries.len() != archive.all_count {
        return Err(HarnessError::invalid("schema archive repeats a type name"));
    }
    Ok(rows)
}

fn entry_row(
    row: &Json,
    archive: &archive::Archive,
    parents: &BTreeMap<String, String>,
    seen: &mut BTreeSet<String>,
    counts: &mut BTreeMap<&'static str, usize>,
) -> Result<(String, Json), HarnessError> {
    let (name, entry, digest) = checked_entry(row, archive, seen)?;
    let exclusion = row.get("exclusion").and_then(Json::as_str);
    let listing = row
        .get("list_supported")
        .and_then(Json::as_bool)
        .unwrap_or(false);
    let capability = capability(exclusion, listing);
    let slot = counts.entry(capability).or_insert(0);
    *slot = slot.saturating_add(1);
    Ok((
        name.to_string(),
        resource_row(name, digest, entry, exclusion, capability, parents),
    ))
}

fn checked_entry<'a>(
    row: &'a Json,
    archive: &'a archive::Archive,
    seen: &mut BTreeSet<String>,
) -> Result<(&'a str, &'a archive::Entry, &'a str), HarnessError> {
    let name = text(row, "type_name")?;
    if !seen.insert(name.to_string()) {
        return Err(HarnessError::invalid(format!(
            "capability matrix repeats {name}"
        )));
    }
    let entry = archive
        .get(name)
        .ok_or_else(|| HarnessError::invalid(format!("archive lacks {name}")))?;
    let digest = text(row, "schema_sha256")?;
    if digest != sha256_hex(&entry.raw) {
        return Err(HarnessError::invalid(format!(
            "schema digest differs for {name}"
        )));
    }
    Ok((name, entry, digest))
}

fn resource_row(
    name: &str,
    digest: &str,
    entry: &archive::Entry,
    exclusion: Option<&str>,
    capability: &str,
    parents: &BTreeMap<String, String>,
) -> Json {
    Json::object(vec![
        ("type_name", Json::string(name)),
        ("schema_sha256", Json::string(digest)),
        ("visibility", Json::string(visibility(&entry.schema))),
        ("capability", Json::string(capability)),
        ("parent_type", parent_type(capability, name, parents)),
        (
            "exclusion",
            exclusion.map(Json::string).unwrap_or(Json::Null),
        ),
    ])
}

fn check_coverage(seen: usize, archive: &archive::Archive) -> Result<(), HarnessError> {
    if seen != archive.entries.len() {
        return Err(HarnessError::invalid(
            "capability matrix does not cover every archived schema",
        ));
    }
    Ok(())
}

fn totals(
    resource_types: &mut [(String, Json)],
    counts: &BTreeMap<&'static str, usize>,
) -> Result<Totals, HarnessError> {
    resource_types.sort_by(|left, right| left.0.cmp(&right.0));
    let total = resource_types.len();
    let list = counts.get("list").copied().unwrap_or(0);
    let parent = counts.get("parent").copied().unwrap_or(0);
    let unsupported = counts.get("unsupported").copied().unwrap_or(0);
    if total != list.saturating_add(parent).saturating_add(unsupported) {
        return Err(HarnessError::invalid(
            "universal capability partition does not sum to the schema count",
        ));
    }
    Ok(Totals {
        total,
        list,
        parent,
        unsupported,
    })
}

fn document_json(
    archive: &archive::Archive,
    totals: Totals,
    resource_types: Vec<(String, Json)>,
) -> Json {
    Json::object(vec![
        ("schema_version", Json::count(1)),
        ("source", source_block(archive)),
        ("summary", summary_block(&totals)),
        (
            "resource_types",
            Json::array(resource_types.into_iter().map(|(_, row)| row).collect()),
        ),
    ])
}

fn source_block(archive: &archive::Archive) -> Json {
    Json::object(vec![
        ("archive", Json::string(ARCHIVE)),
        ("sha256", Json::string(ARCHIVE_SHA256)),
        ("schema_count", Json::count(archive.all_count)),
    ])
}

fn summary_block(totals: &Totals) -> Json {
    Json::object(vec![
        ("total", Json::count(totals.total)),
        ("list_capable", Json::count(totals.list)),
        ("parent_required", Json::count(totals.parent)),
        ("unsupported", Json::count(totals.unsupported)),
    ])
}

pub use output::{UNIVERSAL_CATALOG, check, rendered, run};

fn capability(exclusion: Option<&str>, listing: bool) -> &'static str {
    if exclusion.is_some_and(|reason| PARENT_REASONS.contains(&reason)) {
        "parent"
    } else if listing {
        "list"
    } else {
        "unsupported"
    }
}

fn visibility(schema: &Value) -> &str {
    schema
        .get("visibility")
        .and_then(Value::as_str)
        .unwrap_or("PUBLIC")
}

fn parent_type(capability: &str, name: &str, parents: &BTreeMap<String, String>) -> Json {
    if capability == "parent"
        && let Some(parent) = parents.get(name)
    {
        return Json::string(parent);
    }
    Json::Null
}

fn parent_types(root: &Path) -> Result<BTreeMap<String, String>, HarnessError> {
    let catalog = read(&root.join(PARENT_CATALOG))?;
    let mut parents = BTreeMap::new();
    for plan in catalog
        .get("parent_plans")
        .and_then(Json::as_array)
        .unwrap_or_default()
    {
        let name = text(plan, "type_name")?;
        let parent = text(plan, "parent_type")?;
        parents.insert(name.to_string(), parent.to_string());
    }
    Ok(parents)
}

fn read(path: &Path) -> Result<Json, HarnessError> {
    Json::parse(&std::fs::read_to_string(path)?)
}

fn text<'a>(value: &'a Json, key: &str) -> Result<&'a str, HarnessError> {
    value
        .get(key)
        .and_then(Json::as_str)
        .ok_or_else(|| HarnessError::invalid(format!("missing {key}")))
}
