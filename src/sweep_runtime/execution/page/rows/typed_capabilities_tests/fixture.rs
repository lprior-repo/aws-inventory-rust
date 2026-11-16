use super::*;
use catalog_tool::{endpoints, kind_map, overlay, runtime_catalog, smithy};
use std::sync::LazyLock;

static CATALOG: LazyLock<Result<Vec<u8>, String>> = LazyLock::new(|| compile().map_err(|error| error.to_string()));

pub(super) fn bytes() -> TestResult<&'static [u8]> {
    CATALOG.as_ref().map(Vec::as_slice).map_err(|error| error.clone().into())
}

fn compile() -> TestResult<Vec<u8>> {
    let root = tempfile::tempdir()?;
    let source = root.path().join("source");
    let embedded = overlay::embedded()?;
    let mut policies = Vec::new();
    for (service, date, operations) in [
        ("evs", "2023-07-27", &["ListEnvironments", "ListEnvironmentVlans"][..]),
        ("qconnect", "2020-10-19", &["ListAssistants", "ListAIPrompts", "ListAIPromptVersions"][..]),
        ("s3tables", "2018-05-10", &["ListTableBuckets", "ListNamespaces"][..]),
        ("securitylake", "2018-05-10", &["ListDataLakes"][..]),
    ] {
        crate::sweep_runtime::pinned_model::install(&source.join("models"), service, date, operations)?;
        let policy = embedded.iter().find(|candidate| candidate.service() == service).ok_or("production service absent")?;
        let selected = operations.iter().map(|operation| {
            let key = overlay::OperationName::new(operation)?;
            Ok((*operation, policy.operations().get(&key).ok_or("production operation absent")?.clone()))
        }).collect::<TestResult<Vec<_>>>()?;
        policies.push(overlay::ServiceOverlay::new(service, selected)?);
    }
    let checked = overlay::check_policies(&source, &policies, &catalog_tool::classes::DenyList::builtin())?;
    let fields = smithy::Registry::parse(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/catalog/field-ids.txt")))?;
    let base = smithy::build_catalog(&source, &fields, "46135f8468cd5cb60d983b97290c9f6661e359c6")?;
    let kinds = kind_map::embedded(include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/catalog/cloudcontrol-universal.json")))?;
    Ok(runtime_catalog::augment(&base, &checked, &kinds, &endpoints::verified_endpoints(&source)?)?)
}

pub(super) fn decode(plan: &CallablePlan<'_>, body: &[u8]) -> TestResult<Record> {
    use aws_wire::{request::Protocol, response::{DecodeLimits, ResponseContext}};
    let protocol = match plan.protocol() {
        inventory_core::catalog::runtime::RuntimeProtocol::RestJson1 => Protocol::RestJson1,
        inventory_core::catalog::runtime::RuntimeProtocol::AwsJson11 => Protocol::AwsJson11,
        _ => return Err("unexpected typed operand fixture protocol".into()),
    };
    let headers = http::HeaderMap::new();
    Ok(aws_wire::response::decode(ResponseContext { service: plan.service(), operation: plan.operation(), protocol,
        body, headers: &headers, status: http::StatusCode::OK, limits: DecodeLimits::default() })?)
}
