use super::derive::{DerivedEndpoint, signing_override};
use super::{CaseRun, Endpoint, Error};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedEndpoint {
    service: DerivedService,
    operations: BTreeMap<String, Option<DerivedService>>,
}

impl VerifiedEndpoint {
    pub fn endpoint(&self) -> &DerivedEndpoint {
        &self.service.endpoint
    }
    pub fn exceptions(&self) -> &[EndpointException] {
        &self.service.exceptions
    }
    /// Missing evidence for an explicitly modeled operation never falls back to
    /// the service route. Operations without static context retain that route,
    /// which is unresolved when only operation constants complete the rule set.
    pub fn operation_endpoint(&self, operation: &str) -> Option<&DerivedEndpoint> {
        match self.operations.get(operation) {
            Some(endpoint) => endpoint.as_ref().map(DerivedService::endpoint),
            None => Some(self.endpoint()),
        }
    }

    pub(super) fn bind_operation(&mut self, operation: String, endpoint: Option<Self>) {
        self.operations.insert(operation, endpoint.map(|endpoint| endpoint.service));
    }

    /// Removes the placeholder host label a route was compiled with. The
    /// runtime prepends a modeled host prefix itself, so a route leads with the
    /// label exactly when that prefix supplies it and mentions it nowhere else.
    pub(super) fn without_host_label(mut self, label: &str, prefixed: bool) -> Option<Self> {
        let lead = format!("https://{label}.");
        let strip = |url: &mut String| {
            if prefixed {
                *url = format!("https://{}", url.strip_prefix(&lead)?);
            }
            (!url.contains(label)).then_some(())
        };
        let DerivedEndpoint::Resolved { url_template, signing_name, signing_region, .. } =
            &mut self.service.endpoint
        else {
            return None;
        };
        strip(url_template)?;
        if signing_name.contains(label) || signing_region.contains(label) {
            return None;
        }
        for exception in &mut self.service.exceptions {
            strip(&mut exception.url)?;
            if exception.signing_name.contains(label) || exception.signing_region.contains(label) {
                return None;
            }
        }
        Some(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerificationFailure {
    FailedCases(usize),
    NoInScopeCases,
    NoEndpointEvidence,
    UnresolvedCandidate,
}

impl CaseRun<'_> {
    pub fn verified_endpoint(
        &self,
        signing_name: Option<&str>,
    ) -> Result<VerifiedEndpoint, VerificationFailure> {
        if self.failed() > 0 {
            return Err(VerificationFailure::FailedCases(self.failed()));
        }
        if self.in_scope() == 0 {
            return Err(VerificationFailure::NoInScopeCases);
        }
        if self.endpoints.is_empty() {
            return Err(VerificationFailure::NoEndpointEvidence);
        }
        let service =
            derive_run(self, signing_name).map_err(|_| VerificationFailure::UnresolvedCandidate)?;
        if matches!(&service.endpoint, DerivedEndpoint::Resolved { .. }) {
            Ok(VerifiedEndpoint { service, operations: BTreeMap::new() })
        } else {
            Err(VerificationFailure::UnresolvedCandidate)
        }
    }

    /// A service whose own route cannot resolve without operation constants
    /// carries only its operation routes; its service route stays unresolved.
    pub(super) fn operation_carrier(&self, signing_name: Option<&str>) -> Option<VerifiedEndpoint> {
        let service = derive_run(self, signing_name).ok()?;
        (!matches!(&service.endpoint, DerivedEndpoint::Resolved { .. }))
            .then(|| VerifiedEndpoint { service, operations: BTreeMap::new() })
    }
}

pub(super) fn derive_run(
    run: &CaseRun<'_>,
    signing_name: Option<&str>,
) -> Result<DerivedService, Error> {
    let endpoint = run.rules.derive(signing_name)?;
    let Some(name) = signing_name else {
        return Ok(DerivedService {
            endpoint,
            exceptions: Vec::new(),
        });
    };
    Ok(verify(endpoint, &run.endpoints, name))
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DerivedService {
    endpoint: DerivedEndpoint,
    exceptions: Vec<EndpointException>,
}

impl DerivedService {
    pub fn endpoint(&self) -> &DerivedEndpoint {
        &self.endpoint
    }
    pub fn exceptions(&self) -> &[EndpointException] {
        &self.exceptions
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct EndpointException {
    pub region: String,
    pub url: String,
    pub signing_name: String,
    pub signing_region: String,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct EndpointCase {
    region: Option<String>,
    url: String,
    name: Option<String>,
    signing_region: Option<String>,
}

impl EndpointCase {
    pub(super) fn new(endpoint: Endpoint, region: Option<&str>) -> Self {
        let name = signing_override(&endpoint, "signingName").map(str::to_owned);
        let signing_region = signing_override(&endpoint, "signingRegion").map(str::to_owned);
        Self {
            region: region.map(str::to_owned),
            url: endpoint.url,
            name,
            signing_region,
        }
    }

    fn region(&self) -> &str {
        let Some(region) = &self.region else {
            return "us-west-2";
        };
        region
    }

    fn name<'a>(&'a self, fallback: &'a str) -> &'a str {
        let Some(name) = &self.name else {
            return fallback;
        };
        name
    }

    fn signing_region(&self) -> &str {
        let Some(region) = &self.signing_region else {
            return self.region();
        };
        region
    }

    fn exception(&self, name: &str) -> EndpointException {
        let region = self.region_label();
        EndpointException {
            region: region.into(),
            url: self.url.clone(),
            signing_name: self.name(name).into(),
            signing_region: self.signing_region().into(),
        }
    }

    fn region_label(&self) -> &str {
        let Some(region) = &self.region else {
            return "-";
        };
        region
    }
}

#[derive(Default)]
struct Verification {
    matches: usize,
    mismatches: usize,
    exceptions: BTreeSet<EndpointException>,
}

fn verify(mut endpoint: DerivedEndpoint, cases: &[EndpointCase], name: &str) -> DerivedService {
    let mut evidence = Verification::default();
    if matches!(&endpoint, DerivedEndpoint::Resolved { .. }) {
        for case in cases {
            evidence.observe(&endpoint, case, name);
        }
        if evidence.matches == 0 || evidence.mismatches > evidence.matches {
            endpoint = unconfirmed(endpoint, evidence.matches, evidence.mismatches);
        }
    }
    DerivedService {
        endpoint,
        exceptions: evidence.exceptions.into_iter().collect(),
    }
}

impl Verification {
    fn observe(&mut self, endpoint: &DerivedEndpoint, case: &EndpointCase, name: &str) {
        if reproduces(endpoint, case, name) {
            self.matches = self.matches.saturating_add(1);
        } else {
            self.mismatches = self.mismatches.saturating_add(1);
            self.exceptions.insert(case.exception(name));
        }
    }
}

fn reproduces(endpoint: &DerivedEndpoint, case: &EndpointCase, name: &str) -> bool {
    let DerivedEndpoint::Resolved {
        url_template,
        signing_name,
        signing_region,
        ..
    } = endpoint
    else {
        return false;
    };
    expanded_matches(url_template, case.region(), &case.url)
        && signing_name.as_str() == case.name(name)
        && expanded_matches(signing_region, case.region(), case.signing_region())
}

fn expanded_matches(template: &str, region: &str, actual: &str) -> bool {
    template
        .split("{region}")
        .enumerate()
        .try_fold(actual, |remaining, (index, part)| {
            let remaining = if index == 0 {
                remaining
            } else {
                remaining.strip_prefix(region)?
            };
            remaining.strip_prefix(part)
        })
        .is_some_and(str::is_empty)
}

fn unconfirmed(endpoint: DerivedEndpoint, matches: usize, exceptions: usize) -> DerivedEndpoint {
    match endpoint {
        DerivedEndpoint::Resolved {
            url_template,
            signing_name,
            signing_region,
            ..
        } => DerivedEndpoint::NeedsOverlayRouting {
            url_template,
            signing_name,
            signing_region,
            matches,
            exceptions,
        },
        other => other,
    }
}
