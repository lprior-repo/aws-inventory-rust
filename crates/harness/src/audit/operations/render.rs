//! Render and publish the operation coverage report.

mod additions;
mod report;

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use crate::HarnessError;

use super::super::files::{resolve_target, slash_path};
use super::{CoverageReport, DOCUMENT, OPERATION_ADDITIONS, document};

fn percent(part: usize, total: usize) -> Result<String, HarnessError> {
    if total == 0 {
        return Ok("0.00%".to_owned());
    }
    let part = u128::try_from(part)
        .map_err(|_| HarnessError::invalid("coverage count is not representable"))?;
    let total = u128::try_from(total)
        .map_err(|_| HarnessError::invalid("coverage total is not representable"))?;
    let basis = part
        .checked_mul(10_000)
        .ok_or_else(|| HarnessError::invalid("coverage ratio overflowed"))?
        .checked_div(total)
        .ok_or_else(|| HarnessError::invalid("coverage ratio divided by zero"))?;
    let whole = basis
        .checked_div(100)
        .ok_or_else(|| HarnessError::invalid("coverage ratio overflowed"))?;
    let fraction = basis
        .checked_rem(100)
        .ok_or_else(|| HarnessError::invalid("coverage ratio overflowed"))?;
    Ok(format!("{whole}.{fraction:02}%"))
}

fn first_difference_line(expected: &str, actual: &[u8]) -> usize {
    let expected_bytes = expected.as_bytes();
    let mut index = 0_usize;
    loop {
        match (expected_bytes.get(index), actual.get(index)) {
            (Some(left), Some(right)) if left == right => {
                index = index.saturating_add(1);
            }
            _ => break,
        }
    }
    let prefix = expected_bytes.get(..index).unwrap_or_default();
    1_usize.saturating_add(prefix.iter().filter(|byte| **byte == b'\n').count())
}

fn verify(root: &Path, expected: &str) -> Result<(), HarnessError> {
    let committed = fs::read(root.join(DOCUMENT))
        .map_err(|error| HarnessError::invalid(format!("{DOCUMENT}: {error}")))?;
    if committed == expected.as_bytes() {
        return Ok(());
    }
    Err(HarnessError::invalid(format!(
        "operation coverage document differs; regenerate with `inventory-harness coverage \
         operations --write {DOCUMENT}` (first difference at line {})",
        first_difference_line(expected, &committed)
    )))
}

fn relative_text(root: &Path, path: &Path) -> Result<String, HarnessError> {
    let base = fs::canonicalize(root)?;
    let relative = path
        .strip_prefix(&base)
        .map_err(|_| HarnessError::invalid("coverage document escaped the workspace root"))?;
    Ok(slash_path(relative))
}

/// Without `write` or `check` the full JSON report is returned; `check`
/// requires byte-identical Markdown.
pub fn run(root: &Path, write: Option<&Path>, check: bool) -> Result<Value, HarnessError> {
    guard_flags(write, check)?;
    let report = document(root)?;
    ensure_scannable(&report, check, write)?;
    let rendered = format!("{}\n", report::render(&report)?);
    let unexercised = report.unexercised()?;
    if check {
        verify(root, &rendered)?;
        return Ok(receipt(&report, DOCUMENT.to_owned(), true, unexercised));
    }
    let Some(requested) = write else {
        return report.to_value();
    };
    let output = write_document(root, requested, &rendered)?;
    Ok(receipt(
        &report,
        relative_text(root, &output)?,
        false,
        unexercised,
    ))
}

fn guard_flags(write: Option<&Path>, check: bool) -> Result<(), HarnessError> {
    if check && write.is_some() {
        return Err(HarnessError::invalid(
            "--check and --write are mutually exclusive",
        ));
    }
    Ok(())
}

fn ensure_scannable(
    report: &CoverageReport,
    check: bool,
    write: Option<&Path>,
) -> Result<(), HarnessError> {
    if (check || write.is_some()) && report.scan_status != "verified" {
        return Err(HarnessError::invalid(format!(
            "operation coverage exercised count is unavailable: the callsite scan over \
`crates/aws/src` did not complete ({}) and no digest-bound receipt verifies the reviewed source; \
refusing to write or check the coverage document until callsite scanning is repaired",
            report.scan_status
        )));
    }
    Ok(())
}

fn receipt(report: &CoverageReport, file: String, check: bool, unexercised: usize) -> Value {
    let mut receipt = json!({
        "file": file,
        "operations_total": report.operations(),
        "exercised": report.exercised(),
        "unexercised": unexercised,
        "check": check,
    });
    if report.reviewed_additions() > 0
        && let Some(object) = receipt.as_object_mut()
    {
        object.insert(
            "reviewed_additions".to_owned(),
            json!(report.reviewed_additions()),
        );
    }
    receipt
}

fn guard_additions(root: &Path, output: &Path) -> Result<(), HarnessError> {
    if resolve_target(&root.join(OPERATION_ADDITIONS))? == output {
        return Err(HarnessError::invalid(format!(
            "{OPERATION_ADDITIONS} cannot be replaced by coverage output"
        )));
    }
    Ok(())
}

fn write_document(root: &Path, requested: &Path, rendered: &str) -> Result<PathBuf, HarnessError> {
    let target: PathBuf = if requested.is_absolute() {
        requested.to_path_buf()
    } else {
        root.join(requested)
    };
    let output = resolve_target(&target)?;
    guard_additions(root, &output)?;
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&output, rendered.as_bytes())?;
    Ok(output)
}
