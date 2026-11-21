//! Parent scope assertion execution.

use std::path::Path;

use serde_json::Value;

use crate::assertions::helpers::check as add_check;
use crate::HarnessError;

pub async fn check(
    _root: &Path,
    _database: &Path,
    _scan_id: &str,
    seed_report: Option<&Value>,
) -> Result<Vec<Value>, HarnessError> {
    let mut checks = Vec::new();

    // Simple scope checks
    add_check(
        &mut checks,
        "scope:fixtures_present",
        seed_report.is_some(),
        None,
    );

    Ok(checks)
}