use super::cases::CaseCounts;
use super::report_text::render;
use super::static_context::{HostLabel, Operation};
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
    input: Option<Target>,
    #[serde(default, deserialize_with = "crate::model_source::unique_map")]
    members: BTreeMap<String, Member>,
}
#[derive(Deserialize)]
struct Target {
    target: String,
}
#[derive(Deserialize)]
struct Member {
    #[serde(default)]
    traits: MemberTraits,
}
#[derive(Default, Deserialize)]
struct MemberTraits {
    #[serde(rename = "smithy.api#hostLabel")]
    host_label: Option<serde::de::IgnoredAny>,
    #[serde(rename = "smithy.api#required")]
    required: Option<serde::de::IgnoredAny>,
    #[serde(rename = "smithy.rules#contextParam")]
    context: Option<ContextParameter>,
}
#[derive(Deserialize)]
struct ContextParameter {
    name: String,
}
#[derive(Default, Deserialize)]
struct Traits {
    #[serde(rename = "smithy.rules#endpointRuleSet")]
    rules: Option<Value>,
    #[serde(rename = "smithy.rules#endpointTests")]
    tests: Option<Tests>,
    #[serde(rename = "aws.auth#sigv4")]
    signing: Option<Signing>,
    #[serde(rename = "smithy.rules#staticContextParams")]
    static_context: Option<super::static_context::Context>,
    #[serde(rename = "smithy.api#endpoint")]
    endpoint: Option<HostPrefix>,
}
#[derive(Deserialize)]
struct HostPrefix {
    #[serde(rename = "hostPrefix")]
    prefix: String,
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
            let signing = evidence.signing_name.as_deref();
            let service = run.verified_endpoint(signing).ok();
            let carrier = service.is_none();
            if let Some(mut verified) = service.or_else(|| run.operation_carrier(signing)) {
                let mut routed = false;
                for (operation, modeled) in evidence.operations {
                    let endpoint = super::static_context::compile(
                        file,
                        &evidence.rules,
                        &evidence.cases,
                        &operation,
                        &modeled,
                        signing,
                    )
                    .ok();
                    routed |= endpoint.is_some();
                    verified.bind_operation(operation, endpoint);
                }
                if routed || !carrier {
                    endpoints.insert(evidence.name, verified);
                }
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
    operations: BTreeMap<String, Operation>,
}

fn read_evidence(file: &Path, models: &Path) -> Result<ServiceEvidence, Error> {
    let (traits, operations) = read_traits(file)?;
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
        operations,
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

fn read_traits(file: &Path) -> Result<(Traits, BTreeMap<String, Operation>), Error> {
    let model: Model = crate::model_source::read_model(file)?;
    let mut labels = model
        .shapes
        .iter()
        .filter(|(_, shape)| shape.kind == "operation" && shape.traits.static_context.is_some())
        .filter_map(|(id, shape)| Some((id.clone(), host_label(&model, shape)?)))
        .collect::<BTreeMap<_, _>>();
    let mut service = None;
    let mut operations = BTreeMap::new();
    for (id, shape) in model.shapes {
        if shape.kind == "service" {
            if service.replace(shape.traits).is_some() {
                return Err(malformed(file, "several services"));
            }
        } else if shape.kind == "operation"
            && let Some(context) = shape.traits.static_context
        {
            let (_, name) = id
                .rsplit_once('#')
                .ok_or_else(|| malformed(file, "invalid operation shape identifier"))?;
            let operation = Operation {
                context,
                label: labels.remove(&id),
            };
            if operations.insert(name.to_owned(), operation).is_some() {
                return Err(malformed(file, "duplicate operation endpoint context"));
            }
        }
    }
    let traits = service.ok_or_else(|| malformed(file, "no service"))?;
    Ok((traits, operations))
}

fn host_label(model: &Model, operation: &Shape) -> Option<HostLabel> {
    let input = model.shapes.get(&operation.input.as_ref()?.target)?;
    let mut bound = input.members.iter().filter_map(|(member, shape)| {
        let traits = &shape.traits;
        let context = traits.context.as_ref()?;
        (traits.host_label.is_some() && traits.required.is_some())
            .then_some((member, &context.name))
    });
    let (member, parameter) = bound.next()?;
    if bound.next().is_some() {
        return None;
    }
    let own = format!("{{{member}}}");
    let prefix = operation
        .traits
        .endpoint
        .as_ref()
        .map(|endpoint| endpoint.prefix.as_str());
    let prefixed = match prefix {
        Some(prefix) if prefix.strip_suffix('.') == Some(own.as_str()) => true,
        Some(prefix) if prefix.contains(&own) => return None,
        _ => false,
    };
    Some(HostLabel {
        parameter: parameter.clone(),
        prefixed,
    })
}

fn malformed(file: &Path, message: &str) -> Error {
    model_error(file, Fault::Malformed(message.into()))
}
