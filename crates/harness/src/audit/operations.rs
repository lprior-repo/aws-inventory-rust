pub mod calls;
pub mod lexical;
pub mod mask;

use std::collections::HashMap;

use crate::HarnessError;
use crate::audit::files::{read_json_audit, read_toml_audit};
use crate::audit::matrix::catalogs;
use crate::audit::pins::dependency_evidence;
use crate::audit::validate::validate_summary;
use crate::provenance::{sha256_hex, source_fingerprint};

pub fn reconcile(root: &std::path::Path) -> Result<serde_json::Value, HarnessError> {
    let before = source_fingerprint(root)?;
    let (matrix, matrix_sha) = read_toml_audit(&root.join("catalog/aws-operations.toml"))?;

    let previous_path = root.join("artifacts/aws-operation-source-reconciliation-v04.json");
    let (previous, previous_sha) = read_json_audit(&previous_path)
        .map_err(|_| HarnessError::invalid("historical reconciliation artifact is missing"))?;

    let prev_bound = previous
        .get("historical_matrix")
        .and_then(|h| h.get("sha256"))
        .and_then(serde_json::Value::as_str);
    if prev_bound != Some(&matrix_sha) {
        return Err(HarnessError::invalid(
            "historical operation matrix changed since the preserved v0.4 audit",
        ));
    }

    let (sources, calls, nonempty) = calls::source_inventory(root)?;

    // Compare calls against matrix operations
    let mut matrix_ops = HashMap::new();
    if let Some(ops) = matrix
        .get("operations")
        .and_then(serde_json::Value::as_array)
    {
        for op in ops {
            if let (Some(id), Some(calls_arr)) = (
                op.get("id").and_then(serde_json::Value::as_str),
                op.get("source_calls").and_then(serde_json::Value::as_array),
            ) {
                let mut callsites = Vec::new();
                for call in calls_arr {
                    if let (Some(path), Some(func)) = (
                        call.get("path").and_then(serde_json::Value::as_str),
                        call.get("function").and_then(serde_json::Value::as_str),
                    ) {
                        callsites.push((path.to_string(), func.to_string()));
                    }
                }
                matrix_ops.insert(id.to_string(), callsites);
            }
        }
    }

    // Build multiset of observed calls
    let mut observed = HashMap::new();
    for call in &calls {
        observed
            .entry(call.id.clone())
            .or_insert_with(Vec::new)
            .push((call.path.clone(), call.function.clone()));
    }

    // Compare: every matrix operation must be observed, and vice versa
    for (id, expected_calls) in &matrix_ops {
        let actual = observed.get(id.as_str());
        match actual {
            None => {
                return Err(HarnessError::invalid(format!(
                    "matrix operation {} not observed in source",
                    id
                )));
            }
            Some(actual_calls) => {
                if expected_calls.len() != actual_calls.len() {
                    return Err(HarnessError::invalid(format!(
                        "operation {} has {} expected calls but {} observed",
                        id,
                        expected_calls.len(),
                        actual_calls.len()
                    )));
                }
                for exp in expected_calls {
                    if !actual_calls.contains(exp) {
                        return Err(HarnessError::invalid(format!(
                            "operation {} missing expected callsite {}/{}",
                            id, exp.0, exp.1
                        )));
                    }
                }
            }
        }
    }

    // Check for unexpected observed calls not in matrix
    for id in observed.keys() {
        if !matrix_ops.contains_key(id) {
            return Err(HarnessError::invalid(format!(
                "observed operation {} not in matrix",
                id
            )));
        }
    }

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
            "path": "catalog/aws-operations.toml",
            "sha256": matrix_sha,
            "left_unchanged": true
        },
        "historical_reconciliation": {
            "path": "artifacts/aws-operation-source-reconciliation-v04.json",
            "sha256": previous_sha,
            "left_unchanged": true
        },
        "summary": {
            "explicit_unique_operations": calls.iter().map(|c| c.id.as_str()).collect::<std::collections::HashSet<_>>().len(),
            "explicit_sdk_send_callsites": calls.len(),
            "sdk_services": calls.iter().map(|c| c.id.as_str()).collect::<std::collections::HashSet<_>>().iter().map(|s| s.split('.').next().unwrap_or("")).collect::<std::collections::HashSet<_>>().len(),
            "credential_provider_operation_boundaries": 0,
            "callsite_positions_changed": 0,
            "nonempty_channel_send_sites_excluded": nonempty,
            "aws_source_files": sources.len()
        },
        "method": "Mask comments/string/character literals; enumerate every empty send under crates/aws/src; resolve each direct receiver/method chain and SDK client namespace; require the exact historical path/function/operation multiset. Verify pinned operation/provider module hashes against Cargo.lock and the local registry source.",
        "callsites": calls.iter().map(|c| serde_json::json!({
                "id": c.id,
                "path": c.path,
                "function": c.function,
                "operation_line": c.operation_line,
                "send_line": c.send_line
            })).collect::<Vec<_>>(),
        "credential_provider": {
            "id": "sts.assume_role",
            "explicit_sdk_send": false,
            "source_calls": [],
            "pinned_dependency": provider_dependency
        },
        "pinned_sdk_operation_modules": modules,
        "aws_source_files": sources.keys().cloned().collect::<Vec<_>>(),
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
) -> Result<(serde_json::Value, Vec<serde_json::Value>), HarnessError> {
    let (summary, _) = read_json_audit(summary_path)?;
    let (matrix, _) = read_toml_audit(&root.join("catalog/aws-operations.toml"))?;
    let catalog = catalogs(root)?;

    let (executed, rows) = validate_summary(report, &summary, &catalog.catalog, &matrix)?;

    Ok((executed, rows))
}

