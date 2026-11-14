//! Assemble coverage claims, exclusions and service rollups.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::HarnessError;

use super::inventory::{exercised_status, load, scan, source_texts};
use super::sources::{StatusEvidence, statuses};
use super::{
    Claim, CoverageReport, ExcludedRow, ExercisedRow, Inventory, OperationKey, OperationRow,
    SAMPLE_LIMIT, ServiceRow,
};

mod additive;
mod report;

use report::{Parts, Scan, report};

/// Build the coverage report from the workspace.
pub fn document(root: &Path) -> Result<CoverageReport, HarnessError> {
    let inventory = load(root)?;
    let sources = source_texts(root)?;
    let evidence = statuses(root, &inventory, &sources)?;
    let (mut claims, scan) = scan_state(root, &inventory, &evidence)?;
    claims.retain(|key, _| !evidence.additions.keys.contains(key));
    let excluded = claims_by_operation(&inventory, &evidence, &mut claims)?;
    let exercised = exercised_rows(&inventory, &evidence, &claims)?;
    let services = service_rows(&inventory, &exercised);
    let sample = unexercised_sample(&inventory, &claims, &evidence.additions.keys);
    let additive_rows = additive::rows(&evidence.additions);
    report(
        inventory,
        scan,
        evidence,
        Parts {
            exercised,
            excluded,
            services,
            sample: sample.0,
            sample_truncated: sample.1,
            additive: additive_rows,
        },
    )
}

fn scan_state(
    root: &Path,
    inventory: &Inventory,
    evidence: &StatusEvidence,
) -> Result<(BTreeMap<OperationKey, Claim>, Scan), HarnessError> {
    match scan(root, inventory) {
        Ok(scanned) => verified_scan(scanned, evidence),
        Err(error) => Ok((reviewed_claims(&evidence.claims), unavailable(&error))),
    }
}

fn require_claims(
    current: &BTreeSet<&OperationKey>,
    reviewed: &BTreeSet<&OperationKey>,
) -> Result<(), HarnessError> {
    if current == reviewed {
        return Ok(());
    }
    let missing: Vec<String> = reviewed
        .difference(current)
        .take(5)
        .map(|key| format!("{key:?}"))
        .collect();
    let unreviewed: Vec<String> = current
        .difference(reviewed)
        .take(5)
        .map(|key| format!("{key:?}"))
        .collect();
    Err(HarnessError::invalid(format!(
        "unreviewed SDK callsites disagree with the reviewed operation matrix; add reviewed rows to \
catalog/aws-operation-additions.json; missing={missing:?} unreviewed={unreviewed:?}"
    )))
}

fn verified_scan(
    scanned: super::inventory::Scan,
    evidence: &StatusEvidence,
) -> Result<(BTreeMap<OperationKey, Claim>, Scan), HarnessError> {
    let current: BTreeSet<&OperationKey> = scanned.claims.keys().collect();
    let reviewed: BTreeSet<&OperationKey> = evidence
        .claims
        .keys()
        .chain(evidence.additions.keys.iter())
        .collect();
    require_claims(&current, &reviewed)?;
    additive::require_sites(&scanned.claims, &evidence.additions)?;
    let scan = Scan {
        status: "verified".to_owned(),
        files: scanned.files,
        callsites: scanned.callsites,
        nonempty: scanned.nonempty,
        sha256: Some(scanned.sha256),
    };
    Ok((scanned.claims, scan))
}

fn unavailable(error: &HarnessError) -> Scan {
    Scan {
        status: format!("unavailable: {error}"),
        files: 0,
        callsites: 0,
        nonempty: 0,
        sha256: None,
    }
}

fn reviewed_claims(
    claims: &BTreeMap<OperationKey, BTreeSet<String>>,
) -> BTreeMap<OperationKey, Claim> {
    let mut result = BTreeMap::new();
    for (key, callsites) in claims {
        result.insert(
            key.clone(),
            Claim {
                callsites: callsites.clone(),
                status: None,
            },
        );
    }
    result
}

