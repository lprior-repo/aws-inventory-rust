use super::fixture_catalog::{
    classes::DenyList, endpoints, kind_map, overlay, runtime_catalog, smithy,
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

pub(super) type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

pub(super) struct Compiled {
    pub bytes: Vec<u8>,
    pub fields: smithy::Registry,
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(
        if matches!(env!("CARGO_PKG_NAME"), "catalog-tool" | "inventory-sweep") {
            "../.."
        } else {
            "."
        },
    )
}

fn selected_policy(service: &str, operations: &[&str]) -> TestResult<overlay::ServiceOverlay> {
    let policy = overlay::embedded()?
        .into_iter()
        .find(|policy| policy.service() == service)
        .ok_or("embedded service missing")?;
    let selected = operations
        .iter()
        .map(|operation| {
            let name = overlay::OperationName::new(operation)?;
            let decision = policy
                .operations()
                .get(&name)
                .ok_or("embedded operation missing")?;
            Ok((*operation, decision.clone()))
        })
        .collect::<TestResult<Vec<_>>>()?;
    Ok(overlay::ServiceOverlay::new(service, selected)?)
}

fn selected_model(service: &str, date: &str, operations: &[&str]) -> TestResult<Value> {
    let file = root().join(format!(
        ".local/aws-models/models/{service}/service/{date}/{service}-{date}.json"
    ));
    let mut model: Value = serde_json::from_slice(&fs::read(file)?)?;
    let shapes = model
        .get_mut("shapes")
        .and_then(Value::as_object_mut)
        .ok_or("shapes missing")?;
    shapes.retain(|name, shape| {
        shape.get("type").and_then(Value::as_str) != Some("operation")
            || name
                .rsplit('#')
                .next()
                .is_some_and(|name| operations.contains(&name))
    });
    let targets = shapes
        .iter()
        .filter(|(_, shape)| shape.get("type").and_then(Value::as_str) == Some("operation"))
        .map(|(name, _)| json!({"target":name}))
        .collect::<Vec<_>>();
    shapes
        .values_mut()
        .filter(|shape| shape.get("type").and_then(Value::as_str) == Some("service"))
        .try_for_each(|service| -> TestResult {
            let object = service.as_object_mut().ok_or("service is not an object")?;
            object.insert("operations".to_owned(), Value::Array(targets.clone()));
            object.remove("resources");
            Ok(())
        })?;
    Ok(model)
}

fn endpoint(service: &str) -> TestResult<endpoints::VerifiedEndpoint> {
    let properties =
        json!({"authSchemes":[{"name":"sigv4","signingName":service,"signingRegion":"us-east-1"}]});
    let document = json!({"version":"1.0","parameters":{"Region":{"type":"string"}},"rules":[{"type":"endpoint","conditions":[],"endpoint":{"url":"https://fixture.example.test","properties":properties}}]});
    let rules = endpoints::RuleSet::parse(Path::new("ac-consumer-endpoint.json"), &document)?;
    let cases: Vec<endpoints::Case> = serde_json::from_value(
        json!([{"params":{"Region":"us-east-1"},"expect":{"endpoint":{"url":"https://fixture.example.test","properties":properties}}}]),
    )?;
    rules
        .run_cases(&cases)?
        .verified_endpoint(Some(service))
        .map_err(|error| format!("fixture endpoint evidence failed: {error:?}").into())
}

pub(super) fn compile(service: &str, date: &str, operations: &[&str]) -> TestResult<Compiled> {
    compile_many(&[(service, date, operations)])
}

pub(super) fn compile_many(sources: &[(&str, &str, &[&str])]) -> TestResult<Compiled> {
    compile_many_with(sources, |_, policy| Ok(policy))
}

pub(super) fn compile_many_with(
    sources: &[(&str, &str, &[&str])],
    mut customize: impl FnMut(&str, overlay::ServiceOverlay) -> TestResult<overlay::ServiceOverlay>,
) -> TestResult<Compiled> {
    let models = sources
        .iter()
        .map(|(service, date, operations)| {
            Ok((
                *service,
                *date,
                *operations,
                selected_model(service, date, operations)?,
            ))
        })
        .collect::<TestResult<Vec<_>>>()?;
    let mut names = BTreeSet::new();
    for (_, _, _, model) in &models {
        let shapes = model
            .get("shapes")
            .and_then(Value::as_object)
            .ok_or("shapes missing")?;
        names.extend(
            shapes
                .values()
                .filter_map(|shape| shape.get("members").and_then(Value::as_object))
                .flat_map(|members| members.keys().cloned()),
        );
    }
    let fields = smithy::Registry::default().extend(names)?;
    let temporary = tempfile::tempdir()?;
    let mut policies = Vec::new();
    let mut endpoints = BTreeMap::new();
    for (service, date, operations, model) in &models {
        let directory = temporary
            .path()
            .join(format!("models/{service}/service/{date}"));
        fs::create_dir_all(&directory)?;
        fs::write(
            directory.join(format!("{service}-{date}.json")),
            serde_json::to_vec(model)?,
        )?;
        policies.push(customize(service, selected_policy(service, operations)?)?);
        endpoints.insert((*service).to_owned(), endpoint(service)?);
    }
    let checked = overlay::check_policies(temporary.path(), &policies, &DenyList::builtin())?;
    let base = smithy::build_catalog(temporary.path(), &fields, "ac-consumer-regression")?;
    let mappings = kind_map::embedded(&fs::read(
        root().join("catalog/cloudcontrol-universal.json"),
    )?)?;
    let bytes = runtime_catalog::augment(&base, &checked, &mappings, &endpoints)?;
    temporary.close()?;
    Ok(Compiled { bytes, fields })
}
