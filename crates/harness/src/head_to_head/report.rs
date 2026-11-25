use crate::HarnessError;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

pub type Counts = BTreeMap<String, u64>;

#[derive(Debug, PartialEq)]
pub enum Outcome {
    Success {
        seconds: f64,
        counts: Counts,
        api_calls: Option<u64>,
        left_processes: bool,
    },
    Failed {
        reason: String,
    },
}

pub struct Sample {
    pub tool: String,
    pub run: usize,
    pub outcome: Outcome,
}

pub fn rotation<T>(tools: &[T], run: usize) -> impl Iterator<Item = &T> {
    run.checked_rem(tools.len())
        .into_iter()
        .flat_map(move |offset| tools.iter().skip(offset).chain(tools.iter().take(offset)))
}

pub fn median(values: &[f64]) -> Option<f64> {
    if values.is_empty()
        || values
            .iter()
            .any(|value| !value.is_finite() || *value < 0.0)
    {
        return None;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let middle = sorted.len() / 2;
    let upper = *sorted.get(middle)?;
    if sorted.len().checked_rem(2) == Some(1) {
        return Some(upper);
    }
    let lower = *sorted.get(middle.checked_sub(1)?)?;
    Some(lower / 2.0 + upper / 2.0)
}

pub fn common_kinds(samples: &[Sample], names: &[String]) -> BTreeSet<String> {
    if names.is_empty()
        || names
            .iter()
            .any(|name| !samples.iter().any(|sample| &sample.tool == name))
        || samples
            .iter()
            .any(|sample| matches!(sample.outcome, Outcome::Failed { .. }))
    {
        return BTreeSet::new();
    }
    let mut kinds = samples.iter().filter_map(|sample| match &sample.outcome {
        Outcome::Success { counts, .. } => Some(counts.keys().cloned().collect::<BTreeSet<_>>()),
        Outcome::Failed { .. } => None,
    });
    let Some(first) = kinds.next() else {
        return BTreeSet::new();
    };
    kinds.fold(first, |common, kinds| {
        common.intersection(&kinds).cloned().collect()
    })
}

fn not_compared_kinds(samples: &[Sample], common: &BTreeSet<String>) -> BTreeSet<String> {
    samples
        .iter()
        .filter_map(|sample| match &sample.outcome {
            Outcome::Success { counts, .. } => Some(counts.keys()),
            Outcome::Failed { .. } => None,
        })
        .flatten()
        .filter(|kind| !common.contains(*kind))
        .cloned()
        .collect()
}

pub struct Evidence {
    pub versions: Value,
    pub dataset: Value,
    pub seeded_counts: Counts,
    pub machine: Value,
    pub services: Vec<String>,
    pub scopes: BTreeMap<String, Vec<String>>,
}

fn scopes_agree(names: &[String], evidence: &Evidence) -> bool {
    let expected: BTreeSet<_> = evidence.services.iter().collect();
    !expected.is_empty()
        && names.iter().all(|name| {
            evidence
                .scopes
                .get(name)
                .is_some_and(|services| services.iter().collect::<BTreeSet<_>>() == expected)
        })
}

pub fn build(names: &[String], samples: &[Sample], evidence: &Evidence) -> Value {
    let checked: Vec<_> = samples
        .iter()
        .map(|sample| (sample, sample_seed_failure(sample, &evidence.seeded_counts)))
        .collect();
    let common = match checked.iter().all(|(_, failure)| failure.is_none()) {
        true => common_kinds(samples, names),
        false => BTreeSet::new(),
    };
    let equal_counts = counts_agree(samples, &common);
    let not_compared = not_compared_kinds(samples, &common);
    let equal_scope = scopes_agree(names, evidence);
    let comparison_ready = equal_counts == Some(true) && not_compared.is_empty() && equal_scope;
    let tools: Vec<_> = names
        .iter()
        .map(|name| {
            let selected: Vec<_> = checked
                .iter()
                .filter(|(sample, _)| &sample.tool == name)
                .collect();
            let times: Vec<_> = selected
                .iter()
                .filter_map(|(sample, failure)| match (&sample.outcome, failure) {
                    (Outcome::Success { seconds, .. }, None) => Some(*seconds),
                    _ => None,
                })
                .collect();
            let runs: Vec<_> = selected
                .iter()
                .map(|(sample, failure)| sample_value(sample, &common, failure.as_deref()))
                .collect();
            let min = times.iter().copied().reduce(f64::min);
            let max = times.iter().copied().reduce(f64::max);
            let valid = selected
                .iter()
                .all(|(sample, failure)| failure.is_none() && matches!(sample.outcome, Outcome::Success { .. }));
            let timing = if valid && comparison_ready {
                json!({ "median_seconds": median(&times), "min_seconds": min, "max_seconds": max,
                "spread_seconds": min.zip(max).map(|(min, max)| max - min) })
            } else {
                Value::Null
            };
            json!({ "name": name, "runs": runs, "comparison_timing": timing, "services": evidence.scopes.get(name) })
        })
        .collect();
    let execution_order: Vec<_> = samples
        .iter()
        .map(|sample| json!({ "run": sample.run, "tool": sample.tool }))
        .collect();
    json!({ "format": 1, "versions": evidence.versions, "dataset": evidence.dataset, "machine": evidence.machine,
        "services": evidence.services, "equal_services": equal_scope,
        "common_kinds": common, "equal_common_counts": equal_counts,
        "not_compared_kinds": not_compared,
        "comparison_ready": comparison_ready, "tools": tools, "execution_order": execution_order,
        "timing_scope": "whole collection command; not normalized by kind",
        "comparison_policy": "failed runs, unequal observed count maps or unequal service scopes defer comparison to J3; no failed run is timed" })
}

fn sample_seed_failure(sample: &Sample, seeded: &Counts) -> Option<String> {
    match &sample.outcome {
        Outcome::Success { counts, .. } => seeded_count_failure(counts, seeded),
        Outcome::Failed { .. } => None,
    }
}

pub fn seeded_count_failure(counts: &Counts, seeded: &Counts) -> Option<String> {
    seeded.iter().find_map(|(kind, expected)| {
        let extras_allowed = matches!(kind.as_str(), "AWS::EC2::VPC" | "AWS::EC2::Subnet");
        let relation = if extras_allowed {
            "at least"
        } else {
            "exactly"
        };
        let observed = counts.get(kind).copied();
        let valid = observed.is_some_and(|actual| {
            if extras_allowed {
                actual >= *expected
            } else {
                actual == *expected
            }
        });
        if valid {
            return None;
        }
        let observed =
            observed.map_or_else(|| "missing".into(), |count| format!("observed {count}"));
        Some(format!(
            "seeded kind {kind}: {observed}, requires {relation} {expected}"
        ))
    })
}

fn sample_value(sample: &Sample, common: &BTreeSet<String>, failure: Option<&str>) -> Value {
    match &sample.outcome {
        Outcome::Failed { reason } => {
            json!({ "run": sample.run, "status": "failed", "reason": reason })
        }
        Outcome::Success {
            seconds,
            counts,
            api_calls,
            left_processes,
        } => {
            let common_counts: Counts = counts
                .iter()
                .filter(|(kind, _)| common.contains(*kind))
                .map(|(kind, count)| (kind.clone(), *count))
                .collect();
            let mut value = json!({ "run": sample.run, "status": "success", "wall_seconds": seconds,
                "api_calls": api_calls, "common_counts": common_counts, "observed_counts": counts,
                "left_processes_running": left_processes,
                "process_cleanup": left_processes.then_some("left processes running after exit; killed") });
            if let (Some(reason), Some(object)) = (failure, value.as_object_mut()) {
                object.insert("status".into(), json!("failed"));
                object.insert("reason".into(), json!(reason));
            }
            value
        }
    }
}

pub fn write(path: &std::path::Path, report: &Value) -> Result<(), HarnessError> {
    std::fs::write(path, serde_json::to_vec_pretty(report)?)?;
    Ok(())
}
fn counts_agree(samples: &[Sample], common: &BTreeSet<String>) -> Option<bool> {
    if common.is_empty() {
        return None;
    }
    Some(common.iter().all(|kind| {
        let counts: Vec<_> = samples
            .iter()
            .filter_map(|sample| match &sample.outcome {
                Outcome::Success { counts, .. } => counts.get(kind),
                Outcome::Failed { .. } => None,
            })
            .collect();
        counts
            .first()
            .is_some_and(|first| counts.iter().all(|count| count == first))
    }))
}
