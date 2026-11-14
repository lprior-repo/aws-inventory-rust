//! Machine-readable rendering of the coverage report.

use serde_json::{Value, json};

use crate::HarnessError;

use super::{CoverageReport, INVENTORY, OPERATION_ADDITIONS, OPERATION_MATRIX, SAMPLE_LIMIT};

const METHOD: &str = "Join every operation row in the published SDK inventory by (crate, version, module) with current empty-argument .send() callsites under crates/aws/src, resolving each service to the crate version pinned by Cargo.lock. The callsite scan separately reports lexical invocations. Exercise credit additionally requires a qualifying recorded emulator outcome at the exact same SDK version; historical outcomes are never transferred to upgraded versions. sdk_decode_failure operations are not exercised. Count Cloud Control ListResources/GetResource once each, not per descriptor type. Reviewed additions are admitted from catalog/aws-operation-additions.json only when bound to this matrix, this inventory, exact current callsites and pinned SDK modules; they are reported separately and never counted as exercised.";

const LIMITS: [&str; 6] = [
    "A lexical callsite proves the collector invokes the operation; it does not prove every branch, request shape, IAM permission, or live AWS response.",
    "Cloud Control descriptor breadth is resource-type coverage, not SDK operation coverage.",
    "Operations absent from the exercised set lack qualifying current-version execution evidence; they may still have collector callsites or historical outcomes.",
    "The emulator status column records only an outcome for the exact active SDK version, not live AWS behavior or an outcome inherited from an older SDK.",
    "The inventory counts every cached generated crate version, not just active versions. Rows from versions not pinned by Cargo.lock remain unexercised; historical execution outcomes do not establish that an upgraded SDK version was exercised.",
    "A reviewed addition is not an emulator outcome: its status is unexercised or pending_execution, it carries no execution receipt, and reviewed_additions_exercised is always zero in this report.",
];

impl CoverageReport {
    fn inventory_value(&self) -> Value {
        json!({
            "path": INVENTORY,
            "sha256": self.inventory_sha256,
            "crates": self.crates,
            "operations": self.operations,
        })
    }

    fn callsite_value(&self) -> Value {
        json!({
            "path": "crates/aws/src",
            "files": self.scan_files,
            "callsites": self.scan_callsites,
            "nonempty_channel_sends_excluded": self.nonempty_sends,
            "sha256": self.scan_sha256,
            "sha256_method": "sha256 over sorted 'path:source_sha256' lines",
            "status": self.scan_status,
        })
    }

    fn matrix_value(&self) -> Value {
        json!({
            "path": OPERATION_MATRIX,
            "sha256": self.matrix_sha256,
            "execution": self.matrix_execution,
            "statuses_applied": self.status_count,
            "reviewed_source_files": self.reviewed_files,
            "reviewed_source_files_changed": self.reviewed_changed,
        })
    }

    fn reconciliation_value(&self) -> Value {
        json!({
            "path": self.reconciliation_path,
            "status": self.reconciliation_status,
            "sha256": self.reconciliation_sha256,
            "applied": self.status_source == "reconciliation",
        })
    }

    fn sources_value(&self) -> Value {
        json!({
            "operation_inventory": self.inventory_value(),
            "callsite_scan": self.callsite_value(),
            "operation_matrix": self.matrix_value(),
            "operation_additions": self.additions_value(),
            "reconciliation": self.reconciliation_value(),
        })
    }

    fn additions_value(&self) -> Value {
        if self.additions_status == "absent" {
            return Value::Null;
        }
        json!({
            "path": OPERATION_ADDITIONS,
            "status": self.additions_status,
            "sha256": self.additions_sha256,
            "reviewed_at_utc": self.additions_reviewed_at,
            "bound_historical_matrix_sha256": self.additions_matrix_sha256,
            "operation_boundaries": self.additions.len(),
        })
    }

    fn summary_value(&self, exercised: usize, unexercised: usize) -> Value {
        let services_with_exercised = self.services.iter().filter(|row| row.exercised > 0).count();
        json!({
            "operations_total": self.operations,
            "crates_total": self.crates,
            "services_total": self.services.len(),
            "exercised": exercised,
            "unexercised": unexercised,
            "provider_generic_operations_total": self.provider_generic_total,
            "provider_generic_exercised": self.provider_generic_exercised,
            "typed_native_exercised": self.typed_native_exercised,
            "services_with_exercised_operations": services_with_exercised,
            "reviewed_additions": self.additions.len(),
            "reviewed_additions_exercised": 0,
        })
    }

    fn by_service(&self) -> Vec<Value> {
        self.services
            .iter()
            .map(|row| {
                json!({
                    "service": row.service,
                    "crate": row.crate_name,
                    "total": row.total,
                    "exercised": row.exercised,
                    "unexercised": row.total.saturating_sub(row.exercised),
                })
            })
            .collect()
    }

    fn exercised_values(&self) -> Vec<Value> {
        self.exercised
            .iter()
            .map(|row| {
                json!({
                    "service": row.service,
                    "crate": row.crate_name,
                    "module": row.module,
                    "api_operation": row.api_operation,
                    "provider_generic": row.provider_generic,
                    "emulator_status": row.status,
                    "evidence": row.evidence,
                })
            })
            .collect()
    }

    fn additions_values(&self) -> Vec<Value> {
        self.additions
            .iter()
            .map(|row| {
                json!({
                    "service": row.service,
                    "crate": row.crate_name,
                    "module": row.module,
                    "api_operation": row.api_operation,
                    "status": row.status,
                    "execution_status": "unexercised",
                    "sdk_operation_module_sha256": row.module_sha256,
                    "callsites": row.sites.iter().map(|site| json!({
                        "path": site.path,
                        "function": site.function,
                        "builder_line": site.builder_line,
                        "operation_line": site.operation_line,
                        "send_line": site.send_line,
                    })).collect::<Vec<Value>>(),
                    "evidence": row.evidence,
                })
            })
            .collect()
    }

    fn excluded_values(&self) -> Vec<Value> {
        self.excluded
            .iter()
            .map(|row| {
                json!({
                    "service": row.service,
                    "crate": row.crate_name,
                    "module": row.module,
                    "api_operation": row.api_operation,
                    "status": row.status,
                    "source": row.source,
                })
            })
            .collect()
    }

    fn sample_values(&self) -> Vec<Value> {
        self.sample
            .iter()
            .map(|row| {
                json!({
                    "service": row.service,
                    "crate": row.crate_name,
                    "version": row.version,
                    "module": row.module,
                    "api_operation": row.api_operation,
                })
            })
            .collect()
    }

    /// Render the machine-readable report.
    fn document(&self) -> Result<Value, HarnessError> {
        let exercised = self.exercised();
        let unexercised = self.unexercised()?;
        Ok(json!({
            "schema_version": 1,
            "sources": self.sources_value(),
            "summary": self.summary_value(exercised, unexercised),
            "status_source": self.status_source,
            "by_service": self.by_service(),
            "exercised_operations": self.exercised_values(),
            "excluded_operations": self.excluded_values(),
            "reviewed_additions": self.additions_values(),
            "unexercised_sample": self.sample_values(),
            "unexercised_sample_limit": SAMPLE_LIMIT,
            "unexercised_sample_truncated": self.sample_truncated,
            "method": METHOD,
            "limits": LIMITS,
        }))
    }
}

/// Render the machine-readable coverage report.
pub(super) fn render(report: &CoverageReport) -> Result<Value, HarnessError> {
    report.document()
}
