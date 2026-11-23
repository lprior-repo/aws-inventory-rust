mod coverage;
mod failures;
mod snapshot;
use crate::HarnessError;
pub use coverage::Coverage;
pub use failures::failed_operations;
pub use snapshot::capture_baseline;
use std::{
    collections::{BTreeMap, BTreeSet},
    io::Read,
    path::Path,
};

type Keys = BTreeMap<String, BTreeSet<(String, String, String)>>;

pub struct Baseline(Keys);

pub fn read_text(path: &Path) -> Result<String, HarnessError> {
    let mut text = String::new();
    std::fs::File::open(path)?
        .take(16_777_217)
        .read_to_string(&mut text)?;
    if text.len() > 16_777_216 {
        return Err(HarnessError::invalid("e2e text exceeds 16 MiB"));
    }
    Ok(text)
}

pub fn zero(summary: &str, metric: &str) -> Result<(), HarnessError> {
    let value = metric_value(summary, metric)?;
    if value != 0 {
        return Err(HarnessError::invalid(format!(
            "collector {metric}={value}; expected 0"
        )));
    }
    Ok(())
}

pub(super) fn metric_value(summary: &str, metric: &str) -> Result<u64, HarnessError> {
    let line = summary
        .lines()
        .rev()
        .find(|line| line.starts_with("resource_writes_scope=invocation "))
        .ok_or_else(|| HarnessError::invalid("collector summary missing"))?;
    line.split_whitespace()
        .find_map(|field| {
            field
                .split_once('=')
                .filter(|(key, _)| *key == metric)
                .map(|(_, value)| value)
        })
        .ok_or_else(|| HarnessError::invalid(format!("collector summary lacks {metric}")))?
        .parse()
        .map_err(|_| HarnessError::invalid(format!("collector summary has invalid {metric}")))
}

pub fn outcome(summary: &str) -> String {
    [
        "resource_writes",
        "rows",
        "complete",
        "partial",
        "unsupported",
        "failed",
        "work_remaining",
        "graph_pending",
        "inaccessible",
        "unobserved",
        "unattributed_resources",
    ]
    .iter()
    .filter_map(|metric| {
        metric_value(summary, metric)
            .ok()
            .map(|value| format!("{metric}={value}"))
    })
    .collect::<Vec<_>>()
    .join(" ")
}

pub async fn counts(
    database: &str,
    oracle: &Path,
    baseline: &Baseline,
    coverage: &Coverage,
) -> Result<(), HarnessError> {
    let expected = verified_keys(oracle_keys(&read_text(oracle)?)?, coverage)?;
    let kinds: Vec<_> = expected.keys().map(String::as_str).collect();
    let actual = snapshot::read(database, Some(&kinds)).await?;
    compare(&expected, &snapshot::added(actual, baseline))
}

fn verified_keys(expected: Keys, coverage: &Coverage) -> Result<Keys, HarnessError> {
    let verified =
        expected
            .into_iter()
            .try_fold(Keys::new(), |mut verified, (kind, identities)| {
                if let Some(reason) = coverage.reason(&kind)? {
                    println!(
                        "count {kind} seeded={} not-verified: catalog-deferred ({reason})",
                        identities.len()
                    );
                } else {
                    verified.insert(kind, identities);
                }
                Ok(verified)
            })?;
    if verified.is_empty() {
        return Err(HarnessError::invalid("no callable seeded kinds to verify"));
    }
    Ok(verified)
}

fn oracle_keys(text: &str) -> Result<Keys, HarnessError> {
    let keys = text
        .lines()
        .filter(|line| !line.is_empty())
        .try_fold(Keys::new(), oracle_row)?;
    if keys.is_empty() {
        return Err(HarnessError::invalid(
            "seed oracle contains no resources; nothing verified",
        ));
    }
    Ok(keys)
}

fn oracle_row(mut keys: Keys, line: &str) -> Result<Keys, HarnessError> {
    let fields: Vec<_> = line.split('\t').collect();
    match fields.as_slice() {
        ["resource", kind, account, region, id]
            if [kind, account, region, id]
                .iter()
                .all(|value| !value.is_empty()) =>
        {
            let key = ((*account).into(), (*region).into(), (*id).into());
            if !keys.entry((*kind).into()).or_default().insert(key) {
                return Err(HarnessError::invalid("duplicate oracle resource"));
            }
        }
        ["edge", _, _, _, _, _] => {}
        _ => return Err(HarnessError::invalid("malformed seed oracle row")),
    }
    Ok(keys)
}

fn identity_mismatch(
    kind: &str,
    expected: &BTreeSet<(String, String, String)>,
    actual: Option<&BTreeSet<(String, String, String)>>,
) -> HarnessError {
    let empty = BTreeSet::new();
    let actual = actual.map_or(&empty, |identities| identities);
    let differences = expected
        .symmetric_difference(actual)
        .take(5)
        .map(|(account, region, id)| format!("{account}/{region}/{id}"))
        .collect::<Vec<_>>()
        .join(", ");
    HarnessError::invalid(format!(
        "seeded identities/counts differ for {kind}: {differences}"
    ))
}

fn compare(expected: &Keys, actual: &Keys) -> Result<(), HarnessError> {
    expected.iter().try_for_each(|(kind, keys)| {
        let found = actual.get(kind);
        let count = found.map_or(0, BTreeSet::len);
        println!("count {kind} expected={} actual={count}", keys.len());
        if found != Some(keys) {
            return Err(identity_mismatch(kind, keys, found));
        }
        Ok(())
    })
}

#[cfg(test)]
mod tests;
