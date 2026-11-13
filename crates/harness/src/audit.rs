//! Lexical audit of Rust SDK call sites against retained evidence.
//!
//! This module reproduces two Python development tools:
//! `scripts/catalog/reconcile_aws_operations.py` (bounded lexical enumeration of
//! empty SDK `send()` calls plus pinned module digests) and
//! `scripts/catalog/refresh_cloudcontrol_coverage.py` (transport/typed-read
//! capability accounting). Neither tool makes AWS calls or compiles Cargo.
//!
//! Historical evidence is never rewritten: the reconciliation refuses to
//! replace the operation matrix or the preserved v0.4/v0.5 receipts, and the
//! refresh only restates an execution that is already bound to current
//! catalog, source, binary and fixture digests.

mod bind;
mod binding;
mod calls;
mod command;
mod coverage;
mod files;
mod format;
mod histogram;
mod inventory;
mod jhelpers;
mod json_ordered;
mod json_parse;
mod json_write;
mod ledger;
mod mask;
mod matrix;
mod operations;
mod persisted;
mod pins;
mod reconcile;
mod refresh;
mod resolve;
mod rollup;
mod tokens;
mod validate;

pub use bind::bind_emulator;
pub use calls::{CallSite, calls_in_source};
pub use command::{output_path, run_reconcile};
pub use coverage::apply_verification;
pub use files::read_json_audit;
pub use histogram::checked_histogram;
pub use inventory::source_inventory;
pub use json_ordered::{Json, py_eq};
pub use json_write::{dumps_default, pretty};
pub use ledger::request_evidence;
pub use matrix::{catalogs, historical_operations, verify_callsites};
pub use operations::{
    CoverageReport, document as operation_coverage, run as run_operation_coverage,
};
pub use persisted::persisted_reads;
pub use pins::{dependency_evidence, registry_file};
pub use reconcile::reconcile;
pub use refresh::run_refresh;
pub use rollup::write_rollup;
pub use validate::validate_summary;
