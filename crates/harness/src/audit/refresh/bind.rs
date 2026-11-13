//! Bind one emulator summary to the reviewed coverage matrix.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::HarnessError;
use crate::audit::Json;
use crate::provenance::{
    canonical_sha256, expected_runtime_catalog, file_sha256, source_fingerprint,
};

use super::super::coverage::apply_verification;
use super::super::files::read_json_document;
use super::super::persisted::persisted_reads;

mod bindings;
mod lookup;

use bindings::binding_json;
use lookup::{expected_status, parent_of, scan};

const SUMMARY_FILE: &str = "cloud-control.json";
const CATALOG_COMMAND_FILE: &str = "catalog.json";
const FIXTURE: &str = "crates/aws/tests/fixtures/cloudcontrol-architecture.json";

struct Commands {
    cloud_control_path: PathBuf,
    cloud_control_sha: String,
    cloud_control: Value,
    catalog_path: PathBuf,
    catalog_sha: String,
    catalog: Value,
}

struct Run {
    summary_path: PathBuf,
    summary_sha: String,
    summary: Value,
    parent: PathBuf,
    commands: Commands,
    scans: Vec<Value>,
    executed: Value,
    parent_catalog_sha: String,
    runtime_catalog: Value,
    runtime_catalog_sha: String,
}

pub(super) async fn bind_matrix(
    root: &Path,
    requested: &Path,
    matrix: &Json,
    catalog: &Value,
    catalog_sha: &str,
) -> Result<Json, HarnessError> {
    let run = load(root, requested, catalog)?;
    let (binary_sha, source) = validate(&run, root)?;
    let binding = binding(root, &run, catalog_sha, binary_sha.as_deref(), &source)?;
    let catalog_json = Json::from_value(catalog);
    let counts = counts(&run, &catalog_json).await?;
    let merged = apply_verification(
        matrix,
        &catalog_json,
        &Json::from_value(&run.summary),
        &counts,
        &binding,
    )?;
    write_matrix(root, &merged)?;
    Ok(merged)
}

fn load(root: &Path, requested: &Path, catalog: &Value) -> Result<Run, HarnessError> {
    let (runtime_catalog, runtime_catalog_sha, parent_catalog_sha) = runtime(root, catalog)?;
    let summary_path = fs::canonicalize(requested)?;
    let (summary, summary_sha) = read_json_document(&summary_path)?;
    let parent = parent_of(&summary_path)?.to_path_buf();
    let commands = commands(&parent)?;
    let scans = command_scans(&commands.cloud_control)?;
    let executed = executed_block(&summary);
    Ok(Run {
        summary_path,
        summary_sha,
        summary,
        parent,
        commands,
        scans,
        executed,
        parent_catalog_sha,
        runtime_catalog,
        runtime_catalog_sha,
    })
}

fn runtime(root: &Path, catalog: &Value) -> Result<(Value, String, String), HarnessError> {
    let (parent_catalog, parent_catalog_sha) =
        read_json_document(&root.join("catalog/cloudcontrol-parent-resources.json"))?;
    let runtime_catalog = expected_runtime_catalog(catalog, &parent_catalog)?;
    let runtime_catalog_sha = canonical_sha256(&runtime_catalog);
    Ok((runtime_catalog, runtime_catalog_sha, parent_catalog_sha))
}

fn commands(parent: &Path) -> Result<Commands, HarnessError> {
    let cloud_control_path = parent.join(SUMMARY_FILE);
    let (cloud_control, cloud_control_sha) = read_json_document(&cloud_control_path)?;
    let catalog_path = parent.join(CATALOG_COMMAND_FILE);
    let (catalog, catalog_sha) = read_json_document(&catalog_path)?;
    Ok(Commands {
        cloud_control_path,
        cloud_control_sha,
        cloud_control,
        catalog_path,
        catalog_sha,
        catalog,
    })
}

fn command_scans(command: &Value) -> Result<Vec<Value>, HarnessError> {
    let scans = command
        .pointer("/result/scans")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if !matches!(
        command.get("exit_code").and_then(Value::as_i64),
        Some(0 | 2)
    ) || scans.len() != 1
    {
        return Err(HarnessError::invalid(
            "coverage requires one completed Rust Cloud Control command",
        ));
    }
    Ok(scans)
}

fn executed_block(summary: &Value) -> Value {
    summary
        .get("execution")
        .filter(|value| value.is_object())
        .cloned()
        .unwrap_or_else(|| Value::Object(serde_json::Map::new()))
}

fn validate(run: &Run, root: &Path) -> Result<(Option<String>, Value), HarnessError> {
    check_catalog_alignment(run)?;
    let source = source_fingerprint(root)?;
    if run.executed.get("source") != Some(&source) {
        return Err(HarnessError::invalid(
            "collector source changed since the asserted emulator run",
        ));
    }
    let binary_sha = check_binary(run)?;
    check_command(run, binary_sha.as_deref())?;
    check_catalog_command(run, binary_sha.as_deref())?;
    Ok((binary_sha, source))
}

