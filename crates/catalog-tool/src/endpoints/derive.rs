use super::eval::Bindings;
use super::{Endpoint, Error, Fault, Outcome, RuleSet};
use serde_json::{Value, json};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceClass {
    Global,

    Regional,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DerivedEndpoint {
    Resolved {
        class: ServiceClass,

        url_template: String,

        signing_name: String,

        signing_region: String,
    },

    NeedsOverlay {
        parameter: String,
    },

    NeedsOverlayRouting {
        url_template: String,

        signing_name: String,

        signing_region: String,

        matches: usize,

        exceptions: usize,
    },

    Unsupported {
        reason: String,
    },
}

impl RuleSet {
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
