//! Bind a completed emulator run to the current source, binary and catalogs.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Map, Value, json};

use crate::HarnessError;
use crate::provenance::file_sha256;

use super::files::{read_json_audit, relative_slash};
use super::ledger::request_evidence;
use super::matrix::{Catalogs, catalogs};
use super::validate::validate_summary;

type Receipts = (Map<String, Value>, BTreeMap<&'static str, Value>);

const COMMANDS: [(&str, &str); 2] = [
    ("catalog_command", "catalog.json"),
    ("cloud_control_command", "cloud-control.json"),
];

struct Context {
    summary_path: PathBuf,
    summary: Value,
    summary_sha: String,
    frozen: Catalogs,
    executed: Value,
    rows: Vec<Value>,
}

/// Bind a completed emulator run to the current source, binary and catalogs.
pub fn bind_emulator(
    report: &Value,
    root: &Path,
    summary_path: &Path,
) -> Result<Value, HarnessError> {
    let context = load(report, root, summary_path)?;
    let binary_sha = check_binary(&context.executed)?;
    check_fixture(root, &context.executed)?;
    let (mut artifacts, commands) = receipts(root, &context, binary_sha.as_deref())?;
    check_catalog(&commands, &context.frozen)?;
    check_cloud_control(&commands, &context.executed, binary_sha.as_deref())?;
    let ledger = ledger_evidence(root, &context, &mut artifacts)?;
    Ok(receipt(&context, artifacts, ledger))
}

fn load(report: &Value, root: &Path, summary_path: &Path) -> Result<Context, HarnessError> {
    let summary_path = fs::canonicalize(summary_path)?;
    let (summary, summary_sha) = read_json_audit(&summary_path)?;
    let (matrix, _) = read_json_audit(&root.join("catalog/aws-operations.json"))?;
    let frozen = catalogs(root)?;
    let (executed, rows) = validate_summary(report, &summary, &frozen.catalog, &matrix)?;
    Ok(Context {
        summary_path,
        summary,
        summary_sha,
        frozen,
        executed,
        rows,
    })
}

fn check_binary(executed: &Value) -> Result<Option<String>, HarnessError> {
    let binary = executed.get("binary").cloned().unwrap_or(Value::Null);
    let binary_path = Path::new(binary.get("path").and_then(Value::as_str).unwrap_or(""));
    let binary_sha = binary.get("sha256").and_then(Value::as_str);
    if !binary_path.is_file()
        || binary_sha.is_none()
        || Some(file_sha256(binary_path)?.as_str()) != binary_sha
    {
        return Err(HarnessError::invalid(
            "executed collector binary is absent or changed",
        ));
    }
    Ok(binary_sha.map(str::to_owned))
}

fn check_fixture(root: &Path, executed: &Value) -> Result<(), HarnessError> {
    let fixture = root.join("crates/aws/tests/fixtures/cloudcontrol-architecture.json");
    if Some(file_sha256(&fixture)?.as_str())
        != executed
            .get("architecture_fixture_sha256")
            .and_then(Value::as_str)
    {
        return Err(HarnessError::invalid(
            "architecture fixture changed after emulator execution",
        ));
    }
    Ok(())
}

fn receipts(
    root: &Path,
    context: &Context,
    binary_sha: Option<&str>,
) -> Result<Receipts, HarnessError> {
    let mut artifacts = summary_artifact(root, context)?;
    let parent = parent_of(&context.summary_path)?;
    let mut commands: BTreeMap<&str, Value> = BTreeMap::new();
    for (key_name, filename) in COMMANDS {
        let path = parent.join(filename);
        let (command, digest) = read_json_audit(&path)?;
        check_receipt(
            &context.executed,
            key_name,
            filename,
            &digest,
            &command,
            binary_sha,
        )?;
        artifacts.insert(relative_slash(root, &path)?, Value::String(digest));
        commands.insert(key_name, command);
    }
    Ok((artifacts, commands))
}

fn summary_artifact(root: &Path, context: &Context) -> Result<Map<String, Value>, HarnessError> {
    let mut artifacts = Map::new();
    artifacts.insert(
        relative_slash(root, &context.summary_path)?,
        Value::String(context.summary_sha.clone()),
    );
    Ok(artifacts)
}

fn check_receipt(
    executed: &Value,
    key_name: &str,
    filename: &str,
    digest: &str,
    command: &Value,
    binary_sha: Option<&str>,
) -> Result<(), HarnessError> {
    let binding = executed.get(key_name).cloned().unwrap_or(Value::Null);
    let bound = binding.get("file").and_then(Value::as_str) == Some(filename)
        && binding.get("sha256").and_then(Value::as_str) == Some(digest)
        && command.get("binary_sha256").and_then(Value::as_str) == binary_sha;
    if !bound {
        return Err(HarnessError::invalid(
            "emulator command receipt is not bound to this run/binary",
        ));
    }
    Ok(())
}

fn check_catalog(commands: &BTreeMap<&str, Value>, frozen: &Catalogs) -> Result<(), HarnessError> {
    let catalog_command = commands
        .get("catalog_command")
        .ok_or_else(|| HarnessError::invalid("missing catalog command receipt"))?;
    if catalog_command.get("exit_code") != Some(&json!(0))
        || catalog_command.pointer("/result/cloud_control") != Some(&frozen.merged)
    {
        return Err(HarnessError::invalid(
            "executed binary catalog differs from current descriptor data",
        ));
    }
    Ok(())
}

fn check_cloud_control(
    commands: &BTreeMap<&str, Value>,
    executed: &Value,
    binary_sha: Option<&str>,
) -> Result<(), HarnessError> {
    let command = commands
        .get("cloud_control_command")
        .ok_or_else(|| HarnessError::invalid("missing Cloud Control command receipt"))?;
    let scans = command
        .pointer("/result/scans")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let expected = executed
        .get("cloud_control_command")
        .cloned()
        .unwrap_or(Value::Null);
    if !cloud_control_bound(command, &scans, &expected, binary_sha) {
        return Err(HarnessError::invalid(
            "Cloud Control command/scan binding differs",
        ));
    }
    Ok(())
}

fn cloud_control_bound(
    command: &Value,
    scans: &[Value],
    expected: &Value,
    binary_sha: Option<&str>,
) -> bool {
    let scan_id = scans.first().and_then(|row| row.get("scan_id")).cloned();
    matches!(
        command.get("exit_code").and_then(Value::as_i64),
        Some(0 | 2)
    ) && scans.len() == 1
        && scan_id == expected.get("scan_id").cloned()
        && expected.get("binary_sha256").and_then(Value::as_str) == binary_sha
}

fn ledger_evidence(
    root: &Path,
    context: &Context,
    artifacts: &mut Map<String, Value>,
) -> Result<Value, HarnessError> {
    let ledger = parent_of(&context.summary_path)?.join("requests.jsonl");
    let evidence = request_evidence(&ledger, &context.rows)?;
    if let Some(digest) = evidence.get("sha256").and_then(Value::as_str) {
        artifacts.insert(
            relative_slash(root, &ledger)?,
            Value::String(digest.to_owned()),
        );
    }
    Ok(evidence)
}

fn receipt(context: &Context, artifacts: Map<String, Value>, ledger: Value) -> Value {
    json!({
        "status": "current_execution_receipts_bound",
        "current_run_bound": true,
        "artifact_sha256": Value::Object(artifacts),
        "execution": context.executed,
        "checks_passed": context.summary.get("passed_checks").cloned().unwrap_or(Value::Null),
        "checks_failed": context.summary.get("failed_checks").cloned().unwrap_or(Value::Null),
        "operation_results": context.rows,
        "operation_outcomes": outcomes(&context.rows),
        "request_ledger": ledger,
        "meaning": "Recorded statuses are retained without promoting probe-only, unsupported, decode-failure or HTTP-only evidence. Per-type typed read coverage belongs to the separately asserted capability matrix; no live AWS claim.",
    })
}

fn outcomes(rows: &[Value]) -> BTreeMap<String, i64> {
    let mut outcomes: BTreeMap<String, i64> = BTreeMap::new();
    for row in rows {
        let status = super::files::python_str(row.get("status").unwrap_or(&Value::Null));
        let count = outcomes.entry(status).or_insert(0);
        *count = count.saturating_add(1);
    }
    outcomes
}

fn parent_of(path: &Path) -> Result<&Path, HarnessError> {
    path.parent()
        .ok_or_else(|| HarnessError::invalid("summary has no parent directory"))
}
