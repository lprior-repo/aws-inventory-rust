//! Bind the capability matrix to observed traffic and persisted typed reads.

mod merge;

pub use merge::apply_verification;

use std::collections::{BTreeMap, BTreeSet};

use crate::HarnessError;

use super::Json;
use super::jhelpers::{array, text};

/// Maximum executable catalog types (and matrix entries) considered.
const MAX_TYPES: usize = 4_096;

/// Checks a complete asserted emulator run must declare for coverage.
const REQUIRED_CHECKS: [&str; 8] = [
    "coverage:all_catalog_types_listed",
    "coverage:every_registered_type_read",
    "sqlite:integrity",
    "cloud_control:architecture_fixture_metadata",
    "provenance:embedded_catalog_matches",
    "provenance:source_unchanged",
    "provenance:binary_unchanged",
    "provenance:fixture_unchanged",
];

fn catalog_types(catalog: &Json) -> Result<BTreeMap<String, Json>, HarnessError> {
    let rows = array(catalog, "resource_types")?;
    let mut types = BTreeMap::new();
    for row in rows {
        let name = text(row, "type_name")?;
        if types.insert(name.to_owned(), row.clone()).is_some() {
            return Err(HarnessError::invalid("invalid executable catalog type set"));
        }
    }
    if types.is_empty() || types.len() != rows.len() || types.len() > MAX_TYPES {
        return Err(HarnessError::invalid("invalid executable catalog type set"));
    }
    Ok(types)
}

fn assert_complete(summary: &Json) -> Result<(), HarnessError> {
    let checks = check_rows(summary)?;
    let names = check_names(&checks);
    if !is_complete(summary, &checks, &names) {
        return Err(HarnessError::invalid(
            "coverage refresh requires a complete asserted emulator run",
        ));
    }
    Ok(())
}

fn check_rows(summary: &Json) -> Result<Vec<Json>, HarnessError> {
    match summary.get("checks") {
        None | Some(Json::Null) => Ok(Vec::new()),
        Some(Json::Array(rows)) => Ok(rows.clone()),
        Some(_) => Err(HarnessError::invalid(
            "coverage refresh requires a complete asserted emulator run",
        )),
    }
}

fn check_names(checks: &[Json]) -> BTreeSet<&str> {
    checks
        .iter()
        .filter_map(|row| row.get("check").and_then(Json::as_str))
        .collect()
}

fn is_complete(summary: &Json, checks: &[Json], names: &BTreeSet<&str>) -> bool {
    summary.get("schema_version").and_then(Json::as_i64) == Some(1)
        && summary.get("failed_checks").and_then(Json::as_i64) == Some(0)
        && !checks.is_empty()
        && checks
            .iter()
            .all(|row| row.get("passed").and_then(Json::as_bool) == Some(true))
        && summary.get("passed_checks").and_then(Json::as_i64) == i64::try_from(checks.len()).ok()
        && REQUIRED_CHECKS.iter().all(|name| names.contains(name))
}

fn executed_block(summary: &Json) -> Result<Json, HarnessError> {
    match summary.get("execution") {
        None | Some(Json::Null) => Ok(Json::Object(Vec::new())),
        Some(value @ Json::Object(_)) => Ok(value.clone()),
        Some(_) => Err(HarnessError::invalid(
            "current catalog/source/binary/fixture differs from executed evidence",
        )),
    }
}

fn observed_rows(
    summary: &Json,
    types: &BTreeMap<String, Json>,
) -> Result<Vec<Json>, HarnessError> {
    let observed = match summary.get("cloud_control_resource_types") {
        None | Some(Json::Null) => Vec::new(),
        Some(Json::Array(rows)) => rows.clone(),
        Some(_) => {
            return Err(HarnessError::invalid(
                "emulator type set differs from the executable catalog",
            ));
        }
    };
    let names: BTreeSet<String> = observed
        .iter()
        .map(|row| text(row, "type_name").map(str::to_owned))
        .collect::<Result<_, _>>()?;
    if observed.len() != types.len() || names != types.keys().cloned().collect() {
        return Err(HarnessError::invalid(
            "emulator type set differs from the executable catalog",
        ));
    }
    Ok(observed)
}
