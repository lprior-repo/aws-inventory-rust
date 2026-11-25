mod coverage;
#[path = "../coverage_status.rs"]
mod coverage_status;
mod failures;
mod snapshot;
use crate::HarnessError;
pub use coverage::Coverage;
pub(super) use coverage_status::Selection;
pub use failures::failed_operations;
pub use snapshot::capture_baseline;
use std::{
    collections::{BTreeMap, BTreeSet},
    io::Read,
    path::Path,
};

type Keys = BTreeMap<String, BTreeSet<(String, String, String)>>;
type Counts = BTreeMap<String, usize>;

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
    verify_keys(database, baseline, &expected).await
}

pub(super) async fn counts_with_report(
    database: &str,
    work: &Path,
    baseline: &Baseline,
    coverage: &Coverage,
    selection: Selection<'_>,
) -> Result<(), HarnessError> {
    let expected = read_text(&work.join("oracle.tsv"))
        .and_then(|text| oracle_keys(&text))
        .and_then(|keys| verified_keys(keys, coverage));
    let empty = Keys::new();
    report_then_verify(
        expected,
        async |keys| {
            coverage
                .report(
                    database,
                    work,
                    keys.map_or(&empty, |keys| &keys.required),
                    selection,
                )
                .await
        },
        async |keys| verify_keys(database, baseline, &keys).await,
    )
    .await
}

async fn report_then_verify<T>(
    expected: Result<T, HarnessError>,
    report: impl AsyncFnOnce(Option<&T>) -> Result<(), HarnessError>,
    verify: impl AsyncFnOnce(T) -> Result<(), HarnessError>,
) -> Result<(), HarnessError> {
    report(expected.as_ref().ok()).await?;
    verify(expected?).await
}

async fn verify_keys(
    database: &str,
    baseline: &Baseline,
    expected: &Expected,
) -> Result<(), HarnessError> {
    let (mut actual, mut counts) = snapshot::read_snapshot(database).await?;
    snapshot::relevant(
        &mut actual,
        &mut counts,
        baseline,
        expected,
        &mut std::io::stdout().lock(),
    )?;
    compare(&expected.required, &actual, &counts)
}

#[derive(Default)]
struct Expected {
    required: Keys,
    deferred: Keys,
}

fn verified_keys(expected: Keys, coverage: &Coverage) -> Result<Expected, HarnessError> {
    let verified = expected.into_iter().try_fold(
        Expected::default(),
        |mut verified, (kind, identities)| {
            if let Some(reason) = coverage.reason(&kind)? {
                println!(
                    "count {kind} seeded={} not-verified: catalog-deferred ({reason})",
                    identities.len()
                );
                verified.deferred.insert(kind, identities);
            } else {
                verified.required.insert(kind, identities);
            }
            Ok::<_, HarnessError>(verified)
        },
    )?;
    if verified.required.is_empty() {
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
        | ["resource", kind, account, region, id, "service-created"]
            if [kind, account, region, id]
                .iter()
                .all(|value| !value.is_empty()) =>
        {
            let key = ((*account).into(), (*region).into(), (*id).into());
            if !keys.entry((*kind).into()).or_default().insert(key) {
                return Err(HarnessError::invalid("duplicate oracle resource"));
            }
        }
        [
            "edge",
            source_kind,
            source_id,
            relation,
            target_kind,
            target_id,
        ] if [source_kind, source_id, relation, target_kind, target_id]
            .iter()
            .all(|value| !value.is_empty()) => {}
        _ => return Err(HarnessError::invalid("malformed seed oracle row")),
    }
    Ok(keys)
}

type Identities = BTreeSet<(String, String, String)>;

fn listed<'a>(identities: impl Iterator<Item = &'a (String, String, String)>) -> String {
    let mut identities = identities;
    let mut listed = identities
        .by_ref()
        .take(5)
        .map(|(account, region, id)| format!("{account}/{region}/{id}"))
        .collect::<Vec<_>>();
    let omitted = identities.count();
    if omitted != 0 {
        listed.push(format!("+{omitted} more"));
    }
    listed.join(", ")
}

fn identity_mismatch(
    kind: &str,
    expected: &Identities,
    actual: Option<&Identities>,
    count: usize,
) -> String {
    let empty = BTreeSet::new();
    let actual = actual.map_or(&empty, |identities| identities);
    format!(
        "{kind} expected={} actual={count} missing=[{}] unexpected=[{}]",
        expected.len(),
        listed(expected.difference(actual)),
        listed(actual.difference(expected))
    )
}

fn compare(expected: &Keys, actual: &Keys, counts: &Counts) -> Result<(), HarnessError> {
    let kinds: BTreeSet<_> = expected
        .keys()
        .chain(
            actual
                .iter()
                .filter(|(_, ids)| !ids.is_empty())
                .map(|(kind, _)| kind),
        )
        .chain(
            counts
                .iter()
                .filter(|(_, count)| **count != 0)
                .map(|(kind, _)| kind),
        )
        .collect();
    let empty = BTreeSet::new();
    let mismatches: Vec<String> = kinds
        .iter()
        .filter_map(|&kind| {
            let keys = expected.get(kind).map_or(&empty, |keys| keys);
            let found = actual.get(kind);
            let count = counts.get(kind).map_or(0, |value| *value);
            println!("count {kind} expected={} actual={count}", keys.len());
            (found.map_or(&empty, |ids| ids) != keys || count != keys.len())
                .then(|| identity_mismatch(kind, keys, found, count))
        })
        .collect();
    if mismatches.is_empty() {
        return Ok(());
    }
    Err(HarnessError::invalid(format!(
        "seeded identities/counts differ for {} of {} kinds: {}",
        mismatches.len(),
        kinds.len(),
        mismatches.join("; ")
    )))
}

#[cfg(test)]
mod tests;
