mod compile;
pub(super) mod identity;
mod page;
mod service;
use super::plan_validation;
use super::{RuntimeCatalogCompileError, encode};
use crate::{
    endpoints::{DerivedEndpoint, ServiceClass, VerifiedEndpoint},
    kind_map::{BootstrapPurpose, KindMap},
    overlay::{Binding, FixedValue, Identity, ItemSelector, OperationOverlay, Scope},
};
pub(super) use compile::{compile, decisions};
use inventory_core::record::Value;
use page::compile as compile_page;
pub(super) use service::{catalog_operation, catalog_service};

fn scope(value: Option<Scope>) -> Result<i64, RuntimeCatalogCompileError> {
    match value {
        Some(Scope::Region) => Ok(0),
        Some(Scope::Account) => Ok(1),
        Some(Scope::HomeRegion) => Ok(2),
        _ => Err(RuntimeCatalogCompileError::Capability),
    }
}

fn item_value(
    selector: Option<&ItemSelector>,
    services: &super::ServiceIndex<'_>,
) -> Result<Value, RuntimeCatalogCompileError> {
    match selector {
        Some(ItemSelector::Root) => encode::map([(1, Value::Int(0))]),
        Some(ItemSelector::Member(path)) => encode::map([
            (1, Value::Int(1)),
            (2, encode::path(services, &path.to_string())?),
        ]),
        None => Err(RuntimeCatalogCompileError::Capability),
    }
}

fn identity_value(
    identity: Option<&Identity>,
    services: &super::ServiceIndex<'_>,
) -> Result<Value, RuntimeCatalogCompileError> {
    match identity {
        Some(Identity::Arn(path)) => encode::map([
            (1, Value::Int(0)),
            (2, selected_path(services, path.as_str())?),
        ]),
        Some(Identity::Key(paths)) => encode::map([
            (1, Value::Int(2)),
            (
                2,
                Value::List(
                    paths
                        .iter()
                        .map(|path| selected_path(services, path.as_str()))
                        .collect::<Result<_, _>>()?,
                ),
            ),
        ]),
        Some(Identity::Template(template)) => identity::compile(template, services),
        _ => Err(RuntimeCatalogCompileError::Capability),
    }
}

fn exclusions(
    overlay: &OperationOverlay,
    services: &super::ServiceIndex<'_>,
) -> Result<Value, RuntimeCatalogCompileError> {
    overlay
        .exclude_from_hash
        .iter()
        .map(|path| encode::path(services, &path.to_string()))
        .collect::<Result<Vec<_>, _>>()
        .map(Value::List)
}

fn endpoint_value(endpoint: &VerifiedEndpoint) -> Result<Value, RuntimeCatalogCompileError> {
    let DerivedEndpoint::Resolved {
        class,
        url_template: url,
        signing_name,
        signing_region,
    } = endpoint.endpoint()
    else {
        return Err(RuntimeCatalogCompileError::Capability);
    };
    let class = match class {
        ServiceClass::Global => 0,
        ServiceClass::Regional => 1,
    };
    encode::map([
        (1, Value::Text(url.clone())),
        (2, Value::Text(signing_region.clone())),
        (3, Value::Text(signing_name.clone())),
        (4, Value::Int(class)),
        (5, Value::Int(0)),
    ])
}

fn review_value(
    review: &crate::overlay::ReviewedService,
) -> Result<Value, RuntimeCatalogCompileError> {
    encode::map([
        (1, Value::Text("catalog/aws-overlay".to_owned())),
        (2, Value::Text(review.reviewer().to_owned())),
        (3, Value::Text(review.reviewed().to_owned())),
    ])
}

fn purpose_value(
    service: &str,
    operation: &str,
    kinds: &KindMap,
    services: &super::ServiceIndex<'_>,
) -> Result<Value, RuntimeCatalogCompileError> {
    match kinds.bootstrap_for(service, operation) {
        Some(BootstrapPurpose::AccountIdentity { account_path }) => encode::map([
            (1, Value::Int(1)),
            (2, encode::path(services, account_path)?),
        ]),
        Some(purpose @ BootstrapPurpose::RegionDiscovery { .. }) => {
            region_purpose(purpose, services)
        }
        Some(BootstrapPurpose::Enumeration) | None => encode::map([(1, Value::Int(0))]),
    }
}

