//! Baseline summary validation: assertion completeness, identities and roles.

use std::collections::BTreeSet;
use std::path::Path;

use serde_json::Value;

use super::read_json;
use crate::HarnessError;

/// Assertions that must be present for a baseline to become an oracle.
const REQUIRED_CHECKS: [&str; 22] = [
    "endpoint:emulator_identity",
    "native:terminal_scan",
    "global:all_32_native_kinds",
    "sqlite:integrity",
    "sqlite:all_scans_finalized",
    "coverage:every_operation_attempted",
    "coverage:no_unexplained_operation_failures",
    "coverage:every_registered_type_read",
    "provenance:embedded_catalog_matches",
    "provenance:source_unchanged",
    "provenance:binary_unchanged",
    "provenance:fixture_unchanged",
    "provenance:parent_fixture_unchanged",
    "provenance:scan_commands_bound",
    "parents:fixture_hash",
    "parents:seed_source_hash",
    "parents:parent_catalog_hash",
    "parents:seed_complete",
    "parents:exact_child_count",
    "parents:exact_parent_edge_count",
    "parents:all_twenty_four_parent_discoveries_complete",
    "parents:no_collection_failures",
];

/// Reject any baseline that is not complete, independently asserted and pinned.
pub fn validate(summary: &Value) -> Result<(), HarnessError> {
    check_assertion_counts(summary)?;
    check_checks(summary)?;
    check_emulator(summary)
}

fn check_assertion_counts(summary: &Value) -> Result<(), HarnessError> {
    let incomplete = || {
        HarnessError::invalid(
            "snapshot requires the complete independently asserted fixture baseline",
        )
    };
    if summary.get("failed_checks").and_then(Value::as_u64) != Some(0) {
        return Err(incomplete());
    }
    if summary
        .get("passed_checks")
        .and_then(Value::as_u64)
        .is_none_or(|passed| passed < 1_000)
    {
        return Err(incomplete());
    }
    Ok(())
}

fn check_checks(summary: &Value) -> Result<(), HarnessError> {
    let checks = checks(summary)?;
    check_all_passed(checks)?;
    check_accounting(summary, checks)?;
    check_required_names(checks)
}

fn checks(summary: &Value) -> Result<&Vec<Value>, HarnessError> {
    summary
        .get("checks")
        .and_then(Value::as_array)
        .filter(|checks| !checks.is_empty())
        .ok_or_else(|| {
            HarnessError::invalid("snapshot baseline contains missing or unsuccessful assertions")
        })
}

fn check_all_passed(checks: &[Value]) -> Result<(), HarnessError> {
    if checks
        .iter()
        .any(|check| check.get("passed") != Some(&Value::Bool(true)))
    {
        return Err(HarnessError::invalid(
            "snapshot baseline contains missing or unsuccessful assertions",
        ));
    }
    Ok(())
}

fn check_accounting(summary: &Value, checks: &[Value]) -> Result<(), HarnessError> {
    let passed = summary.get("passed_checks").and_then(Value::as_u64);
    if passed != u64::try_from(checks.len()).ok() {
        return Err(HarnessError::invalid(
            "snapshot baseline assertion accounting is inconsistent",
        ));
    }
    Ok(())
}

fn check_required_names(checks: &[Value]) -> Result<(), HarnessError> {
    let names: BTreeSet<&str> = checks
        .iter()
        .filter_map(|check| check.get("check").and_then(Value::as_str))
        .collect();
    if REQUIRED_CHECKS
        .iter()
        .any(|required| !names.contains(required))
    {
        return Err(HarnessError::invalid(
            "snapshot baseline lacks required independent or provenance assertions",
        ));
    }
    Ok(())
}

fn check_emulator(summary: &Value) -> Result<(), HarnessError> {
    let emulator = summary
        .get("emulator")
        .filter(|value| value.is_object())
        .ok_or_else(|| HarnessError::invalid("snapshot baseline lacks emulator identity"))?;
    let version = emulator
        .get("version_output")
        .and_then(Value::as_str)
        .ok_or_else(|| HarnessError::invalid("snapshot baseline lacks emulator version"))?;
    if identity(emulator) != expected_identity()
        || !version.contains(crate::emulator::PINNED_VERSION)
    {
        return Err(HarnessError::invalid(
            "snapshot baseline emulator identity does not match the pinned fixture",
        ));
    }
    Ok(())
}

fn identity(emulator: &Value) -> (Option<&str>, Option<&str>, Option<&str>) {
    (
        emulator.get("account").and_then(Value::as_str),
        emulator.get("region").and_then(Value::as_str),
        emulator.get("source_commit").and_then(Value::as_str),
    )
}

fn expected_identity() -> (
    Option<&'static str>,
    Option<&'static str>,
    Option<&'static str>,
) {
    (
        Some("000000000000"),
        Some("us-east-1"),
        Some(crate::emulator::PINNED_COMMIT),
    )
}

/// Read the synthetic assume-role identity the baseline seeded.
pub fn assume_role(evidence: &Path) -> Result<String, HarnessError> {
    let (seed, _) = read_json(&evidence.join("seed-results.json"))?;
    let role = seed
        .get("modules")
        .and_then(|modules| modules.get("seed_discovery"))
        .and_then(|seed| seed.get("fixtures"))
        .and_then(|fixtures| fixtures.get("assume_role_arn"))
        .and_then(Value::as_str)
        .ok_or_else(|| HarnessError::invalid("snapshot assume-role fixture is missing"))?
        .to_owned();
    if !role.starts_with("arn:aws:iam::000000000000:role/") {
        return Err(HarnessError::invalid(
            "snapshot assume-role fixture belongs to a different account",
        ));
    }
    Ok(role)
}
