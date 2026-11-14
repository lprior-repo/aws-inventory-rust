use super::eval::Bindings;
use super::{Endpoint, Error, Fault, Outcome, RuleSet};
use serde_json::{Value, json};

/// Scope of a successfully derived commercial endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceClass {
    /// One URL and signing region across both probe regions.
    Global,
    /// Region-sensitive URL or signing region.
    Regional,
}

/// Derived catalog inputs, or the explicit reason no template can be produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DerivedEndpoint {
    /// A resolved endpoint with modeled SigV4 overrides applied.
    Resolved {
        /// Global when both URL and signing region agree across the two probes.
        class: ServiceClass,
        /// URL varies by `{region}` only when the two probe URLs differ.
        url_template: String,
        /// Modeled signing name, falling back to the service's SigV4 trait.
        signing_name: String,
        /// Fixed region or `{region}`.
        signing_region: String,
    },
    /// Region-only evaluation requires a request-specific parameter.
    NeedsOverlay { parameter: String },
    /// Published cases do not sufficiently confirm the candidate template.
    NeedsOverlayRouting {
        /// Candidate URL retained for overlay review.
        url_template: String,
        /// Candidate signing name.
        signing_name: String,
        /// Candidate independently derived signing-region template.
        signing_region: String,
        /// Passing endpoint cases reproduced by the candidate.
        matches: usize,
        /// Contradicting passing cases, before deduplication.
        exceptions: usize,
    },
    /// The rule error does not name a missing input, or SigV4 is unavailable.
    Unsupported { reason: String },
}

impl RuleSet {
    /// Derive a candidate using two commercial regions, with default options off.
    /// Use `CaseCounts::derive` to verify it against passing published cases.
    ///
    /// # Errors
    /// Returns interpreter failures; modeled errors become explicit classifications.
    pub fn derive(&self, signing_name: Option<&str>) -> Result<DerivedEndpoint, Error> {
        let Some(name) = signing_name else {
            return Ok(DerivedEndpoint::Unsupported {
                reason: "no SigV4 trait".into(),
            });
        };
        let first = self.probe("us-west-2")?;
        let second = self.probe("eu-west-1")?;
        Ok(self.derive_outcomes((first, second), name))
    }

    fn probe(&self, region: &str) -> Result<Outcome, Error> {
        let input = self.region_inputs(region);
        match self.evaluate(&input) {
            Err(Error::Model {
                kind: Fault::MissingParameter(name),
                ..
            }) => Ok(Outcome::Error(format!("Missing {name}"))),
            result => result,
        }
    }

    fn overlay(&self, message: String) -> DerivedEndpoint {
        let normalized = normalize(&message);
        let missing = self.document.parameters.keys().find(|name| {
            !matches!(
                name.as_str(),
                "Region" | "UseFIPS" | "UseDualStack" | "Endpoint"
            ) && normalized.contains(&normalize(name))
        });
        let Some(name) = missing else {
            return DerivedEndpoint::Unsupported { reason: message };
        };
        DerivedEndpoint::NeedsOverlay {
            parameter: name.clone(),
        }
    }

    fn region_inputs(&self, region: &str) -> Bindings {
        [("Region".into(), json!(region))]
            .into_iter()
            .chain(
                ["UseFIPS", "UseDualStack"]
                    .into_iter()
                    .filter(|name| self.document.parameters.contains_key(*name))
                    .map(|name| (name.into(), Value::Bool(false))),
            )
            .filter(|(name, _)| self.document.parameters.contains_key(name))
            .collect()
    }

    fn derive_outcomes(&self, outcomes: (Outcome, Outcome), name: &str) -> DerivedEndpoint {
        match outcomes {
            (Outcome::Endpoint(a), Outcome::Endpoint(b)) => resolved(&a, &b, name),
            (Outcome::UnsupportedFunction { name }, _)
            | (_, Outcome::UnsupportedFunction { name }) => DerivedEndpoint::NeedsOverlay {
                parameter: format!("function:{name}"),
            },
            (Outcome::Error(message), _) | (_, Outcome::Error(message)) => self.overlay(message),
        }
    }
}

fn normalize(value: &str) -> String {
    value
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .flat_map(char::to_lowercase)
        .collect()
}

pub(super) fn signing_override<'a>(endpoint: &'a Endpoint, key: &str) -> Option<&'a str> {
    let scheme = endpoint
        .properties
        .get("authSchemes")
        .and_then(Value::as_array)
        .and_then(|schemes| {
            schemes
                .iter()
                .find(|v| v.get("name").and_then(Value::as_str) == Some("sigv4"))
        });
    scheme.and_then(|v| v.get(key)).and_then(Value::as_str)
}

fn resolved(a: &Endpoint, b: &Endpoint, name: &str) -> DerivedEndpoint {
    let region = signing(a, "signingRegion", "us-west-2");
    let fixed_region = region == signing(b, "signingRegion", "eu-west-1");
    let class = if a.url == b.url && fixed_region {
        ServiceClass::Global
    } else {
        ServiceClass::Regional
    };
    DerivedEndpoint::Resolved {
        class,
        url_template: if a.url == b.url {
            a.url.clone()
        } else {
            regional_template(&a.url)
        },
        signing_name: signing(a, "signingName", name).into(),
        signing_region: if fixed_region {
            region.into()
        } else {
            regional_template(region)
        },
    }
}

fn regional_template(value: &str) -> String {
    value.replace("us-west-2", "{region}")
}

fn signing<'a>(endpoint: &'a Endpoint, key: &str, fallback: &'a str) -> &'a str {
    let Some(value) = signing_override(endpoint, key) else {
        return fallback;
    };
    value
}
