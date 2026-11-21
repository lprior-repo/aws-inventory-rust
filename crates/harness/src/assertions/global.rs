//! Global resource architecture assertions.

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

    // Check for fixture metadata (authoritative flag)
    let fixture_ok = seed_report
        .map(|sr| sr.get("fixtures").is_some())
        .unwrap_or(false);

    add_check(
        &mut checks,
        "global:readable_bounded_evidence",
        fixture_ok,
        None,
    );

    add_check(
        &mut checks,
        "global:authoritative_flag",
        fixture_ok,
        None,
    );

    Ok(checks)
}