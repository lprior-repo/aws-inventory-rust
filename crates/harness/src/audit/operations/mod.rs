//! Coverage of the published SDK operation inventory by collector callsites.
//!
//! The source universe is every operation row in `catalog/aws-sdk-operations.json`.
//! The exercised set is proven by the current empty `send()` callsites under
//! `crates/aws/src`, resolved to the crate version pinned by `Cargo.lock` and
//! qualified by recorded emulator outcomes. An operation recorded as
//! `sdk_decode_failure` is not counted as exercised.
//!
//! Cloud Control `ListResources` and `GetResource` are classified as
//! provider-generic: they count once each, never once per descriptor type.
//!
//! Reviewed additive callsites from `catalog/aws-operation-additions.json` are
//! reported separately and never counted as exercised.

pub(crate) mod additions;
pub(crate) mod inventory;
mod join;
mod render;
mod sources;
mod value;

pub use join::document;
pub use render::run;

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

use crate::HarnessError;

/// Operation identity inside one crate version.
pub(crate) type OperationKey = (String, String, String);

/// Published per-crate SDK operation inventory.
pub const INVENTORY: &str = "catalog/aws-sdk-operations.json";
/// Reviewed operation matrix and its recorded emulator outcomes.
pub const OPERATION_MATRIX: &str = "catalog/aws-operations.json";
/// Reviewed additive operation overlay; absent is a no-op.
pub const OPERATION_ADDITIONS: &str = "catalog/aws-operation-additions.json";
/// Newest source-bound reconciliation receipt.
pub const RECONCILIATION: &str = "artifacts/aws-operation-source-reconciliation-v06.json";
/// Rendered coverage document.
pub const DOCUMENT: &str = "docs/OPERATION_COVERAGE.md";
/// Deterministic size of the unexercised sample in the JSON report.
pub const SAMPLE_LIMIT: usize = 50;
/// SDK crate name prefix.
pub(crate) const CRATE_PREFIX: &str = "aws-sdk-";
/// Modules served generically for every Cloud Control descriptor type.
const PROVIDER_GENERIC_MODULES: [&str; 2] = ["get_resource", "list_resources"];

/// One operation row from the published inventory.
#[derive(Clone, Debug)]
pub struct OperationRow {
    /// SDK service namespace derived from the crate name.
    pub service: String,
    /// SDK crate name.
    pub crate_name: String,
    /// Crate version; empty when the inventory omits it.
    pub version: String,
    /// Snake-case client method and operation module.
    pub module: String,
    /// Recorded API operation name.
    pub api_operation: String,
}

impl OperationRow {
    pub(crate) fn key(&self) -> (String, String, String) {
        (
            self.crate_name.clone(),
            self.version.clone(),
            self.module.clone(),
        )
    }

    pub(crate) fn provider_generic(&self) -> bool {
        self.crate_name == "aws-sdk-cloudcontrol"
            && PROVIDER_GENERIC_MODULES.contains(&self.module.as_str())
    }
}

/// Parsed inventory, the crate version active for each service, and digests.
pub(crate) struct Inventory {
    pub(crate) rows: BTreeMap<OperationKey, OperationRow>,
    /// Service namespace to the `(crate, version)` pinned by `Cargo.lock`.
    pub(crate) active: BTreeMap<String, (String, String)>,
    /// Crate name to `(service, version)` for the active rows.
    pub(crate) by_crate: BTreeMap<String, (String, String)>,
    pub(crate) sha256: String,
    pub(crate) crates: usize,
}

/// Proven exercise claim for one operation.
#[derive(Clone, Debug, Default)]
pub(crate) struct Claim {
    pub(crate) callsites: BTreeSet<String>,
    pub(crate) status: Option<String>,
}

/// One exercised operation with its evidence.
#[derive(Clone, Debug)]
pub struct ExercisedRow {
    /// Service namespace.
    pub service: String,
    /// SDK crate.
    pub crate_name: String,
    /// SDK method.
    pub module: String,
    /// API operation.
    pub api_operation: String,
    /// Provider-generic Cloud Control coverage flag.
    pub provider_generic: bool,
    /// Recorded emulator outcome when one applies.
    pub status: Option<String>,
    /// Sorted source citations.
    pub evidence: Vec<String>,
}

