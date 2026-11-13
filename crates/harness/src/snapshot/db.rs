//! Stable read transaction over one baseline database and its five cases.

mod verify;

use std::collections::BTreeMap;
use std::path::Path;

use serde_json::{Value, json};
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{Connection, Sqlite, SqliteConnection, Transaction};

use super::binding::{Baseline, bound_command};
use super::{CASES, cases, field};
use crate::HarnessError;
use verify::{verify_integrity, verify_persisted, verify_persisted_scans};

/// Observations for one scan, keyed to a fixed private statement.
const RESOURCE_OBSERVATIONS: &str =
    "SELECT resource_json FROM resource_observations WHERE scan_id=? LIMIT ?";
/// Edges for one scan, keyed to a fixed private statement.
const EDGE_OBSERVATIONS: &str = "SELECT edge_json FROM edge_observations WHERE scan_id=? LIMIT ?";

/// Collect all five cases inside one read-only transaction.
pub(super) async fn collect_cases(
    evidence: &Path,
    baseline: &Baseline,
    arguments: &BTreeMap<String, Vec<String>>,
) -> Result<Vec<Value>, HarnessError> {
    let options = SqliteConnectOptions::new()
        .filename(evidence.join("inventory.sqlite"))
        .read_only(true)
        .create_if_missing(false);
    let mut connection = SqliteConnection::connect_with(&options).await?;
    let mut transaction = connection.begin().await?;
    verify_integrity(&mut transaction).await?;
    let (collected, scans) = collect_all(&mut transaction, evidence, baseline, arguments).await?;
    verify_persisted_scans(&mut transaction, scans).await?;
    transaction.commit().await?;
    Ok(collected)
}

async fn collect_all(
    transaction: &mut Transaction<'_, Sqlite>,
    evidence: &Path,
    baseline: &Baseline,
    arguments: &BTreeMap<String, Vec<String>>,
) -> Result<(Vec<Value>, BTreeMap<String, String>), HarnessError> {
    let mut collected = Vec::with_capacity(CASES.len());
    let mut scans = BTreeMap::new();
    for (case_name, command_name) in CASES {
        collected.push(
            collect_case(
                transaction,
                evidence,
                baseline,
                arguments,
                case_name,
                command_name,
                &mut scans,
            )
            .await?,
        );
    }
    Ok((collected, scans))
}

/// Rows read for one baseline case.
struct CaseEvidence {
    resources: Vec<Value>,
    edges: Vec<Value>,
    scopes: Vec<Value>,
    failures: Vec<Value>,
}

async fn collect_case(
    transaction: &mut Transaction<'_, Sqlite>,
    evidence: &Path,
    baseline: &Baseline,
    arguments: &BTreeMap<String, Vec<String>>,
    case_name: &str,
    command_name: &str,
    scans: &mut BTreeMap<String, String>,
) -> Result<Value, HarnessError> {
    let binding = scan_binding(baseline, command_name)?;
    let command = bound_command(evidence, command_name, binding, &baseline.binary_sha)?;
    check_arguments(
        &command,
        arguments,
        binding,
        baseline,
        case_name,
        command_name,
    )?;
    let (scan_id, status) = terminal_scan(case_name, &command)?;
    record_scan(binding, &scan_id, &status, scans)?;
    verify_persisted(&mut *transaction, &scan_id, &status).await?;
    let evidence_rows = collect_rows(transaction, &scan_id).await?;
    case_document(case_name, status, evidence_rows)
}

fn scan_binding<'a>(baseline: &'a Baseline, command_name: &str) -> Result<&'a Value, HarnessError> {
    baseline
        .execution
        .get("scan_commands")
        .and_then(|bindings| bindings.get(command_name))
        .ok_or_else(|| {
            HarnessError::invalid("snapshot baseline must bind exactly the five collector commands")
        })
}

fn check_arguments(
    command: &Value,
    arguments: &BTreeMap<String, Vec<String>>,
    binding: &Value,
    baseline: &Baseline,
    case_name: &str,
    command_name: &str,
) -> Result<(), HarnessError> {
    let declared = to_values(arguments, command_name)?;
    if command.get("arguments").and_then(Value::as_array) != Some(&declared)
        || binding.get("binary_sha256").and_then(Value::as_str) != Some(&baseline.binary_sha)
    {
        return Err(HarnessError::invalid(format!(
            "snapshot case {case_name} has different collection arguments or binary"
        )));
    }
    Ok(())
}