fn region_purpose(
    purpose: &BootstrapPurpose,
    services: &super::ServiceIndex<'_>,
) -> Result<Value, RuntimeCatalogCompileError> {
    let BootstrapPurpose::RegionDiscovery {
        regions_path,
        region_name_path,
        opt_in_status_path,
    } = purpose
    else {
        return Err(RuntimeCatalogCompileError::Capability);
    };
    let mut fields = vec![
        (1, Value::Int(2)),
        (2, encode::path(services, regions_path)?),
        (3, encode::path(services, region_name_path)?),
    ];
    if let Some(path) = opt_in_status_path {
        fields.push((4, encode::path(services, path)?));
    }
    encode::map(fields)
}

fn protocol(
    service: inventory_core::catalog::Service<'_>,
) -> Result<i64, RuntimeCatalogCompileError> {
    let protocols = service.protocols()?.collect::<Result<Vec<_>, _>>()?;
    [
        ("smithy.protocols#rpcv2Cbor", 2),
        ("aws.protocols#awsJson1_1", 1),
        ("aws.protocols#awsJson1_0", 0),
        ("aws.protocols#awsQuery", 3),
        ("aws.protocols#ec2Query", 4),
        ("aws.protocols#restJson1", 5),
        ("aws.protocols#restXml", 6),
    ]
    .into_iter()
    .find_map(|(name, code)| protocols.contains(&name).then_some(code))
    .ok_or(RuntimeCatalogCompileError::Capability)
}
fn binding_values(
    service: &str,
    overlay: &OperationOverlay,
    services: &super::ServiceIndex<'_>,
) -> Result<Value, RuntimeCatalogCompileError> {
    let bindings = overlay
        .bind
        .iter()
        .map(|(path, binding)| binding_value(service, path, binding, services))
        .chain(overlay.fixed.iter().map(|(path, value)| {
            binding_record(
                path,
                2,
                (fixed_value(value)?, Value::Null, Value::Null),
                services,
            )
        }))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Value::List(bindings))
}

fn binding_value(
    service: &str,
    path: &crate::overlay::MemberPath,
    binding: &Binding,
    services: &super::ServiceIndex<'_>,
) -> Result<Value, RuntimeCatalogCompileError> {
    let (source, values) = match binding {
        Binding::Account => (0, (Value::Null, Value::Null, Value::Null)),
        Binding::Region => (1, (Value::Null, Value::Null, Value::Null)),
        Binding::From { operation, path } => (
            4,
            (
                Value::Text(service.to_owned()),
                Value::Text(operation.to_string()),
                selected_path(services, path.as_str())?,
            ),
        ),
    };
    binding_record(path, source, values, services)
}

fn binding_record(
    path: &crate::overlay::MemberPath,
    source: i64,
    values: (Value, Value, Value),
    services: &super::ServiceIndex<'_>,
) -> Result<Value, RuntimeCatalogCompileError> {
    let (third, fourth, fifth) = values;
    let present = [(3, third), (4, fourth), (5, fifth)]
        .into_iter()
        .filter(|(_, value)| !matches!(value, Value::Null));
    encode::map(
        [
            (1, encode::path(services, &path.to_string())?),
            (2, Value::Int(source)),
        ]
        .into_iter()
        .chain(present),
    )
}

fn fixed_value(value: &FixedValue) -> Result<Value, RuntimeCatalogCompileError> {
    match value {
        FixedValue::String(value) => Ok(Value::Text(value.clone())),
        FixedValue::Integer(value) => Ok(Value::Int(*value)),
        FixedValue::Boolean(value) => Ok(Value::Bool(*value)),
        FixedValue::Strings(values) => Ok(Value::List(
            values.iter().cloned().map(Value::Text).collect(),
        )),
    }
}

fn selected_path(
    services: &super::ServiceIndex<'_>,
    path: &str,
) -> Result<Value, RuntimeCatalogCompileError> {
    if path == "$" {
        return Ok(Value::List(Vec::new()));
    }
    encode::path(services, path)
}
