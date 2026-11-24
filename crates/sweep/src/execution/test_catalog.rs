use catalog_tool::{classes::DenyList, endpoints, kind_map, overlay, runtime_catalog, smithy};
use std::{collections::BTreeMap, path::Path};

type TestResult<T> = Result<T, Box<dyn std::error::Error>>;

pub(super) fn compile(
    service: &str,
    date: &str,
    operations: &[&str],
    resolved: Option<endpoints::VerifiedEndpoint>,
    mut customize: impl FnMut(&str, &mut overlay::OperationOverlay) -> TestResult<()>,
) -> TestResult<Vec<u8>> {
    let directory = tempfile::tempdir()?;
    crate::pinned_model::install(&directory.path().join("models"), service, date, operations)?;
    let fields = smithy::Registry::parse(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../catalog/field-ids.txt"
    )))?;
    let base = smithy::build_catalog(directory.path(), &fields, "ownership-regression")?;
    let embedded = overlay::embedded()?
        .into_iter()
        .find(|policy| policy.service() == service)
        .ok_or("embedded policy absent")?;
    let mut selected = Vec::new();
    for operation in operations {
        let key = overlay::OperationName::new(operation)?;
        let mut policy = embedded
            .operations()
            .get(&key)
            .ok_or("operation absent")?
            .clone();
        customize(operation, &mut policy)?;
        selected.push((*operation, policy));
    }
    let policies = [overlay::ServiceOverlay::new(service, selected)?];
    let checked = overlay::check_policies(directory.path(), &policies, &DenyList::builtin())?;
    let kinds = kind_map::embedded(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../catalog/cloudcontrol-universal.json"
    )))?;
    let resolved = match resolved {
        Some(endpoint) => endpoint,
        None => endpoint(service)?,
    };
    let endpoints = BTreeMap::from([(service.to_owned(), resolved)]);
    Ok(runtime_catalog::augment(
        &base, &checked, &kinds, &endpoints,
    )?)
}

fn endpoint(service: &str) -> TestResult<endpoints::VerifiedEndpoint> {
    use serde_json::json;
    let auth = json!({"authSchemes":[{
        "name":"sigv4", "signingName":service, "signingRegion":"us-west-2"
    }]});
    let document = json!({"version":"1.0", "parameters":{"Region":{"type":"string"}},
    "rules":[
        {"type":"endpoint", "conditions":[{"fn":"isSet","argv":[{"ref":"Region"}]}],
         "endpoint":{"url":"https://regional.example.test", "properties":auth}},
        {"type":"endpoint", "conditions":[],
         "endpoint":{"url":"https://global.example.test", "properties":auth}}
    ]});
    let rules = endpoints::RuleSet::parse(Path::new("ownership-endpoints.json"), &document)?;
    let cases: Vec<endpoints::Case> = serde_json::from_value(json!([
        {"params":{"Region":"us-east-1"}, "expect":{"endpoint":{
            "url":"https://regional.example.test", "properties":auth}}},
        {"params":{"Region":"us-west-2"}, "expect":{"endpoint":{
            "url":"https://regional.example.test", "properties":auth}}},
        {"params":{}, "expect":{"endpoint":{
            "url":"https://global.example.test", "properties":auth}}}
    ]))?;
    rules
        .run_cases(&cases)?
        .verified_endpoint(Some(service))
        .map_err(|error| format!("endpoint evidence failed: {error:?}").into())
}

pub(super) fn loopback_endpoint(
    service: &str,
    address: std::net::SocketAddr,
) -> TestResult<endpoints::VerifiedEndpoint> {
    use serde_json::json;
    let url = format!("http://{address}/{{Region}}");
    let global = format!("http://{address}/global");
    let auth = |region| {
        json!({"authSchemes":[{
            "name":"sigv4", "signingName":service, "signingRegion":region
        }]})
    };
    let document = json!({"version":"1.0", "parameters":{"Region":{"type":"string"}},
    "rules":[
        {"type":"endpoint", "conditions":[{"fn":"isSet","argv":[{"ref":"Region"}]}],
         "endpoint":{"url":url, "properties":auth("{Region}")}},
        {"type":"endpoint", "conditions":[],
         "endpoint":{"url":global, "properties":auth("us-east-1")}}
    ]});
    let rules = endpoints::RuleSet::parse(Path::new("loopback-endpoints.json"), &document)?;
    let cases: Vec<endpoints::Case> = serde_json::from_value(json!([
        {"params":{"Region":"us-east-1"}, "expect":{"endpoint":{
            "url":format!("http://{address}/us-east-1"), "properties":auth("us-east-1")}}},
        {"params":{"Region":"us-west-2"}, "expect":{"endpoint":{
            "url":format!("http://{address}/us-west-2"), "properties":auth("us-west-2")}}},
        {"params":{}, "expect":{"endpoint":{"url":global, "properties":auth("us-east-1")}}}
    ]))?;
    rules
        .run_cases(&cases)?
        .verified_endpoint(Some(service))
        .map_err(|error| format!("loopback endpoint evidence failed: {error:?}").into())
}
