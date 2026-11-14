use super::derive::signing_override;
use super::{CaseCounts, DerivedEndpoint, Endpoint, Error, RuleSet};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DerivedService {
    pub endpoint: DerivedEndpoint,

    pub exceptions: Vec<EndpointException>,
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

impl CaseCounts {
    pub fn derive(
        &self,
        rules: &RuleSet,
        signing_name: Option<&str>,
    ) -> Result<DerivedService, Error> {
        let endpoint = rules.derive(signing_name)?;
        let Some(name) = signing_name else {
            return Ok(DerivedService {
                endpoint,
                exceptions: Vec::new(),
            });
        };
        Ok(verify(endpoint, &self.endpoints, name))
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
