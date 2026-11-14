use super::{Error, Fault, Outcome, RuleSet, partition};
use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap;

/// One model-owned endpoint case; operation binding is explicitly out of scope.
#[derive(Debug, Deserialize)]
pub struct Case {
    /// Explicit endpoint parameters, before applying defaults.
    #[serde(default)]
    pub params: BTreeMap<String, Value>,
    /// Exact expected endpoint or modeled error.
    pub expect: Outcome,
    /// Operation inputs or builtins requiring a separate binding layer.
    #[serde(default, rename = "operationInputs")]
    pub operation_inputs: Vec<Value>,
}

/// Exclusive buckets, chosen in declaration order when several exclusions apply.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SkipReason {
    /// Requires operation input or builtin binding.
    OperationInputs,
    /// Region does not match the commercial partition table.
    OtherPartition,
    /// FIPS is enabled.
    Fips,
    /// Dual-stack is enabled.
    DualStack,
    /// A custom endpoint is supplied.
    CustomEndpoint,
    /// Parameters other than Region and the two disabled flags are supplied.
    OtherParameters,
    /// A baseline case takes a branch requiring a request-time function.
    RequestTimeFunction,
}

/// Deterministic counts; preclassified skipped inputs are never evaluated.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct CaseCounts {
    /// Number of cases meeting the exact baseline scope.
    pub in_scope: usize,
    /// Baseline cases matching their expectations.
    pub passed: usize,
    /// Baseline cases failing or encountering interpreter errors.
    pub failed: usize,
    /// Exclusive exclusion buckets.
    pub skipped: BTreeMap<SkipReason, usize>,
    /// Baseline inputs whose taken branch needs an unsupported function.
    pub unsupported: Vec<UnsupportedCase>,
    pub(super) endpoints: Vec<super::exceptions::EndpointCase>,
}

/// An explicit diagnostic retaining the model-owned zero-based test index.
#[derive(Debug, PartialEq, Eq)]
pub struct UnsupportedCase {
    /// Index in `endpointTests.testCases`.
    pub index: usize,
    /// Unsupported function reached during evaluation.
    pub name: String,
}

/// Classify explicit inputs only, never defaults or expected output.
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
    /// Execute baseline cases only; record lazy unsupported calls as request-time skips.
    ///
    /// # Errors
    /// Rejects more than 20,000 cases; interpreter errors count as case failures.
    pub fn run_cases(&self, cases: &[Case]) -> Result<CaseCounts, Error> {
        if cases.len() > 20_000 {
            return Err(self.error(Fault::Bound("test cases")));
        }
        Ok(cases
            .iter()
            .enumerate()
            .fold(CaseCounts::default(), |mut totals, (index, case)| {
                totals.record(self, index, case);
                totals
            }))
    }
}

impl CaseCounts {
    fn record(&mut self, rules: &RuleSet, index: usize, case: &Case) {
        if let Some(reason) = classify_case(case) {
            self.skip(reason);
            return;
        }
        match rules.evaluate(&case.params) {
            Ok(Outcome::UnsupportedFunction { name }) => {
                self.skip(SkipReason::RequestTimeFunction);
                self.unsupported.push(UnsupportedCase { index, name });
            }
            actual => {
                let matched = actual.as_ref().is_ok_and(|value| value == &case.expect);
                self.baseline(matched);
                if matched {
                    self.remember(rules, case, actual);
                }
            }
        }
    }

    fn skip(&mut self, reason: SkipReason) {
        let count = self.skipped.entry(reason).or_default();
        *count = count.saturating_add(1);
    }

    pub(super) fn add(&mut self, other: Self) {
        self.in_scope = self.in_scope.saturating_add(other.in_scope);
        self.passed = self.passed.saturating_add(other.passed);
        self.failed = self.failed.saturating_add(other.failed);
        other.skipped.into_iter().for_each(|(reason, n)| {
            let count = self.skipped.entry(reason).or_default();
            *count = count.saturating_add(n);
        });
        self.unsupported.extend(other.unsupported);
    }
}

impl CaseCounts {
    fn remember(&mut self, rules: &RuleSet, case: &Case, actual: Result<Outcome, Error>) {
        let Ok(Outcome::Endpoint(endpoint)) = actual else {
            return;
        };
        let region = case
            .params
            .get("Region")
            .or_else(|| {
                rules
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
        _ => true,
    })
}

impl CaseCounts {
    fn baseline(&mut self, matched: bool) {
        self.in_scope = self.in_scope.saturating_add(1);
        if matched {
            self.passed = self.passed.saturating_add(1);
        } else {
            self.failed = self.failed.saturating_add(1);
        }
    }
}
