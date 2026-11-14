use std::path::Path;

use serde_json::Value;

use crate::HarnessError;

use super::Run;

pub(super) fn scan(run: &Run) -> Result<&Value, HarnessError> {
    run.scans
        .first()
        .ok_or_else(|| HarnessError::invalid("missing coverage scan"))
}

pub(super) fn parent_of(path: &Path) -> Result<&Path, HarnessError> {
    path.parent()
        .ok_or_else(|| HarnessError::invalid("summary has no parent directory"))
}

pub(super) fn expected_status(scan: &Value) -> Result<Option<&str>, HarnessError> {
    match scan.get("status") {
        None => Ok(None),
        Some(Value::String(value)) => Ok(Some(value.as_str())),
        Some(_) => Err(HarnessError::invalid(
            "persisted coverage scan status differs from the command receipt",
        )),
    }
}
