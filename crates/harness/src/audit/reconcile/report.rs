//! JSON receipt assembly for the reconciliation report.

use std::path::Path;

use serde_json::{Map, Value, json};

use super::super::format::iso8601_now;
use crate::HarnessError;

use super::super::operations::OPERATION_ADDITIONS;
use super::{CatalogFacts, Facts, Inputs};

const METHOD: &str = "Mask comments/string/character literals; enumerate every empty send under crates/aws/src; resolve each direct receiver/method chain and SDK client namespace; require the exact historical path/function/operation multiset. Verify pinned operation/provider module hashes against Cargo.lock and the local registry source.";

pub(super) struct Report<'a> {
    pub(super) before: &'a Value,
    pub(super) inputs: &'a Inputs,
    pub(super) catalogs: &'a CatalogFacts,
    pub(super) facts: &'a Facts,
}

pub(super) fn report(root: &Path, data: Report<'_>) -> Result<Value, HarnessError> {
    let Report {
        before,
        inputs,
        catalogs,
        facts,
    } = data;
    let mut fields = header(root, inputs, facts)?;
    add_tail(&mut fields, before, inputs, catalogs, facts);
    Ok(Value::Object(fields))
}

fn header(root: &Path, inputs: &Inputs, facts: &Facts) -> Result<Map<String, Value>, HarnessError> {
    let mut fields = Map::new();
    fields.insert("schema_version".to_owned(), json!(1));
    fields.insert("observed_at".to_owned(), json!(iso8601_now()?));
    fields.insert("status".to_owned(), json!("source_boundaries_reconciled"));
    fields.insert("sdk_operation_boundaries_added".to_owned(), json!(0));
    fields.insert("sdk_operation_boundaries_removed".to_owned(), json!(0));
    fields.insert(
        "historical_matrix".to_owned(),
        historical_matrix(root, inputs)?,
    );
    fields.insert(
        "historical_reconciliation".to_owned(),
        historical_reconciliation(root, inputs)?,
    );
    if inputs.additions.is_present() {
        fields.insert(
            "reviewed_additions".to_owned(),
            reviewed_additions(inputs, facts),
        );
    }
    fields.insert("summary".to_owned(), summary(inputs, facts));
    fields.insert("method".to_owned(), json!(method(inputs)));
    Ok(fields)
}

fn add_tail(
    fields: &mut Map<String, Value>,
    before: &Value,
    inputs: &Inputs,
    catalogs: &CatalogFacts,
    facts: &Facts,
) {
    digest_tail(fields, facts);
    dimension_tail(fields, before, inputs, catalogs);
}

fn digest_tail(fields: &mut Map<String, Value>, facts: &Facts) {
    fields.insert(
        "callsites".to_owned(),
        Value::Array(facts.callsites.clone()),
    );
    fields.insert("credential_provider".to_owned(), credential_provider(facts));
    fields.insert(
        "pinned_sdk_operation_modules".to_owned(),
        Value::Array(facts.modules.clone()),
    );
    fields.insert(
        "aws_source_files".to_owned(),
        Value::Array(facts.rows.clone()),
    );
}

fn dimension_tail(
    fields: &mut Map<String, Value>,
    before: &Value,
    inputs: &Inputs,
    catalogs: &CatalogFacts,
) {
    fields.insert("source".to_owned(), before.clone());
    fields.insert(
        "cloud_control_provider_dimension".to_owned(),
        provider_dimension(inputs, catalogs),
    );
    fields.insert("emulator_evidence".to_owned(), emulator_evidence());
    fields.insert("limits".to_owned(), limits());
}

fn historical_matrix(root: &Path, inputs: &Inputs) -> Result<Value, HarnessError> {
    Ok(json!({
        "path": relative_text(root, &root.join("catalog/aws-operations.json"))?,
        "sha256": inputs.matrix_sha,
        "left_unchanged": true,
    }))
}

fn historical_reconciliation(root: &Path, inputs: &Inputs) -> Result<Value, HarnessError> {
    Ok(json!({
        "path": relative_text(
            root,
            &root.join("artifacts/aws-operation-source-reconciliation-v04.json"),
        )?,
        "sha256": inputs.previous_sha,
        "left_unchanged": true,
    }))
}

