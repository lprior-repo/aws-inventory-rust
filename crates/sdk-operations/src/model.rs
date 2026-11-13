//! Serialized schema of `catalog/aws-sdk-operations.json`.

use serde::{Deserialize, Serialize};

/// Schema revision of the checked-in operations inventory.
pub const SCHEMA_VERSION: u32 = 1;

/// Complete inventory document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Inventory {
    /// Schema revision; currently [`SCHEMA_VERSION`].
    pub schema_version: u32,
    /// Registry that was scanned.
    pub source: Source,
    /// Crate and operation totals.
    pub summary: Summary,
    /// One row per generated SDK crate, sorted by name then version.
    pub crates: Vec<CrateEntry>,
}

/// Origin of the scanned crates.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Source {
    /// Absolute registry source directory.
    pub registry: String,
}

/// Aggregate counts for quick sanity checks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Summary {
    /// Number of crates inventoried.
    pub crates: usize,
    /// Number of operations inventoried across every crate.
    pub operations: usize,
}

/// One generated SDK crate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrateEntry {
    /// Crate package name, e.g. `aws-sdk-cloudcontrol`.
    pub name: String,
    /// Crate version, e.g. `1.122.0`.
    pub version: String,
    /// SHA-256 of the crate source tree.
    pub sha256: String,
    /// Operation modules, sorted by module name.
    pub operations: Vec<Operation>,
}

/// One operation module of a generated SDK crate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Operation {
    /// Stable module name under `src/operation`, e.g. `list_resources`.
    pub module: String,
    /// PascalCase API operation derived from the module name.
    pub api_operation: String,
    /// SHA-256 of the operation module source file.
    pub source_sha256: String,
}
