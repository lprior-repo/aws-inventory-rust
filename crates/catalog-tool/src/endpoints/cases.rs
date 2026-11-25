use super::{Error, Fault, Outcome, RuleSet, partition};
use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Deserialize)]
pub struct Case {
    #[serde(default)]
    pub params: BTreeMap<String, Value>,

    pub expect: Outcome,

    #[serde(default, rename = "operationInputs")]
    pub operation_inputs: Vec<Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SkipReason {
    OperationInputs,

    OtherPartition,

    Fips,

    DualStack,

    CustomEndpoint,

    OtherParameters,

    RequestTimeFunction,
}

#[derive(Debug)]
pub struct CaseRun<'a> {
    pub(super) rules: &'a RuleSet,
    in_scope: usize,
    passed: usize,
    failed: usize,
    skipped: BTreeMap<SkipReason, usize>,
    unsupported: Vec<UnsupportedCase>,
    failures: Vec<CaseFailure>,
    pub(super) endpoints: Vec<super::exceptions::EndpointCase>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaseFailure {
    index: usize,
    reason: String,
}

impl CaseFailure {
    pub fn index(&self) -> usize {
        self.index
    }
    pub fn reason(&self) -> &str {
        &self.reason
    }
}

impl<'a> CaseRun<'a> {
    pub fn in_scope(&self) -> usize {
        self.in_scope
    }
    pub fn passed(&self) -> usize {
        self.passed
    }
    pub fn failed(&self) -> usize {
        self.failed
    }
    pub fn skipped(&self) -> &BTreeMap<SkipReason, usize> {
        &self.skipped
    }
    pub fn unsupported(&self) -> &[UnsupportedCase] {
        &self.unsupported
    }
    pub fn failures(&self) -> &[CaseFailure] {
        &self.failures
    }

    pub fn derive(&self, signing_name: Option<&str>) -> Result<super::DerivedService, Error> {
        super::exceptions::derive_run(self, signing_name)
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub(super) struct CaseCounts {
    pub(super) in_scope: usize,
    pub(super) passed: usize,
    pub(super) failed: usize,
    pub(super) skipped: BTreeMap<SkipReason, usize>,
}

impl CaseRun<'_> {
    pub(super) fn totals(&self) -> CaseCounts {
        CaseCounts {
            in_scope: self.in_scope,
            passed: self.passed,
            failed: self.failed,
            skipped: self.skipped.clone(),
        }
    }
}

impl CaseCounts {
    pub(super) fn add(&mut self, other: Self) {
        self.in_scope = self.in_scope.saturating_add(other.in_scope);
        self.passed = self.passed.saturating_add(other.passed);
        self.failed = self.failed.saturating_add(other.failed);
        other.skipped.into_iter().for_each(|(reason, n)| {
            let count = self.skipped.entry(reason).or_default();
            *count = count.saturating_add(n);
        });
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnsupportedCase {
    pub index: usize,
    pub name: String,
}

#[must_use]
pub fn classify_case(case: &Case) -> Option<SkipReason> {
    use SkipReason as S;
    [
        (!case.operation_inputs.is_empty(), S::OperationInputs),
        (outside_partition(case), S::OtherPartition),
        (
            case.params.get("UseFIPS") == Some(&Value::Bool(true)),
            S::Fips,
        ),
        (
            case.params.get("UseDualStack") == Some(&Value::Bool(true)),
            S::DualStack,
        ),
        (case.params.contains_key("Endpoint"), S::CustomEndpoint),
        (other_parameters(case), S::OtherParameters),
    ]
    .into_iter()
    .find_map(|(skip, reason)| skip.then_some(reason))
}

impl RuleSet {
    pub fn run_cases(&self, cases: &[Case]) -> Result<CaseRun<'_>, Error> {
        if cases.len() > 20_000 {
            return Err(self.error(Fault::Bound("test cases")));
        }
        let mut run = CaseRun {
            rules: self,
            in_scope: 0,
            passed: 0,
            failed: 0,
            skipped: BTreeMap::new(),
            unsupported: Vec::new(),
            failures: Vec::new(),
            endpoints: Vec::new(),
        };
        cases
            .iter()
            .enumerate()
            .for_each(|(index, case)| run.record(index, case));
        Ok(run)
    }
}

impl CaseRun<'_> {
    fn record(&mut self, index: usize, case: &Case) {
        if let Some(reason) = classify_case(case) {
            self.skip(reason);
            return;
        }
        match self.rules.evaluate(&case.params) {
            Ok(Outcome::UnsupportedFunction { name }) => {
                self.skip(SkipReason::RequestTimeFunction);
                self.unsupported.push(UnsupportedCase { index, name });
            }
            actual => self.record_result(index, case, actual),
        }
    }

    fn record_result(&mut self, index: usize, case: &Case, actual: Result<Outcome, Error>) {
        let matched = actual.as_ref().is_ok_and(|value| value == &case.expect);
        self.in_scope = self.in_scope.saturating_add(1);
        if matched {
            self.passed = self.passed.saturating_add(1);
            self.remember(case, actual);
        } else {
            self.failed = self.failed.saturating_add(1);
            self.failures.push(CaseFailure {
                index,
                reason: match actual {
                    Err(error) => diagnostic_reason(error),
                    Ok(Outcome::Error(message)) => format!("modeled error: {message}"),
                    _ => "actual outcome differs from expected outcome".into(),
                },
            });
        }
    }

    fn skip(&mut self, reason: SkipReason) {
        let count = self.skipped.entry(reason).or_default();
        *count = count.saturating_add(1);
    }

    fn remember(&mut self, case: &Case, actual: Result<Outcome, Error>) {
        let Ok(Outcome::Endpoint(endpoint)) = actual else {
            return;
        };
        let region = case
            .params
            .get("Region")
            .or_else(|| {
                self.rules
                    .document
                    .parameters
                    .get("Region")
                    .and_then(|p| p.default.as_ref())
            })
            .and_then(Value::as_str);
        self.endpoints
            .push(super::exceptions::EndpointCase::new(endpoint, region));
    }
}

fn diagnostic_reason(error: Error) -> String {
    match error {
        Error::Model { kind, .. } => kind.to_string(),
        other => other.to_string(),
    }
}
fn outside_partition(case: &Case) -> bool {
    case.params
        .get("Region")
        .and_then(Value::as_str)
        .is_some_and(|region| !partition::contains(region))
}

fn other_parameters(case: &Case) -> bool {
    case.params.iter().any(|(name, value)| match name.as_str() {
        "Region" => !value.is_string(),
        "UseFIPS" | "UseDualStack" => value != &Value::Bool(false),
        "AccountIdEndpointMode" => !matches!(value.as_str(), Some("disabled" | "preferred")),
        _ => true,
    })
}
