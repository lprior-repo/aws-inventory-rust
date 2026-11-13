//! Join reviewed additions into rows that never join the exercised set.

use std::collections::{BTreeMap, BTreeSet};

use crate::HarnessError;

use super::super::additions::Additions;
use super::super::{AdditiveRow, AdditiveSite, Claim, OPERATION_ADDITIONS, OperationKey};

/// Require the scanned callsites of every addition to equal its review.
pub(super) fn require_sites(
    scanned: &BTreeMap<OperationKey, Claim>,
    additions: &Additions,
) -> Result<(), HarnessError> {
    for row in &additions.rows {
        let declared: BTreeSet<String> = row
            .sites
            .iter()
            .map(|site| format!("{}:{}", site.path, site.function))
            .collect();
        let current = scanned
            .get(&row.key)
            .map(|claim| claim.callsites.clone())
            .unwrap_or_default();
        if declared != current {
            return Err(HarnessError::invalid(format!(
                "reviewed addition {} callsites changed; re-review {OPERATION_ADDITIONS}",
                row.id
            )));
        }
    }
    Ok(())
}

/// Additive rows in deterministic order, with reviewed evidence citations.
pub(super) fn rows(additions: &Additions) -> Vec<AdditiveRow> {
    let mut rows: Vec<AdditiveRow> = additions
        .rows
        .iter()
        .map(|row| AdditiveRow {
            service: row.service.clone(),
            crate_name: row.crate_name.clone(),
            module: row.module.clone(),
            api_operation: row.api_operation.clone(),
            status: row.status.clone(),
            module_sha256: row.module_sha256.clone(),
            evidence: evidence(row),
            sites: row.sites.iter().map(site).collect(),
        })
        .collect();
    rows.sort_by(|left, right| (&left.service, &left.module).cmp(&(&right.service, &right.module)));
    rows
}

fn site(value: &super::super::additions::AdditionSite) -> AdditiveSite {
    AdditiveSite {
        path: value.path.clone(),
        function: value.function.clone(),
        builder_line: value.builder_line,
        operation_line: value.operation_line,
        send_line: value.send_line,
    }
}

fn evidence(row: &super::super::additions::AdditionRow) -> Vec<String> {
    let mut evidence: Vec<String> = row
        .sites
        .iter()
        .map(|site| {
            format!(
                "reviewed:{OPERATION_ADDITIONS}:{}:{}",
                site.path, site.function
            )
        })
        .collect();
    evidence.push("unexercised:no_execution_receipt".to_owned());
    evidence.sort();
    evidence
}