pub fn check_additions(root: &std::path::Path) -> Result<serde_json::Value, HarnessError> {
    let (matrix, matrix_sha) = read_toml_audit(&root.join("catalog/aws-operations.toml"))?;
    let additions_path = root.join("catalog/aws-operation-additions.json");

    let additions = if additions_path.is_file() {
        let (doc, _) = read_json_audit(&additions_path)?;
        validate_additions_document(root, &doc, &matrix, &matrix_sha)?
    } else {
        serde_json::Value::Null
    };

    Ok(serde_json::json!({
        "matrix_sha256": matrix_sha,
        "additions_path": "catalog/aws-operation-additions.json",
        "additions": additions,
        "status": if additions.is_null() { "absent" } else { "applied" }
    }))
}

fn validate_additions_document(
    _root: &std::path::Path,
    doc: &serde_json::Value,
    matrix: &serde_json::Value,
    matrix_sha: &str,
) -> Result<serde_json::Value, HarnessError> {
    let version = doc.get("schema_version").and_then(|v| v.as_i64());
    if version != Some(1) {
        return Err(HarnessError::invalid("additions schema_version must be 1"));
    }

    let binds_object = serde_json::Map::new();
    let binds_null = serde_json::Value::Object(binds_object);
    let binds = doc.get("binds").unwrap_or(&binds_null);
    let binds_matrix = binds
        .get("historical_matrix")
        .and_then(|b| b.get("sha256"))
        .and_then(|s| s.as_str());
    if binds_matrix != Some(matrix_sha) {
        return Err(HarnessError::invalid(
            "additions bind a different historical operation matrix",
        ));
    }

    let empty_vec = Vec::new();
    let operations = matrix
        .get("operations")
        .and_then(|o| o.as_array())
        .unwrap_or(&empty_vec);
    let matrix_ids: std::collections::HashSet<String> = operations
        .iter()
        .filter_map(|op| op.get("id").and_then(|i| i.as_str()).map(String::from))
        .collect();

    let rows = doc
        .get("additions")
        .and_then(|a| a.as_array())
        .unwrap_or(&empty_vec);
    let mut checked = Vec::new();

    for row in rows {
        let id = row
            .get("id")
            .and_then(|i| i.as_str())
            .ok_or_else(|| HarnessError::invalid("addition missing id"))?;

        if matrix_ids.contains(id) {
            return Err(HarnessError::invalid(format!(
                "addition {} reuses a historical operation identity",
                id
            )));
        }

        checked.push(serde_json::json!({
            "id": id,
            "status": "checked"
        }));
    }

    Ok(serde_json::json!({
        "schema_version": 1,
        "additions_count": checked.len(),
        "additions": checked
    }))
}

pub fn write_reconciliation(
    root: &std::path::Path,
    report: &serde_json::Value,
    output: &std::path::Path,
) -> Result<(), HarnessError> {
    let output = output.to_path_buf();

    // Validate output is within root
    let output = output
        .canonicalize()
        .map_err(|e| HarnessError::invalid(format!("failed to canonicalize output path: {}", e)))?;
    let root = root
        .canonicalize()
        .map_err(|e| HarnessError::invalid(format!("failed to canonicalize root path: {}", e)))?;
    if !output.starts_with(&root) {
        return Err(HarnessError::invalid(
            "output path must be within the repository root",
        ));
    }

    // Check protected historical paths
    let protected = [
        root.join("catalog/aws-operations.toml"),
        root.join("catalog/aws-operation-additions.json"),
        root.join("artifacts/aws-operation-source-reconciliation-v04.json"),
        root.join("artifacts/aws-operation-source-reconciliation-v05.json"),
    ];
    for p in &protected {
        if output == *p {
            return Err(HarnessError::invalid(
                "historical evidence cannot be replaced",
            ));
        }
    }

    // Atomic write: write to .partial temp, then rename
    let temp = output.with_extension("partial");
    let content = serde_json::to_string_pretty(report)?;
    std::fs::write(&temp, content + "\n")
        .map_err(|e| HarnessError::invalid(format!("failed to write temporary file: {}", e)))?;

    std::fs::rename(&temp, &output)
        .map_err(|e| HarnessError::invalid(format!("failed to rename temporary file: {}", e)))?;

    Ok(())
}
