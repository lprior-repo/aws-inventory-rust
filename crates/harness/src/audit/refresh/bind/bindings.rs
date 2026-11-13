//! Artifact-digest bindings recorded in the refreshed coverage evidence.

use std::path::Path;

use serde_json::Value;

use crate::HarnessError;
use crate::audit::Json;

use super::super::super::files::relative_slash;
use super::Run;

pub(super) fn binding_json(
    root: &Path,
    run: &Run,
    catalog_sha: &str,
    binary_sha: Option<&str>,
    scan_id: &str,
    fixture_sha: String,
    source: &Value,
) -> Result<Json, HarnessError> {
    let mut fields = run_file_bindings(root, run)?;
    fields.extend(database_bindings(root, run, scan_id)?);
    fields.extend(execution_bindings(run, binary_sha, source));
    fields.extend(model_bindings(run, catalog_sha, fixture_sha));
    fields.extend(catalog_command_bindings(root, run)?);
    fields.push((
        "semantics".to_owned(),
        Json::String(
            "Rust request ledger plus typed Cloud Control GetResource observations in the asserted scan"
                .to_owned(),
        ),
    ));
    Ok(Json::Object(fields))
}

fn run_file_bindings(root: &Path, run: &Run) -> Result<Vec<(String, Json)>, HarnessError> {
    Ok(vec![
        (
            "summary_file".to_owned(),
            Json::String(relative_slash(root, &run.summary_path)?),
        ),
        (
            "summary_sha256".to_owned(),
            Json::String(run.summary_sha.clone()),
        ),
        (
            "command_file".to_owned(),
            Json::String(relative_slash(root, &run.commands.cloud_control_path)?),
        ),
        (
            "command_sha256".to_owned(),
            Json::String(run.commands.cloud_control_sha.clone()),
        ),
    ])
}

fn database_bindings(
    root: &Path,
    run: &Run,
    scan_id: &str,
) -> Result<Vec<(String, Json)>, HarnessError> {
    let database = run.parent.join("inventory.sqlite");
    Ok(vec![
        (
            "database_file".to_owned(),
            Json::String(relative_slash(root, &database)?),
        ),
        ("scan_id".to_owned(), Json::String(scan_id.to_owned())),
    ])
}

fn execution_bindings(run: &Run, binary_sha: Option<&str>, source: &Value) -> Vec<(String, Json)> {
    let emulator_source_commit = run
        .summary
        .pointer("/emulator/source_commit")
        .cloned()
        .unwrap_or(Value::Null);
    let source_sha = source.get("sha256").cloned().unwrap_or(Value::Null);
    vec![
        (
            "emulator_source_commit".to_owned(),
            Json::from_value(&emulator_source_commit),
        ),
        ("source_sha256".to_owned(), Json::from_value(&source_sha)),
        (
            "binary_sha256".to_owned(),
            Json::String(binary_sha.unwrap_or_default().to_owned()),
        ),
    ]
}

fn model_bindings(run: &Run, catalog_sha: &str, fixture_sha: String) -> Vec<(String, Json)> {
    vec![
        (
            "catalog_sha256".to_owned(),
            Json::String(catalog_sha.to_owned()),
        ),
        (
            "architecture_fixture_sha256".to_owned(),
            Json::String(fixture_sha),
        ),
        (
            "parent_catalog_sha256".to_owned(),
            Json::String(run.parent_catalog_sha.clone()),
        ),
        (
            "runtime_catalog_canonical_sha256".to_owned(),
            Json::String(run.runtime_catalog_sha.clone()),
        ),
    ]
}

fn catalog_command_bindings(root: &Path, run: &Run) -> Result<Vec<(String, Json)>, HarnessError> {
    Ok(vec![
        (
            "catalog_command_file".to_owned(),
            Json::String(relative_slash(root, &run.commands.catalog_path)?),
        ),
        (
            "catalog_command_sha256".to_owned(),
            Json::String(run.commands.catalog_sha.clone()),
        ),
    ])
}
