use crate::{TestResult, runtime};
use inventory_engine::inventory::decode_scope_key;
use sqlx::SqlitePool;
use std::{collections::BTreeMap, path::Path};

type ProbeTable = BTreeMap<(String, String), String>;

fn probe_report(path: &Path) -> TestResult<ProbeTable> {
    let mut table = ProbeTable::new();
    for line in std::fs::read_to_string(path)?.lines() {
        let fields: Vec<&str> = line.split_whitespace().collect();
        let [service, operation, _protocol, outcome, ..] = fields.as_slice() else {
            continue;
        };
        if !matches!(
            *outcome,
            "items" | "empty" | "unsupported" | "failed" | "skipped"
        ) {
            continue;
        }
        let key = ((*service).to_owned(), (*operation).to_owned());
        if table.insert(key, (*outcome).to_owned()).is_some() {
            return Err(format!("duplicate probe entry {service} {operation}").into());
        }
    }
    Ok(table)
}

fn contradicts(stored: &str, probed: &str) -> bool {
    matches!(
        (stored, probed),
        ("failed" | "inaccessible", "items" | "empty") | ("unsupported", "items")
    )
}

pub(super) async fn violations(pool: &SqlitePool) -> TestResult<Vec<String>> {
    let probes = probe_report(&runtime::workspace().join("catalog/emulator-probe.report"))?;
    let rows: Vec<(Vec<u8>, String, Option<String>)> =
        sqlx::query_as("SELECT scope_key,outcome,detail FROM scope_outcomes")
            .fetch_all(pool)
            .await?;
    let mut matched = 0usize;
    let mut violations = Vec::new();
    for (scope, outcome, detail) in rows {
        let scope = decode_scope_key(&scope)?;
        let key = (
            scope.service().as_str().to_owned(),
            scope.operation().as_str().to_owned(),
        );
        let Some(probed) = probes.get(&key) else {
            continue;
        };
        matched = matched.saturating_add(1);
        if contradicts(&outcome, probed) {
            violations.push(format!(
                "{} {} stored {outcome} (detail: {}) but probe records {probed}",
                key.0,
                key.1,
                detail.unwrap_or_default()
            ));
        }
    }
    if matched == 0 {
        violations.push("no stored scope matched any probe report entry".to_owned());
    }
    Ok(violations)
}
