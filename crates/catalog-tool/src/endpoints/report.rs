use super::report_text::render;
use super::{Case, CaseCounts, DerivedEndpoint, EndpointException, Error, Fault, RuleSet};
use serde::Deserialize;
use serde_json::Value;
use std::{collections::BTreeMap, path::Path};

#[derive(Deserialize)]
struct Model {
    shapes: BTreeMap<String, Shape>,
}
#[derive(Deserialize)]
struct Shape {
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    traits: Traits,
}
#[derive(Default, Deserialize)]
struct Traits {
    #[serde(rename = "smithy.rules#endpointRuleSet")]
    rules: Option<Value>,
    #[serde(rename = "smithy.rules#endpointTests")]
    tests: Option<Tests>,
    #[serde(rename = "aws.auth#sigv4")]
    signing: Option<Signing>,
}
#[derive(Deserialize)]
struct Tests {
    #[serde(rename = "testCases")]
    cases: Vec<Case>,
}
#[derive(Deserialize)]
struct Signing {
    name: String,
}
pub(super) struct Service {
    pub(super) name: String,
    pub(super) endpoint: DerivedEndpoint,
    pub(super) unsupported: Vec<super::UnsupportedCase>,
    pub(super) exceptions: Vec<EndpointException>,
}

/// Read models one at a time and return a complete, path-independent endpoint report.
///
/// # Errors
/// Returns named schema/input errors or bound violations; case mismatches are counted.
/// Discovery is capped at 1,024 models; individual reads use Smithy's 64 MiB limit.
pub fn build_report(source: &Path) -> Result<String, Error> {
    let models = source.join("models");
    let files = crate::smithy::find_model_files(&models)?;
    if files.is_empty() {
        return Err(crate::smithy::Error::NoModels.into());
    }
    if files.len() > 1024 {
        return Err(model_error(&models, Fault::Bound("services")));
    }
    let (counts, mut services) = load_services(&files, &models)?;
    services.sort_by(|a, b| a.name.cmp(&b.name));
    render(&counts, &services)
}

fn read_service(file: &Path, models: &Path) -> Result<(Service, CaseCounts), Error> {
    let traits = read_traits(file)?;
    let value = traits
        .rules
        .ok_or_else(|| malformed(file, "missing rules"))?;
    let rules = RuleSet::parse(file, &value)?;
    let tests = traits
        .tests
        .ok_or_else(|| malformed(file, "missing tests"))?;
    let mut counts = rules.run_cases(&tests.cases)?;
    let derived = counts.derive(&rules, traits.signing.as_ref().map(|s| s.name.as_str()))?;
    let name = service_name(file, models)?;
    let unsupported = std::mem::take(&mut counts.unsupported);
    Ok((
        Service {
            name,
            endpoint: derived.endpoint,
            exceptions: derived.exceptions,
            unsupported,
        },
        counts,
    ))
}

fn model_error(file: &Path, kind: Fault) -> Error {
    Error::Model {
        file: file.to_path_buf(),
        kind,
    }
}

fn load_services(
    files: &[std::path::PathBuf],
    models: &Path,
) -> Result<(CaseCounts, Vec<Service>), Error> {
    files.iter().try_fold(
        (CaseCounts::default(), Vec::new()),
        |(mut counts, mut rows), file| {
            let (service, cases) = read_service(file, models)?;
            counts.add(cases);
            rows.push(service);
            Ok((counts, rows))
        },
    )
}

fn read_traits(file: &Path) -> Result<Traits, Error> {
    let model: Model = crate::smithy::read_json(file)?;
    let mut services = model.shapes.into_values().filter(|s| s.kind == "service");
    let shape = services
        .next()
        .ok_or_else(|| malformed(file, "no service"))?;
    if services.next().is_some() {
        return Err(malformed(file, "several services"));
    }
    Ok(shape.traits)
}

fn service_name(file: &Path, models: &Path) -> Result<String, Error> {
    file.strip_prefix(models)
        .ok()
        .and_then(|p| p.components().next())
        .and_then(|p| p.as_os_str().to_str())
        .map(str::to_owned)
        .ok_or_else(|| malformed(file, "invalid service directory"))
}

fn malformed(file: &Path, message: &str) -> Error {
    model_error(file, Fault::Malformed(message.into()))
}
