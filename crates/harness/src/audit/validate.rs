//! Require a complete asserted emulator run before binding its operations.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Map, Value, json};

use crate::HarnessError;
use crate::provenance::canonical_sha256;

/// Checks a complete asserted emulator run must declare.
const REQUIRED_CHECKS: [&str; 6] = [
    "provenance:embedded_catalog_matches",
    "provenance:source_unchanged",
    "provenance:binary_unchanged",
    "provenance:fixture_unchanged",
    "coverage:every_operation_attempted",
    "coverage:no_unexplained_operation_failures",
];

fn truthy(value: Option<&Value>) -> bool {
    match value {
        Some(Value::Null) | None => false,
        Some(Value::Bool(flag)) => *flag,
        Some(Value::Number(number)) => number.as_f64().map(|value| value != 0.0).unwrap_or(false),
        Some(Value::String(text)) => !text.is_empty(),
        Some(Value::Array(items)) => !items.is_empty(),
        Some(Value::Object(map)) => !map.is_empty(),
    }
}

fn executed_object(summary: &Value) -> Result<Map<String, Value>, HarnessError> {
    match summary.get("execution") {
        None | Some(Value::Null) => Ok(Map::new()),
        Some(Value::Object(map)) => Ok(map.clone()),
        Some(_) => Err(HarnessError::invalid("invalid emulator execution block")),
    }
}

fn check_set(summary: &Value) -> Result<(Vec<Value>, BTreeSet<String>), HarnessError> {
    let checks = match summary.get("checks") {
        None | Some(Value::Null) => Vec::new(),
        Some(Value::Array(rows)) => rows.clone(),
        Some(_) => return Err(HarnessError::invalid("invalid emulator check list")),
    };
    let mut names = BTreeSet::new();
    for row in &checks {
        if let Some(name) = row.get("check").and_then(Value::as_str) {
            names.insert(name.to_owned());
        }
    }
    Ok((checks, names))
}

fn complete_run(summary: &Value, checks: &[Value], names: &BTreeSet<String>) -> bool {
    summary.get("schema_version") == Some(&json!(1))
        && summary.get("failed_checks") == Some(&json!(0))
        && !checks.is_empty()
        && checks
            .iter()
            .all(|row| row.get("passed") == Some(&Value::Bool(true)))
        && summary.get("passed_checks") == Some(&json!(checks.len()))
        && REQUIRED_CHECKS.iter().all(|name| names.contains(*name))
}

fn require_complete(summary: &Value) -> Result<(), HarnessError> {
    let (checks, names) = check_set(summary)?;
    if !complete_run(summary, &checks, &names) {
        return Err(HarnessError::invalid(
            "emulator binding requires a complete asserted run",
        ));
    }
    Ok(())
}

fn require_alignment(
    report: &Value,
    executed: &Map<String, Value>,
    dimension: &Value,
    catalog: &Value,
) -> Result<(), HarnessError> {
    let aligned = executed.get("source") == report.get("source")
        && executed.get("catalog_sha256") == dimension.get("catalog_sha256")
        && executed.get("catalog_canonical_sha256")
            == Some(&Value::String(canonical_sha256(catalog)));
    if !aligned {
        return Err(HarnessError::invalid(
            "emulator evidence belongs to a different source or catalog",
        ));
    }
    Ok(())
}

fn require_parent_alignment(
    executed: &Map<String, Value>,
    dimension: &Value,
) -> Result<(), HarnessError> {
    if !dimension
        .get("parent_catalog_sha256")
        .is_some_and(|value| !value.is_null())
    {
        return Ok(());
    }
    let aligned = executed.get("parent_catalog_sha256") == dimension.get("parent_catalog_sha256")
        && executed.get("runtime_catalog_canonical_sha256")
            == dimension.get("runtime_catalog_canonical_sha256");
    if !aligned {
        return Err(HarnessError::invalid(
            "emulator evidence belongs to a different parent or merged runtime catalog",
        ));
    }
    Ok(())
}

fn expected_rows(matrix: &Value) -> Result<BTreeMap<String, &Value>, HarnessError> {
    let mut expected: BTreeMap<String, &Value> = BTreeMap::new();
    for key in ["operations", "credential_provider_boundaries"] {
        let rows = matrix
            .get(key)
            .and_then(Value::as_array)
            .ok_or_else(|| HarnessError::invalid(format!("invalid {key}")))?;
        for row in rows {
            let id = row
                .get("id")
                .and_then(Value::as_str)
                .ok_or_else(|| HarnessError::invalid("invalid operation identity"))?;
            expected.insert(id.to_owned(), row);
        }
    }
    Ok(expected)
}

fn summary_rows(summary: &Value) -> Result<Vec<Value>, HarnessError> {
    match summary.get("expected_operations") {
        None | Some(Value::Null) => Ok(Vec::new()),
        Some(Value::Array(rows)) => Ok(rows.clone()),
        Some(_) => Err(HarnessError::invalid("invalid expected_operations")),
    }
}

fn require_identities(
    rows: &[Value],
    expected: &BTreeMap<String, &Value>,
) -> Result<(), HarnessError> {
    let identities: BTreeSet<&str> = rows
        .iter()
        .filter_map(|row| row.get("id").and_then(Value::as_str))
        .collect();
    let expected_ids: BTreeSet<&str> = expected.keys().map(String::as_str).collect();
    if rows.len() != expected.len() || identities != expected_ids {
        return Err(HarnessError::invalid(
            "emulator operation identity set differs from reconciled boundaries",
        ));
    }
    Ok(())
}

fn require_classified(row: &Value, old: &Value) -> Result<(), HarnessError> {
    let boundary = !truthy(old.get("explicit_sdk_send"));
    let classified = row.get("service") == old.get("boto3_service")
        && row.get("operation") == old.get("api_operation")
        && row.get("credential_provider_boundary") == Some(&Value::Bool(boundary));
    if !classified {
        return Err(HarnessError::invalid(
            "emulator operation identity/provider classification differs",
        ));
    }
    Ok(())
}

fn require_rows(rows: &[Value], expected: &BTreeMap<String, &Value>) -> Result<(), HarnessError> {
    require_identities(rows, expected)?;
    for row in rows {
        let id = row
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(|| HarnessError::invalid("invalid operation identity"))?;
        let old = expected
            .get(id)
            .ok_or_else(|| HarnessError::invalid("invalid operation identity"))?;
        require_classified(row, old)?;
    }
    Ok(())
}

/// Require a complete asserted emulator run and its operation identities.
pub fn validate_summary(
    report: &Value,
    summary: &Value,
    catalog: &Value,
    matrix: &Value,
) -> Result<(Value, Vec<Value>), HarnessError> {
    require_complete(summary)?;
    let executed = executed_object(summary)?;
    let dimension = report
        .get("cloud_control_provider_dimension")
        .ok_or_else(|| HarnessError::invalid("missing provider dimension"))?;
    require_alignment(report, &executed, dimension, catalog)?;
    require_parent_alignment(&executed, dimension)?;
    let expected = expected_rows(matrix)?;
    let rows = summary_rows(summary)?;
    require_rows(&rows, &expected)?;
    Ok((Value::Object(executed), rows))
}
