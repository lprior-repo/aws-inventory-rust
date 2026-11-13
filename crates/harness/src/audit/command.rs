//! Publish a reconciliation receipt and refuse historical replacements.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::HarnessError;
use crate::provenance::{file_sha256, source_fingerprint};

use super::bind::bind_emulator;
use super::files::{MAX_DOCUMENT_BYTES, relative_slash, resolve_target};
use super::reconcile::reconcile;

/// Historical evidence files that must never be rewritten.
const HISTORICAL: [&str; 4] = [
    "catalog/aws-operations.json",
    "artifacts/aws-operation-source-reconciliation-v04.json",
    "artifacts/aws-operation-source-reconciliation-v05.json",
    "catalog/aws-operation-additions.json",
];

/// Refuse outputs that would replace historical evidence.
pub fn output_path(root: &Path, requested: &Path) -> Result<PathBuf, HarnessError> {
    let output = resolve_target(requested)?;
    let base = fs::canonicalize(root)?;
    if output.strip_prefix(&base).is_err() {
        return Err(HarnessError::invalid(
            "output path is outside the workspace root",
        ));
    }
    for name in HISTORICAL {
        if resolve_target(&base.join(name))? == output {
            return Err(HarnessError::invalid(
                "historical evidence cannot be replaced",
            ));
        }
    }
    Ok(output)
}

/// Bind the emulator summary and auditor provenance onto the report.
fn reconciled_report(root: &Path, emulator_summary: Option<&Path>) -> Result<Value, HarnessError> {
    let mut report = reconcile(root)?;
    if let Some(summary) = emulator_summary {
        let bound = bind_emulator(&report, root, summary)?;
        if let Some(object) = report.as_object_mut() {
            object.insert("emulator_evidence".to_owned(), bound);
        }
    }
    let auditor = root.join("scripts/catalog/reconcile_aws_operations.py");
    let fixture = root.join("scripts/emulator/fixture_provenance.py");
    let record = serde_json::json!({
        "path": relative_slash(root, &auditor)?,
        "sha256": file_sha256(&auditor)?,
        "fixture_provenance_sha256": file_sha256(&fixture)?,
    });
    if let Some(object) = report.as_object_mut() {
        object.insert("auditor".to_owned(), record);
    }
    Ok(report)
}

/// Encode the report under its byte budget and rename it into place.
fn publish(report: &Value, output: &Path) -> Result<(), HarnessError> {
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }
    let file_name = output
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .ok_or_else(|| HarnessError::invalid("output path has no file name"))?;
    let temporary = output.with_file_name(format!("{file_name}.partial"));
    let mut encoded = serde_json::to_string_pretty(report)?;
    encoded.push('\n');
    if encoded.len() > MAX_DOCUMENT_BYTES {
        return Err(HarnessError::invalid(
            "reconciliation output byte budget exceeded",
        ));
    }
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)?;
    file.write_all(encoded.as_bytes())?;
    file.flush()?;
    fs::rename(&temporary, output)?;
    Ok(())
}

/// The summary line with its published path and status attached.
fn summary_line(root: &Path, report: &Value, output: &Path) -> Result<Value, HarnessError> {
    let mut line = report.get("summary").cloned().unwrap_or(Value::Null);
    if let Some(object) = line.as_object_mut() {
        object.insert(
            "output".to_owned(),
            Value::String(relative_slash(root, output)?),
        );
        object.insert(
            "status".to_owned(),
            report
                .get("status")
                .cloned()
                .unwrap_or_else(|| Value::String("unknown".to_owned())),
        );
    }
    Ok(line)
}

/// Run the reconciliation, bind an optional emulator summary and publish it.
///
/// Returns the receipt and the exact summary line the Python command prints.
pub fn run_reconcile(
    root: &Path,
    output_requested: &Path,
    emulator_summary: Option<&Path>,
) -> Result<(Value, String), HarnessError> {
    let report = reconciled_report(root, emulator_summary)?;
    if report.get("source") != Some(&source_fingerprint(root)?) {
        return Err(HarnessError::invalid(
            "source changed while operation reconciliation was running",
        ));
    }
    let output = output_path(root, output_requested)?;
    publish(&report, &output)?;
    let line = summary_line(root, &report, &output)?;
    Ok((report, super::dumps_default(&line)))
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::reconcile;
    use crate::audit::run_refresh;

    fn parity_report(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut report = reconcile(root)?;
        if let Some(object) = report.as_object_mut() {
            object.remove("observed_at");
        }
        println!("{}", serde_json::to_string_pretty(&report)?);
        Ok(())
    }

    fn parity_refresh(root: &str) -> Result<(), Box<dyn std::error::Error>> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        let (_, line) = runtime.block_on(run_refresh(Path::new(root), None))?;
        println!("{line}");
        Ok(())
    }

    fn parity_run(root: &str) -> Result<(), Box<dyn std::error::Error>> {
        let output = Path::new(root).join("artifacts/aws-operation-source-reconciliation-v06.json");
        let (_, line) = super::run_reconcile(Path::new(root), &output, None)?;
        println!("{line}");
        Ok(())
    }

    #[test]
    #[ignore = "manual parity driver: set AUDIT_PARITY_ROOT and AUDIT_PARITY_MODE"]
    fn parity_dump() -> Result<(), Box<dyn std::error::Error>> {
        let root = std::env::var("AUDIT_PARITY_ROOT")?;
        match std::env::var("AUDIT_PARITY_MODE").as_deref() {
            Ok("refresh") => parity_refresh(&root),
            Ok("run") => parity_run(&root),
            _ => parity_report(Path::new(&root)),
        }
    }
}
