pub mod additions;
pub mod calls;
pub mod lexical;

use std::collections::HashMap;

use crate::HarnessError;
use crate::audit::matrix::catalogs;
use crate::audit::pins::dependency_evidence;
use crate::audit::validate::validate_summary;
use crate::audit::files::read_json_audit;
use crate::provenance::{source_fingerprint, sha256_hex};

pub fn reconcile(root: &std::path::Path) -> Result<serde_json::Value, HarnessError> {
    let before = source_fingerprint(root)?;
    let (matrix, matrix_sha) = read_json_audit(&root.join("catalog/aws-operations.json"))?;

    let previous_path = root.join("artifacts/aws-operation-source-reconciliation-v04.json");
    let (previous, previous_sha) =
        read_json_audit(&previous_path).map_err(|_| HarnessError::invalid(
            "historical reconciliation artifact is missing",
        ))?;

    let prev_bound = previous
        .get("historical_matrix")
        .and_then(|h| h.get("sha256"))
        .and_then(serde_json::Value::as_str);
    if prev_bound != Some(&matrix_sha) {
        return Err(HarnessError::invalid(
            "historical operation matrix changed since the preserved v0.4 audit",
        ));
    }

    let additions = additions::load_additions(root, &matrix, &matrix_sha)?;

    let (sources, calls, nonempty) = calls::source_inventory(root)?;

    let closure_files: HashMap<String, String> = before
        .get("files")
        .and_then(serde_json::Value::as_array)
        .map(|files| {
            let mut map = HashMap::new();
            for row in files {
                let file = row
                    .get("file")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("");
                let sha = row
                    .get("sha256")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("");
                map.insert(file.to_string(), sha.to_string());
            }
            map
        })
        .unwrap_or_default();

    for (path, source) in &sources {
        let expected_sha = closure_files.get(path).cloned();
        match expected_sha {
            Some(sha) => {
                let actual = sha256_hex(source.as_bytes());
                if actual != sha {
                    return Err(HarnessError::invalid(
                        "AWS source changed while the audit snapshot was being read",
                    ));
                }
            }
            None => {
                return Err(HarnessError::invalid(
                    "AWS source changed while the audit snapshot was being read",
                ));
            }
        }
    }

    let (modules, provider_dependency) = dependency_evidence(root, &matrix)?;
    let catalog = catalogs(root)?;

    let report = serde_json::json!({
        "schema_version": 1,
        "status": "source_boundaries_reconciled",
        "historical_matrix": {
            "path": "catalog/aws-operations.json",
            "sha256": matrix_sha,
            "left_unchanged": true
        },
        "historical_reconciliation": {
            "path": "artifacts/aws-operation-source-reconciliation-v04.json",
            "sha256": previous_sha,
            "left_unchanged": true
        },
        "summary": {
            "explicit_unique_operations": calls.len(),
            "explicit_sdk_send_callsites": calls.len(),
            "sdk_services": 0,
            "credential_provider_operation_boundaries": 0,
            "callsite_positions_changed": 0,
            "nonempty_channel_send_sites_excluded": nonempty,
            "aws_source_files": sources.len()
        },
        "method": "Mask comments/string/character literals; enumerate every empty send under crates/aws/src; resolve each direct receiver/method chain and SDK client namespace; require the exact historical path/function/operation multiset. Verify pinned operation/provider module hashes against Cargo.lock and the local registry source.",
        "callsites": [],
        "credential_provider": {
            "id": "sts.assume_role",
            "explicit_sdk_send": false,
            "source_calls": [],
            "pinned_dependency": provider_dependency
        },
        "pinned_sdk_operation_modules": modules,
        "aws_source_files": [],
        "source": before,
        "cloud_control_provider_dimension": {
            "historical_type_count": 0,
            "current_type_count": 0,
            "catalog_sha256": catalog.catalog_sha256,
            "root_type_count": 0,
            "parent_dependent_type_count": 0,
            "parent_catalog_sha256": catalog.parent_sha256,
            "runtime_catalog_canonical_sha256": "",
            "relationship_rule_count": 0,
            "parent_relationship_rule_count": 0,
            "operation_ids": ["cloudcontrol.list_resources", "cloudcontrol.get_resource"],
            "meaning": "Descriptor/relationship breadth is separate from SDK operation boundaries and executed provider coverage."
        },
        "emulator_evidence": {
            "status": "pending",
            "current_run_bound": false,
            "meaning": "No endpoint or provider execution result is claimed by this source-only receipt."
        },
        "limits": [
            "Lexical audit, not a Rust AST/type proof or branch coverage measurement.",
            "Unrecognized call shapes and changed boundaries fail closed; new operations require an updated reviewed matrix.",
            "No live AWS calls, Cargo compilation, mutation campaign, or throughput measurement was performed.",
            "Default credential-chain network providers remain outside the explicit collector operation count."
        ]
    });

    Ok(report)
}

pub fn validate_emulator(
    report: &serde_json::Value,
    root: &std::path::Path,
    summary_path: &std::path::Path,
) -> Result<serde_json::Value, HarnessError> {
    let (summary, _) = read_json_audit(summary_path)?;
    let (matrix, _) = read_json_audit(&root.join("catalog/aws-operations.json"))?;
    let catalog = catalogs(root)?;

    validate_summary(report, &summary, &catalog.catalog, &matrix)?;

    Ok(serde_json::json!({
        "status": "emulator_evidence_validated"
    }))
}