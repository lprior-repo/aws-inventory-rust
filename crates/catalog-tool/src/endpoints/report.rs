use super::cases::CaseCounts;
use super::report_text::render;
use super::{Case, DerivedEndpoint, EndpointException, Error, Fault, RuleSet, VerifiedEndpoint};
use serde::Deserialize;
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

#[derive(Deserialize)]
struct Model {
    #[serde(deserialize_with = "crate::model_source::unique_map")]
    shapes: std::collections::BTreeMap<String, Shape>,
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
    pub(super) failures: Vec<(usize, String)>,
}

pub fn build_report(source: &Path) -> Result<String, Error> {
    let models = source.join("models");
    let files = model_files(&models)?;
    let (counts, mut services) = load_services(&files, &models)?;
    services.sort_by(|a, b| a.name.cmp(&b.name));
    render(&counts, &services)
}

pub fn verified_endpoints(source: &Path) -> Result<BTreeMap<String, VerifiedEndpoint>, Error> {
    let models = source.join("models");
    let files = model_files(&models)?;
    let mut seen = BTreeSet::new();
    files
        .iter()
        .try_fold(BTreeMap::new(), |mut endpoints, file| {
            let evidence = read_evidence(file, &models)?;
            if !seen.insert(evidence.name.clone()) {
                return Err(model_error(
                    file,
                    Fault::Malformed("duplicate service directory".into()),
                ));
            }
            let rules = RuleSet::parse(file, &evidence.rules)?;
            let run = rules.run_cases(&evidence.cases)?;
            if let Ok(verified) = run.verified_endpoint(evidence.signing_name.as_deref()) {
                endpoints.insert(evidence.name, verified);
            }
            Ok(endpoints)
        })
}

fn model_files(models: &Path) -> Result<Vec<std::path::PathBuf>, Error> {
    let files = crate::model_source::discover_models(models)?;
    if files.is_empty() {
        return Err(crate::model_source::Error::NoModels.into());
    }
    if files.len() > 1024 {
        return Err(model_error(models, Fault::Bound("services")));
    }
    Ok(files)
}

struct ServiceEvidence {
    name: String,
    rules: Value,
    cases: Vec<Case>,
    signing_name: Option<String>,
}

fn read_evidence(file: &Path, models: &Path) -> Result<ServiceEvidence, Error> {
    let traits = read_traits(file)?;
    let rules = traits
        .rules
        .ok_or_else(|| malformed(file, "missing rules"))?;
    let tests = traits
        .tests
        .ok_or_else(|| malformed(file, "missing tests"))?;
    let name = crate::model_source::service_name(models, file)?.to_owned();
    Ok(ServiceEvidence {
        name,
        rules,
        cases: tests.cases,
        signing_name: traits.signing.map(|signing| signing.name),
    })
}

fn read_service(file: &Path, models: &Path) -> Result<(Service, CaseCounts), Error> {
    let evidence = read_evidence(file, models)?;
    let rules = RuleSet::parse(file, &evidence.rules)?;
    let run = rules.run_cases(&evidence.cases)?;
    let derived = run.derive(evidence.signing_name.as_deref())?;
    let failures = run
        .failures()
        .iter()
        .map(|failure| (failure.index(), failure.reason().to_owned()))
        .collect();
    let unsupported = run.unsupported().to_vec();
    Ok((
        Service {
            name: evidence.name,
            endpoint: derived.endpoint().clone(),
            unsupported,
            exceptions: derived.exceptions().to_vec(),
            failures,
        },
        run.totals(),
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
    let model: Model = crate::model_source::read_model(file)?;
    let mut services = model.shapes.into_values().filter(|s| s.kind == "service");
    let shape = services
        .next()
        .ok_or_else(|| malformed(file, "no service"))?;
    if services.next().is_some() {
        return Err(malformed(file, "several services"));
    }
    Ok(shape.traits)
}

fn malformed(file: &Path, message: &str) -> Error {
    model_error(file, Fault::Malformed(message.into()))
}
