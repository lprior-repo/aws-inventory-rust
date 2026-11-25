use crate::HarnessError;
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::{self, Write},
    path::Path,
};

pub(super) type Plans = BTreeMap<String, BTreeSet<String>>;

#[derive(Clone, Copy, Default)]
pub(in crate::e2e) struct Selection<'a> {
    pub requested: &'a [String],
    pub executed: &'a [String],
}

impl Selection<'_> {
    fn includes(self, operation: &str) -> bool {
        self.requested.is_empty()
            || self.executed.iter().any(|selector| {
                if selector.contains(':') {
                    selector == operation
                } else {
                    operation
                        .split_once(':')
                        .is_some_and(|(service, _)| selector == service)
                }
            })
    }
}

#[derive(Clone, Copy)]
pub(super) enum Probe<'a> {
    Complete,
    Unsupported,
    Failed(&'a str),
}

#[derive(Default)]
struct Probes {
    complete: bool,
    unsupported: bool,
    failed: Option<String>,
}

#[derive(Default)]
pub(super) struct Evidence {
    operations: BTreeMap<String, Probes>,
    observed: Plans,
    rejected: BTreeMap<String, String>,
}

impl Evidence {
    pub(super) fn probe(&mut self, operation: String, probe: Probe<'_>) {
        let outcomes = self.operations.entry(operation).or_default();
        match probe {
            Probe::Complete => outcomes.complete = true,
            Probe::Unsupported => outcomes.unsupported = true,
            Probe::Failed(code) => {
                if outcomes
                    .failed
                    .as_deref()
                    .is_none_or(|previous| code < previous)
                {
                    outcomes.failed = Some(code.into());
                }
            }
        }
    }

    pub(super) fn observe(&mut self, kind: impl Into<String>, operation: impl Into<String>) {
        self.observed
            .entry(kind.into())
            .or_default()
            .insert(operation.into());
    }

    pub(super) fn reject(&mut self, kind: &str, code: &str) {
        let previous = self.rejected.get(kind);
        if previous.is_none_or(|value| code < value.as_str()) {
            self.rejected.insert(kind.into(), code.into());
        }
    }

    fn status(
        &self,
        kind: &str,
        operations: &BTreeSet<String>,
        selection: Selection<'_>,
    ) -> Status<'_> {
        let probes = || {
            operations
                .iter()
                .filter_map(|operation| self.operations.get(operation))
        };
        if let Some(operation) = self.observed.get(kind).and_then(|found| {
            found
                .iter()
                .find(|operation| operations.contains(*operation))
        }) {
            return Status::Verified(operation);
        }
        if let Some(code) = probes().filter_map(|probe| probe.failed.as_deref()).min() {
            return Status::Failed(code);
        }
        if probes().next().is_none() && !operations.iter().any(|op| selection.includes(op)) {
            return Status::NotRun;
        }
        if let Some(code) = self.rejected.get(kind) {
            return Status::Rejected(code);
        }
        if !probes().any(|probe| probe.complete) && probes().any(|probe| probe.unsupported) {
            return Status::Unsupported;
        }
        Status::Empty
    }
}

#[derive(Clone, Copy)]
enum Status<'a> {
    Verified(&'a str),
    Unsupported,
    Empty,
    Rejected(&'a str),
    Failed(&'a str),
    NotRun,
}

impl fmt::Display for Status<'_> {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Verified(operation) => write!(output, "emulator-verified\t{operation}"),
            Self::Unsupported => output.write_str("implemented\tunsupported"),
            Self::Empty => output.write_str("implemented\tempty: nothing seeded"),
            Self::Rejected(code) => write!(output, "implemented\tseed rejected: {code}"),
            Self::Failed(code) => write!(output, "implemented\tfailed: {code}"),
            Self::NotRun => output.write_str("implemented\tnot run"),
        }
    }
}

pub(super) fn publish(
    path: &Path,
    plans: &Plans,
    evidence: &Evidence,
    selection: Selection<'_>,
) -> Result<(), HarnessError> {
    std::fs::write(path, render(plans, evidence, selection)?)?;
    require_kinds(path, plans)
}

fn render(
    plans: &Plans,
    evidence: &Evidence,
    selection: Selection<'_>,
) -> Result<String, HarnessError> {
    let rows: Vec<_> = plans
        .iter()
        .map(|(kind, operations)| (kind, evidence.status(kind, operations, selection)))
        .collect();
    let verified = rows
        .iter()
        .filter(|(_, status)| matches!(status, Status::Verified(_)))
        .count();
    let implemented = rows
        .len()
        .checked_sub(verified)
        .ok_or_else(|| HarnessError::invalid("invalid coverage-status totals"))?;
    let mut result = format!(
        "totals\tcallable={}\temulator_verified={verified}\timplemented={implemented}",
        plans.len(),
    );
    if !selection.requested.is_empty() {
        let selected: BTreeSet<_> = selection.requested.iter().map(String::as_str).collect();
        write!(
            result,
            "\tpartial selected={}",
            selected.into_iter().collect::<Vec<_>>().join(",")
        )
        .map_err(|_| HarnessError::invalid("cannot format coverage-status report"))?;
    }
    result.push('\n');
    rows.iter()
        .try_for_each(|(kind, status)| writeln!(result, "{kind}\t{status}"))
        .map_err(|_| HarnessError::invalid("cannot format coverage-status report"))?;
    Ok(result)
}

fn require_kinds(path: &Path, plans: &Plans) -> Result<(), HarnessError> {
    let text = super::read_text(path)?;
    let mut lines = text.lines();
    if !lines
        .next()
        .is_some_and(|line| line.starts_with("totals\t"))
    {
        return Err(HarnessError::invalid(
            "coverage-status totals line is missing",
        ));
    }
    let kinds: BTreeSet<_> = lines.try_fold(BTreeSet::new(), |mut kinds, line| {
        let (kind, _) = line
            .split_once('\t')
            .filter(|(kind, _)| !kind.is_empty())
            .ok_or_else(|| HarnessError::invalid("malformed coverage-status row"))?;
        if !kinds.insert(kind) {
            return Err(HarnessError::invalid(format!(
                "duplicate coverage-status kind {kind}"
            )));
        }
        Ok(kinds)
    })?;
    let missing: Vec<_> = plans
        .keys()
        .map(String::as_str)
        .filter(|kind| !kinds.contains(kind))
        .take(5)
        .collect();
    let extra: Vec<_> = kinds
        .into_iter()
        .filter(|kind| !plans.contains_key(*kind))
        .take(5)
        .collect();
    if missing.is_empty() && extra.is_empty() {
        return Ok(());
    }
    Err(HarnessError::invalid(format!(
        "coverage-status kind set differs: missing=[{}] extra=[{}]",
        missing.join(", "),
        extra.join(", ")
    )))
}

#[cfg(test)]
#[path = "e2e/verify/coverage/status_tests.rs"]
mod tests;