fn check_catalog_alignment(run: &Run) -> Result<(), HarnessError> {
    let aligned = run
        .executed
        .get("parent_catalog_sha256")
        .and_then(Value::as_str)
        == Some(run.parent_catalog_sha.as_str())
        && run
            .executed
            .get("runtime_catalog_canonical_sha256")
            .and_then(Value::as_str)
            == Some(run.runtime_catalog_sha.as_str());
    if !aligned {
        return Err(HarnessError::invalid(
            "executed parent/runtime catalog differs from current descriptors",
        ));
    }
    Ok(())
}

fn check_binary(run: &Run) -> Result<Option<String>, HarnessError> {
    let binary = run.executed.get("binary").cloned().unwrap_or(Value::Null);
    let binary_path = Path::new(binary.get("path").and_then(Value::as_str).unwrap_or(""));
    let binary_sha = binary.get("sha256").and_then(Value::as_str);
    if !binary_path.is_file()
        || binary_sha.is_none()
        || Some(file_sha256(binary_path)?.as_str()) != binary_sha
    {
        return Err(HarnessError::invalid(
            "executed collector binary is unavailable or changed",
        ));
    }
    Ok(binary_sha.map(str::to_owned))
}

fn check_command(run: &Run, binary_sha: Option<&str>) -> Result<(), HarnessError> {
    let expected = run
        .executed
        .get("cloud_control_command")
        .cloned()
        .unwrap_or(Value::Null);
    let bound = expected.get("file").and_then(Value::as_str) == Some(SUMMARY_FILE)
        && expected.get("sha256").and_then(Value::as_str)
            == Some(run.commands.cloud_control_sha.as_str())
        && expected.get("scan_id") == run.scans.first().and_then(|row| row.get("scan_id"))
        && expected.get("binary_sha256").and_then(Value::as_str) == binary_sha
        && run
            .commands
            .cloud_control
            .get("binary_sha256")
            .and_then(Value::as_str)
            == binary_sha;
    if !bound {
        return Err(HarnessError::invalid(
            "summary is not bound to this Cloud Control command and scan",
        ));
    }
    Ok(())
}

fn check_catalog_command(run: &Run, binary_sha: Option<&str>) -> Result<(), HarnessError> {
    let expected = run
        .executed
        .get("catalog_command")
        .cloned()
        .unwrap_or(Value::Null);
    if !catalog_command_bound(run, &expected, binary_sha) {
        return Err(HarnessError::invalid(
            "executed binary catalog differs from the current descriptor catalog",
        ));
    }
    Ok(())
}

fn catalog_command_bound(run: &Run, expected: &Value, binary_sha: Option<&str>) -> bool {
    expected.get("file").and_then(Value::as_str) == Some(CATALOG_COMMAND_FILE)
        && expected.get("sha256").and_then(Value::as_str) == Some(run.commands.catalog_sha.as_str())
        && run
            .commands
            .catalog
            .get("exit_code")
            .and_then(Value::as_i64)
            == Some(0)
        && run
            .commands
            .catalog
            .get("binary_sha256")
            .and_then(Value::as_str)
            == binary_sha
        && run.commands.catalog.pointer("/result/cloud_control") == Some(&run.runtime_catalog)
}

fn binding(
    root: &Path,
    run: &Run,
    catalog_sha: &str,
    binary_sha: Option<&str>,
    source: &Value,
) -> Result<Json, HarnessError> {
    let scan = scan(run)?;
    let scan_id = text_of(scan, "scan_id")?;
    let fixture_sha = file_sha256(&root.join(FIXTURE))?;
    binding_json(
        root,
        run,
        catalog_sha,
        binary_sha,
        scan_id,
        fixture_sha,
        source,
    )
}

async fn counts(run: &Run, catalog: &Json) -> Result<BTreeMap<String, i64>, HarnessError> {
    let scan = scan(run)?;
    let scan_id = text_of(scan, "scan_id")?;
    let expected_status = expected_status(scan)?;
    persisted_reads(
        &run.parent.join("inventory.sqlite"),
        scan_id,
        catalog,
        expected_status,
    )
    .await
}

fn write_matrix(root: &Path, merged: &Json) -> Result<(), HarnessError> {
    let target = root.join("catalog/cloudcontrol-capabilities.json");
    let temporary = target.with_file_name("cloudcontrol-capabilities.json.tmp");
    let mut encoded = super::super::pretty(merged);
    encoded.push('\n');
    fs::write(&temporary, encoded)?;
    fs::rename(&temporary, &target)?;
    Ok(())
}

fn text_of<'a>(value: &'a Value, key: &str) -> Result<&'a str, HarnessError> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| HarnessError::invalid(format!("missing {key}")))
}
