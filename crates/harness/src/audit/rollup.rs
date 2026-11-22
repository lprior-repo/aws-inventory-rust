use std::fs;
use std::path::Path;

use crate::HarnessError;

use super::Json;
use super::format::group_thousands;
use super::jhelpers::{array, field, integer, object, text};

pub(super) fn read_ordered(path: &Path) -> Result<(Json, String), HarnessError> {
    let bytes = crate::files::read_bytes(path, super::files::MAX_DOCUMENT_BYTES)
        .map_err(super::files::budget_message)?;
    let digest = crate::provenance::sha256_hex(&bytes);
    let content = String::from_utf8(bytes)
        .map_err(|_| HarnessError::invalid("coverage evidence document is not UTF-8"))?;
    Ok((Json::parse(&content)?, digest))
}

fn histogram_lines(summary: &Json) -> Result<Vec<String>, HarnessError> {
    let exclusions = object(summary, "exclusions")?;
    let mut sorted: Vec<&(String, Json)> = exclusions.iter().collect();
    sorted.sort_by(|left, right| left.0.cmp(&right.0));
    let mut lines = Vec::new();
    for (reason, count) in sorted {
        let count = count
            .as_i64()
            .ok_or_else(|| HarnessError::invalid("invalid exclusion count"))?;
        lines.push(format!("| {reason} | {} |", group_thousands(count)));
    }
    Ok(lines)
}

fn emulator_header(emulator: &Json) -> Result<Vec<String>, HarnessError> {
    let summary_file = text(emulator, "summary_file")?;
    let command_file = text(emulator, "command_file")?;
    let database_file = text(emulator, "database_file")?;
    Ok(vec![
        format!("- Summary: `{summary_file}`."),
        format!(
            "- Summary SHA-256: `{}`.",
            text(emulator, "summary_sha256")?
        ),
        format!("- Cloud Control scan receipt: `{command_file}`."),
        format!("- Database: `{database_file}`."),
        "- Python fixture creation is excluded from operation coverage. A successful HTTP response"
            .to_owned(),
        "  alone does not establish a successful typed, stored read.".to_owned(),
        String::new(),
        "| Observed per-type outcome | Count |".to_owned(),
        "| --- | ---: |".to_owned(),
    ])
}

fn outcome_lines(summary: &Json) -> Result<Vec<String>, HarnessError> {
    let outcomes = object(summary, "emulator_outcomes")?;
    let mut sorted: Vec<&(String, Json)> = outcomes.iter().collect();
    sorted.sort_by(|left, right| left.0.cmp(&right.0));
    let mut lines = Vec::new();
    for (name, count) in sorted {
        let count = count
            .as_i64()
            .ok_or_else(|| HarnessError::invalid("invalid emulator outcome count"))?;
        lines.push(format!("| {name} | {} |", group_thousands(count)));
    }
    Ok(lines)
}

fn failed_types(matrix: &Json) -> Result<Vec<String>, HarnessError> {
    let entries = array(matrix, "resource_types")?;
    let mut failed = Vec::new();
    for row in entries {
        if row.get("selected").and_then(Json::as_bool) != Some(true) {
            continue;
        }
        let observed = row
            .get("emulator_verification")
            .and_then(|value| value.get("read_observed"))
            .and_then(Json::as_bool);
        if observed != Some(true) {
            failed.push(text(row, "type_name")?.to_owned());
        }
    }
    Ok(failed)
}

fn failed_lines(failed: &[String]) -> Vec<String> {
    let mut lines = vec![
        String::new(),
        "Selected types without an observed detailed read:".to_owned(),
        String::new(),
    ];
    for name in failed {
        lines.push(format!("- `{name}`."));
    }
    lines
}

fn evidence_lines(matrix: &Json) -> Result<Vec<String>, HarnessError> {
    let Some(emulator) = matrix
        .get("evidence")
        .and_then(|value| value.get("emulator"))
        .filter(|value| value.truthy())
    else {
        return Ok(vec![
            "The expanded catalog has not yet been bound to a completed emulator run.".to_owned(),
        ]);
    };
    let mut lines = emulator_header(emulator)?;
    let summary = field(matrix, "summary")?;
    lines.extend(outcome_lines(summary)?);
    let failed = failed_types(matrix)?;
    if !failed.is_empty() {
        lines.extend(failed_lines(&failed));
    }
    Ok(lines)
}

fn measure_line(label: &str, count: i64) -> String {
    format!("| {label} | {} |", group_thousands(count))
}

const MEASURES: [(&str, &str); 7] = [
    ("Retained provider schemas", "schema_count"),
    (
        "Schemas declaring read and list handlers",
        "read_and_list_count",
    ),
    ("Selected executable regional descriptors", "selected_count"),
    ("Declared relationship rules", "relationship_rule_count"),
    (
        "Selected types with relationship rules",
        "types_with_relationships",
    ),
    (
        "Types with observed Rust ListResources requests",
        "emulator_list_attempted",
    ),
    (
        "Types with matching Rust GetResource and persisted detailed observations",
        "emulator_read_observed",
    ),
];

fn measure(summary: &Json, label: &str, key: &str) -> Result<String, HarnessError> {
    Ok(measure_line(label, integer(summary, key)?))
}

fn measure_lines(summary: &Json) -> Result<Vec<String>, HarnessError> {
    MEASURES
        .iter()
        .map(|(label, key)| measure(summary, label, key))
        .collect()
}

fn document_header(summary: &Json) -> Result<Vec<String>, HarnessError> {
    let mut lines = vec![
        "# Cloud Control coverage evidence".to_owned(),
        String::new(),
        "Generated from the pinned capability matrix. Schema support, reviewed collector selection,"
            .to_owned(),
        "relationship rules, and observed emulator reads are separate evidence dimensions."
            .to_owned(),
        String::new(),
        "| Measure | Count |".to_owned(),
        "| --- | ---: |".to_owned(),
    ];
    lines.extend(measure_lines(summary)?);
    lines.extend([
        String::new(),
        "## Exclusions".to_owned(),
        String::new(),
        "| Reason | Count |".to_owned(),
        "| --- | ---: |".to_owned(),
    ]);
    Ok(lines)
}

fn interpretation_lines() -> Vec<String> {
    vec![
        String::new(),
        "## Full matrix and interpretation".to_owned(),
        String::new(),
        "- [All provider capability entries](../catalog/cloudcontrol-capabilities.json)."
            .to_owned(),
        "- [Executable descriptors and property mappings](../catalog/cloudcontrol-resources.json)."
            .to_owned(),
        "- [Collection and relationship design](CLOUDCONTROL_COVERAGE.md).".to_owned(),
        String::new(),
        "The pinned emulator uses a generic property store for most types. These observations"
            .to_owned(),
        "exercise SDK decoding, collection, normalization and storage; they do not certify the"
            .to_owned(),
        "production AWS provider's behavior, IAM permissions, regional availability or quotas."
            .to_owned(),
        String::new(),
    ]
}

pub fn write_rollup(matrix: &Json, path: &Path) -> Result<(), HarnessError> {
    let summary = field(matrix, "summary")?;
    let mut lines = document_header(summary)?;
    lines.extend(histogram_lines(summary)?);
    lines.extend([
        String::new(),
        "## Executed evidence".to_owned(),
        String::new(),
    ]);
    lines.extend(evidence_lines(matrix)?);
    lines.extend(interpretation_lines());
    fs::write(path, lines.join("\n"))?;
    Ok(())
}