fn record_scan(
    binding: &Value,
    scan_id: &str,
    status: &str,
    scans: &mut BTreeMap<String, String>,
) -> Result<(), HarnessError> {
    if binding.get("scan_id").and_then(Value::as_str) != Some(scan_id)
        || scans
            .insert(scan_id.to_owned(), status.to_owned())
            .is_some()
    {
        return Err(HarnessError::invalid(
            "snapshot cases require five distinct baseline-bound scan identities",
        ));
    }
    Ok(())
}

async fn collect_rows(
    transaction: &mut Transaction<'_, Sqlite>,
    scan_id: &str,
) -> Result<CaseEvidence, HarnessError> {
    Ok(CaseEvidence {
        resources: cases::observations(transaction, RESOURCE_OBSERVATIONS, scan_id).await?,
        edges: cases::observations(transaction, EDGE_OBSERVATIONS, scan_id).await?,
        scopes: cases::scopes(transaction, scan_id).await?,
        failures: cases::failures(transaction, scan_id).await?,
    })
}

fn case_document(
    case_name: &str,
    status: String,
    rows: CaseEvidence,
) -> Result<Value, HarnessError> {
    if rows.resources.is_empty() || rows.scopes.is_empty() {
        return Err(HarnessError::invalid(format!(
            "snapshot case {case_name} lacks populated finalized evidence"
        )));
    }
    Ok(json!({
        "name": case_name,
        "status": status,
        "resources": rows.resources,
        "edges": rows.edges,
        "scopes": rows.scopes,
        "failures": rows.failures,
    }))
}

fn to_values(
    arguments: &BTreeMap<String, Vec<String>>,
    command_name: &str,
) -> Result<Vec<Value>, HarnessError> {
    arguments
        .get(command_name)
        .map(|values| values.iter().cloned().map(Value::from).collect())
        .ok_or_else(|| HarnessError::invalid("snapshot case has no recorded arguments"))
}

fn terminal_scan(case_name: &str, command: &Value) -> Result<(String, String), HarnessError> {
    let result = field(command, "result")?;
    let (scan, exit) = single_scan(case_name, result, command)?;
    let status = terminal_status(case_name, scan)?;
    check_terminal_consistency(case_name, result, scan, exit, &status)?;
    Ok((scan_identity(scan)?, status))
}

fn single_scan<'a>(
    case_name: &str,
    result: &'a Value,
    command: &Value,
) -> Result<(&'a Value, Option<i64>), HarnessError> {
    let scans = result
        .get("scans")
        .and_then(Value::as_array)
        .ok_or_else(|| no_scan(case_name))?;
    let exit = command.get("exit_code").and_then(Value::as_i64);
    if scans.len() != 1 || !matches!(exit, Some(0) | Some(2)) {
        return Err(no_scan(case_name));
    }
    let scan = scans.first().ok_or_else(|| no_scan(case_name))?;
    Ok((scan, exit))
}

fn terminal_status(case_name: &str, scan: &Value) -> Result<String, HarnessError> {
    let status = scan
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned();
    if status != "complete" && status != "partial" {
        return Err(HarnessError::invalid(format!(
            "snapshot case {case_name} is not terminal"
        )));
    }
    Ok(status)
}

fn check_terminal_consistency(
    case_name: &str,
    result: &Value,
    scan: &Value,
    exit: Option<i64>,
    status: &str,
) -> Result<(), HarnessError> {
    let consistent = result.get("status").and_then(Value::as_str) == Some(status)
        && result.get("write_error").is_none_or(Value::is_null)
        && scan.get("error").is_none_or(Value::is_null)
        && exit == Some(if status == "complete" { 0 } else { 2 });
    if !consistent {
        return Err(HarnessError::invalid(format!(
            "snapshot case {case_name} has inconsistent terminal command evidence"
        )));
    }
    Ok(())
}

fn scan_identity(scan: &Value) -> Result<String, HarnessError> {
    scan.get("scan_id")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| {
            HarnessError::invalid(
                "snapshot cases require five distinct baseline-bound scan identities",
            )
        })
}

fn no_scan(case_name: &str) -> HarnessError {
    HarnessError::invalid(format!(
        "snapshot case {case_name} has no single terminal scan"
    ))
}
