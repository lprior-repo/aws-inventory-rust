//! Coverage report assembly from the scan, evidence and joined rows.

use crate::HarnessError;

use super::super::sources::StatusEvidence;
use super::super::{
    AdditiveRow, CoverageReport, ExcludedRow, ExercisedRow, Inventory, OperationRow,
    RECONCILIATION, ServiceRow,
};

pub(super) struct Scan {
    pub(super) status: String,
    pub(super) files: usize,
    pub(super) callsites: usize,
    pub(super) nonempty: usize,
    pub(super) sha256: Option<String>,
}

pub(super) struct Parts {
    pub(super) exercised: Vec<ExercisedRow>,
    pub(super) excluded: Vec<ExcludedRow>,
    pub(super) services: Vec<ServiceRow>,
    pub(super) sample: Vec<OperationRow>,
    pub(super) sample_truncated: bool,
    pub(super) additive: Vec<AdditiveRow>,
}

pub(super) fn report(
    inventory: Inventory,
    scan: Scan,
    evidence: StatusEvidence,
    parts: Parts,
) -> Result<CoverageReport, HarnessError> {
    let (total, generic, typed) = metrics(&inventory, &parts)?;
    let mut report = census(inventory, scan, evidence);
    fill(&mut report, parts);
    report.provider_generic_total = total;
    report.provider_generic_exercised = generic;
    report.typed_native_exercised = typed;
    Ok(report)
}

fn metrics(inventory: &Inventory, parts: &Parts) -> Result<(usize, usize, usize), HarnessError> {
    let total = inventory
        .rows
        .values()
        .filter(|row| row.provider_generic())
        .count();
    let generic = parts
        .exercised
        .iter()
        .filter(|row| row.provider_generic)
        .count();
    let typed = parts
        .exercised
        .len()
        .checked_sub(generic)
        .ok_or_else(|| HarnessError::invalid("provider-generic count exceeds exercised"))?;
    Ok((total, generic, typed))
}

fn fill(report: &mut CoverageReport, parts: Parts) {
    report.exercised = parts.exercised;
    report.excluded = parts.excluded;
    report.services = parts.services;
    report.sample = parts.sample;
    report.sample_truncated = parts.sample_truncated;
    report.additions = parts.additive;
}

fn census(inventory: Inventory, scan: Scan, evidence: StatusEvidence) -> CoverageReport {
    let mut report = base_census(inventory, scan);
    attach_evidence(&mut report, evidence);
    report
}

fn base_census(inventory: Inventory, scan: Scan) -> CoverageReport {
    CoverageReport {
        inventory_sha256: inventory.sha256,
        crates: inventory.crates,
        operations: inventory.rows.len(),
        scan_files: scan.files,
        scan_callsites: scan.callsites,
        nonempty_sends: scan.nonempty,
        scan_sha256: scan.sha256,
        scan_status: scan.status,
        ..CoverageReport::default()
    }
}

fn attach_evidence(report: &mut CoverageReport, evidence: StatusEvidence) {
    report.reviewed_files = evidence.reviewed_files;
    report.reviewed_changed = evidence.reviewed_changed;
    report.matrix_sha256 = evidence.matrix_sha256;
    report.matrix_execution = evidence.matrix_execution;
    report.reconciliation_path = RECONCILIATION.to_owned();
    report.reconciliation_status = evidence.reconciliation_status;
    report.reconciliation_sha256 = evidence.reconciliation_sha256;
    report.status_source = evidence.status_source;
    report.status_count = evidence.statuses.len();
    report.additions_status = evidence.additions.status.clone();
    report.additions_sha256 = evidence
        .additions
        .is_present()
        .then(|| evidence.additions.sha256.clone());
    report.additions_reviewed_at = evidence.additions.reviewed_at_utc;
    report.additions_matrix_sha256 = evidence.additions.matrix_sha256;
}