fn claims_by_operation(
    inventory: &Inventory,
    evidence: &StatusEvidence,
    claims: &mut BTreeMap<OperationKey, Claim>,
) -> Result<Vec<ExcludedRow>, HarnessError> {
    let mut excluded = Vec::new();
    for (key, status) in &evidence.statuses {
        if exercised_status(status) {
            if let Some(claim) = claims.get_mut(key) {
                claim.status = Some(status.clone());
            }
            continue;
        }
        if let Some(row) = excluded_row(inventory, evidence, claims, key, status) {
            excluded.push(row);
        }
    }
    claims.retain(|_, claim| claim.status.is_some());
    excluded
        .sort_by(|left, right| (&left.service, &left.module).cmp(&(&right.service, &right.module)));
    Ok(excluded)
}

fn excluded_row(
    inventory: &Inventory,
    evidence: &StatusEvidence,
    claims: &mut BTreeMap<OperationKey, Claim>,
    key: &OperationKey,
    status: &str,
) -> Option<ExcludedRow> {
    claims.remove(key)?;
    let row = inventory.rows.get(key)?;
    Some(ExcludedRow {
        service: row.service.clone(),
        crate_name: row.crate_name.clone(),
        module: row.module.clone(),
        api_operation: row.api_operation.clone(),
        status: status.to_owned(),
        source: evidence.status_source_path.clone(),
    })
}

fn exercised_rows(
    inventory: &Inventory,
    evidence: &StatusEvidence,
    claims: &BTreeMap<OperationKey, Claim>,
) -> Result<Vec<ExercisedRow>, HarnessError> {
    let mut exercised = Vec::with_capacity(claims.len());
    for (key, claim) in claims {
        exercised.push(exercised_row(inventory, evidence, key, claim)?);
    }
    exercised.sort_by(|left, right| {
        (&left.service, &left.module, &left.api_operation).cmp(&(
            &right.service,
            &right.module,
            &right.api_operation,
        ))
    });
    Ok(exercised)
}

fn exercised_row(
    inventory: &Inventory,
    evidence: &StatusEvidence,
    key: &OperationKey,
    claim: &Claim,
) -> Result<ExercisedRow, HarnessError> {
    let row = inventory
        .rows
        .get(key)
        .ok_or_else(|| HarnessError::invalid("claim has no inventory row"))?;
    Ok(ExercisedRow {
        service: row.service.clone(),
        crate_name: row.crate_name.clone(),
        module: row.module.clone(),
        api_operation: row.api_operation.clone(),
        provider_generic: row.provider_generic(),
        status: claim.status.clone(),
        evidence: citations(claim, evidence),
    })
}

fn citations(claim: &Claim, evidence: &StatusEvidence) -> Vec<String> {
    let mut citations: Vec<String> = claim
        .callsites
        .iter()
        .map(|site| {
            if site.starts_with("reviewed:") {
                site.clone()
            } else {
                format!("callsite:{site}")
            }
        })
        .collect();
    if claim.status.is_some() {
        citations.push(format!("reconciled:{}", evidence.status_source_path));
    }
    citations.sort();
    citations
}

fn service_rows(inventory: &Inventory, exercised: &[ExercisedRow]) -> Vec<ServiceRow> {
    let mut services: BTreeMap<String, ServiceRow> = BTreeMap::new();
    for row in inventory.rows.values() {
        let slot = services
            .entry(row.service.clone())
            .or_insert_with(|| ServiceRow {
                service: row.service.clone(),
                crate_name: row.crate_name.clone(),
                total: 0,
                exercised: 0,
            });
        slot.total = slot.total.saturating_add(1);
    }
    for row in exercised {
        if let Some(slot) = services.get_mut(&row.service) {
            slot.exercised = slot.exercised.saturating_add(1);
        }
    }
    services.into_values().collect()
}

fn unexercised_sample(
    inventory: &Inventory,
    claims: &BTreeMap<OperationKey, Claim>,
    addition_keys: &BTreeSet<OperationKey>,
) -> (Vec<OperationRow>, bool) {
    let mut sample: Vec<OperationRow> = Vec::new();
    for (key, row) in &inventory.rows {
        if !claims.contains_key(key) && !addition_keys.contains(key) && sample.len() < SAMPLE_LIMIT
        {
            sample.push(row.clone());
        }
    }
    let reviewed = claims.len().saturating_add(addition_keys.len());
    let truncated = inventory.rows.len() > reviewed.saturating_add(sample.len());
    (sample, truncated)
}