fn summary(inputs: &Inputs, facts: &Facts) -> Value {
    let mut summary = json!({
        "explicit_unique_operations": inputs.operation_count,
        "explicit_sdk_send_callsites": facts.callsites.len(),
        "sdk_services": facts.services,
        "credential_provider_operation_boundaries": facts.provider_sites.len(),
        "callsite_positions_changed": facts.position_changes,
        "nonempty_channel_send_sites_excluded": facts.nonempty,
        "aws_source_files": facts.rows.len(),
    });
    if inputs.additions.is_present()
        && let Some(object) = summary.as_object_mut()
    {
        object.insert(
            "reviewed_additive_operations".to_owned(),
            json!(inputs.additions.rows.len()),
        );
        object.insert(
            "reviewed_additive_callsites".to_owned(),
            json!(reviewed_callsites(facts)),
        );
    }
    summary
}

fn reviewed_callsites(facts: &Facts) -> usize {
    facts
        .callsites
        .iter()
        .filter(|row| row.get("origin").and_then(Value::as_str) == Some("reviewed_addition"))
        .count()
}

fn method(inputs: &Inputs) -> String {
    if inputs.additions.is_present() {
        return format!(
            "{METHOD} Reviewed additive callsites come only from {OPERATION_ADDITIONS}, are \
verified against pinned SDK operation modules, and are never emulator outcomes."
        );
    }
    METHOD.to_owned()
}

fn reviewed_additions(inputs: &Inputs, facts: &Facts) -> Value {
    json!({
        "path": OPERATION_ADDITIONS,
        "sha256": inputs.additions.sha256,
        "bound_historical_matrix_sha256": inputs.matrix_sha,
        "operation_boundaries": inputs.additions.rows.len(),
        "callsites": reviewed_callsites(facts),
        "execution_status": "unexercised",
        "operations": additive_operations(inputs),
        "pinned_sdk_operation_modules": facts.addition_modules,
    })
}

fn additive_operations(inputs: &Inputs) -> Value {
    Value::Array(
        inputs
            .additions
            .rows
            .iter()
            .map(|row| {
                json!({
                    "id": row.id,
                    "sdk_crate": row.crate_name,
                    "sdk_version": row.key.1,
                    "api_operation": row.api_operation,
                    "status": row.status,
                    "execution_status": "unexercised",
                    "callsites": row.sites.len(),
                })
            })
            .collect(),
    )
}

fn credential_provider(facts: &Facts) -> Value {
    json!({
        "id": "sts.assume_role",
        "explicit_sdk_send": false,
        "source_calls": facts.provider_sites,
        "pinned_dependency": facts.provider_dependency,
    })
}

fn provider_dimension(inputs: &Inputs, catalogs: &CatalogFacts) -> Value {
    json!({
        "historical_type_count": inputs
            .matrix
            .pointer("/cloud_control_provider_dimension/resource_type_count")
            .cloned()
            .unwrap_or(Value::Null),
        "current_type_count": catalogs.descriptor_count,
        "catalog_sha256": catalogs.catalog_sha256,
        "root_type_count": catalogs.root_types,
        "parent_dependent_type_count": catalogs.parent_types,
        "parent_catalog_sha256": catalogs.parent_sha256,
        "runtime_catalog_canonical_sha256": catalogs.runtime_sha,
        "relationship_rule_count": catalogs.relationship_rules,
        "parent_relationship_rule_count": catalogs.parent_plans,
        "operation_ids": ["cloudcontrol.list_resources", "cloudcontrol.get_resource"],
        "meaning": "Descriptor/relationship breadth is separate from SDK operation boundaries and executed provider coverage.",
    })
}

fn emulator_evidence() -> Value {
    json!({
        "status": "pending",
        "current_run_bound": false,
        "meaning": "No endpoint or provider execution result is claimed by this source-only receipt.",
    })
}

fn limits() -> Value {
    json!([
        "Lexical audit, not a Rust AST/type proof or branch coverage measurement.",
        "Unrecognized call shapes and changed boundaries fail closed; new operations require an updated reviewed matrix.",
        "No live AWS calls, Cargo compilation, mutation campaign, or throughput measurement was performed.",
        "Default credential-chain network providers remain outside the explicit collector operation count.",
    ])
}

fn relative_text(root: &Path, path: &Path) -> Result<String, HarnessError> {
    let resolved = std::fs::canonicalize(path)?;
    let base = std::fs::canonicalize(root)?;
    Ok(super::super::files::slash_path(
        resolved
            .strip_prefix(&base)
            .map_err(|_| HarnessError::invalid("evidence path escaped the workspace root"))?,
    ))
}