/// One reviewed callsite of an additive operation.
#[derive(Clone, Debug)]
pub struct AdditiveSite {
    /// Workspace-relative source path.
    pub path: String,
    /// Enclosing function.
    pub function: String,
    /// Line of the request builder.
    pub builder_line: u64,
    /// Line of the operation method.
    pub operation_line: u64,
    /// Line of `send`.
    pub send_line: u64,
}

/// One reviewed additive operation that is not exercised.
#[derive(Clone, Debug)]
pub struct AdditiveRow {
    /// Service namespace.
    pub service: String,
    /// SDK crate.
    pub crate_name: String,
    /// SDK method.
    pub module: String,
    /// API operation.
    pub api_operation: String,
    /// Reviewed status; always unexercised.
    pub status: String,
    /// Pinned SDK operation module digest.
    pub module_sha256: String,
    /// Sorted reviewed callsite citations.
    pub evidence: Vec<String>,
    /// Exact reviewed callsite line bindings.
    pub sites: Vec<AdditiveSite>,
}

/// One operation removed from the exercised set by recorded evidence.
#[derive(Clone, Debug)]
pub struct ExcludedRow {
    /// Service namespace.
    pub service: String,
    /// SDK crate.
    pub crate_name: String,
    /// SDK method.
    pub module: String,
    /// API operation.
    pub api_operation: String,
    /// Recorded outcome that excluded it.
    pub status: String,
    /// Document supplying that outcome.
    pub source: String,
}

/// One service rollup.
#[derive(Clone, Debug)]
pub struct ServiceRow {
    /// Service namespace.
    pub service: String,
    /// SDK crate.
    pub crate_name: String,
    /// Operation rows published for every version of the crate.
    pub total: usize,
    /// Operations exercised.
    pub exercised: usize,
}

/// Complete coverage report, rendered to JSON or Markdown.
#[derive(Default)]
pub struct CoverageReport {
    pub(crate) inventory_sha256: String,
    pub(crate) crates: usize,
    pub(crate) operations: usize,
    pub(crate) scan_files: usize,
    pub(crate) scan_callsites: usize,
    pub(crate) nonempty_sends: usize,
    pub(crate) scan_sha256: Option<String>,
    pub(crate) scan_status: String,
    pub(crate) reviewed_files: usize,
    pub(crate) reviewed_changed: usize,
    pub(crate) matrix_sha256: String,
    pub(crate) matrix_execution: String,
    pub(crate) reconciliation_path: String,
    pub(crate) reconciliation_status: String,
    pub(crate) reconciliation_sha256: Option<String>,
    pub(crate) status_source: String,
    pub(crate) status_count: usize,
    pub(crate) exercised: Vec<ExercisedRow>,
    pub(crate) excluded: Vec<ExcludedRow>,
    pub(crate) services: Vec<ServiceRow>,
    pub(crate) sample: Vec<OperationRow>,
    pub(crate) sample_truncated: bool,
    pub(crate) additions_status: String,
    pub(crate) additions_sha256: Option<String>,
    pub(crate) additions_reviewed_at: String,
    pub(crate) additions_matrix_sha256: String,
    pub(crate) additions: Vec<AdditiveRow>,
    pub(crate) provider_generic_total: usize,
    pub(crate) provider_generic_exercised: usize,
    pub(crate) typed_native_exercised: usize,
}

impl CoverageReport {
    /// Total operation rows in the published inventory.
    pub fn operations(&self) -> usize {
        self.operations
    }

    /// Operations proven exercised.
    pub fn exercised(&self) -> usize {
        self.exercised.len()
    }

    /// Reviewed additive operations admitted from the overlay.
    pub fn reviewed_additions(&self) -> usize {
        self.additions.len()
    }

    /// Operations not exercised.
    pub fn unexercised(&self) -> Result<usize, HarnessError> {
        self.operations
            .checked_sub(self.exercised())
            .ok_or_else(|| HarnessError::invalid("exercised count exceeds the inventory"))
    }

    /// Render the machine-readable report.
    pub fn to_value(&self) -> Result<Value, HarnessError> {
        value::render(self)
    }
}
