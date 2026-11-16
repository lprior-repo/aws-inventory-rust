mod compile;
pub(super) mod identity;
mod page;
mod service;
mod tags;
mod fixed_input;
mod selection;
mod routing;
mod capabilities;
mod binding_alternatives;
mod tag_endpoint;
mod map_exclusions;
mod row_policy;
pub(super) mod emissions;
use super::plan_validation;
use super::{RuntimeCatalogCompileError, encode};
use crate::{
    endpoints::{DerivedEndpoint, ServiceClass, VerifiedEndpoint},
    kind_map::{BootstrapPurpose, KindMap},
    overlay::{Binding, Identity, ItemSelector, OperationOverlay, Scope},
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
        Some(ItemSelector::Explicit { path, kind }) => selection::item(path, *kind, services),
        None => Err(RuntimeCatalogCompileError::Capability),
    }
}

fn identity_value(
    identity: Option<&Identity>,
    service: &str,
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
        Some(value @ (Identity::Choice(_) | Identity::CorrelatedKey(_) | Identity::MapArn { .. })) => selection::identity(value, service, services),
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

fn endpoint_value(endpoint: &VerifiedEndpoint, operation: &str) -> Result<Value, RuntimeCatalogCompileError> {
    let DerivedEndpoint::Resolved {
        class,
        url_template: url,
        signing_name,
        signing_region,
    } = endpoint.operation_endpoint(operation).ok_or(RuntimeCatalogCompileError::Capability)?
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
    operation: inventory_core::catalog::Operation<'_>,
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
                (fixed_input::compile(
                    catalog_service(services, service)?,
                    super::model_path::resolve(catalog_service(services, service)?, operation.input()?, path.as_str())?.1,
                    value,
                )?, Value::Null, Value::Null),
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
    if let Binding::Transformed { binding, transform: crate::overlay::BindingTransform::StripPrefix(prefix) } = binding {
        if prefix.is_empty() || prefix.len() > 256 || prefix.chars().any(char::is_control)
            || !matches!(binding.as_ref(), Binding::From { .. } | Binding::Context { .. } | Binding::FromArn { .. } | Binding::FromService { .. }) {
            return Err(RuntimeCatalogCompileError::Capability);
        }
        return encode::map([(1, encode::path(services, path.as_str())?), (2, Value::Int(9)),
            (3, plain_binding_value(service, path, binding, services)?), (4, Value::Text(prefix.clone()))]);
    }
    plain_binding_value(service, path, binding, services)
}

fn plain_binding_value(
    service: &str,
    path: &crate::overlay::MemberPath,
    binding: &Binding,
    services: &super::ServiceIndex<'_>,
) -> Result<Value, RuntimeCatalogCompileError> {
    let (source, values) = match binding {
        Binding::Account => (0, (Value::Null, Value::Null, Value::Null)),
        Binding::Region => (1, (Value::Null, Value::Null, Value::Null)),
        Binding::RegionList => (11, (Value::Null, Value::Null, Value::Null)),
        Binding::FromAny(alternatives) => {
            return binding_alternatives::encode(service, path, alternatives, services);
        }
        Binding::FromBranch { operation, branch, path: source_path } => {
            return branch_binding(service, path, operation, branch, crate::overlay::ContextSource::SelectedItem, source_path, services);
        }
        Binding::ContextFromBranch { operation, branch, source, path: source_path } => {
            return branch_binding(service, path, operation, branch, *source, source_path, services);
        }
        Binding::Context { operation, source, path } => (
            match source {
                crate::overlay::ContextSource::SelectedItem => 5,
                crate::overlay::ContextSource::Response => 6,
                crate::overlay::ContextSource::Request => 7,
            },
            (
                Value::Text(service.to_owned()),
                Value::Text(operation.to_string()),
                selected_path(services, path.as_str())?,
            ),
        ),
        Binding::From { operation, path } => (
            4,
            (
                Value::Text(service.to_owned()),
                Value::Text(operation.to_string()),
                selected_path(services, path.as_str())?,
            ),
        ),
        Binding::FromService { service, operation, path } => (
            4, (Value::Text(service.clone()), Value::Text(operation.to_string()), selected_path(services, path.as_str())?),
        ),
        Binding::FromArn { operation, path: source_path, arn_service, resource_kind } => {
            let kind = selection::arn_kind(*resource_kind);
            return encode::map([
                (1, encode::path(services, path.as_str())?), (2, Value::Int(8)),
                (3, Value::Text(service.to_owned())), (4, Value::Text(operation.to_string())),
                (5, selected_path(services, source_path.as_str())?), (6, Value::Text(arn_service.clone())), (7, Value::Int(kind)),
            ]);
        }
        Binding::FromUrl { operation, path: source_path, resource_kind } => {
            let kind = match resource_kind { crate::overlay::UrlResourceKind::SqsQueue => 0 };
            return encode::map([(1, encode::path(services, path.as_str())?), (2, Value::Int(10)),
                (3, Value::Text(service.to_owned())), (4, Value::Text(operation.to_string())),
                (5, selected_path(services, source_path.as_str())?), (6, Value::Int(kind))]);
        }
        Binding::Transformed { .. } => return Err(RuntimeCatalogCompileError::Capability),
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


fn selected_path(
    services: &super::ServiceIndex<'_>,
    path: &str,
) -> Result<Value, RuntimeCatalogCompileError> {
    if path == "$" {
        return Ok(Value::List(Vec::new()));
    }
    encode::path(services, path)
}

fn branch_binding(service: &str, target: &crate::overlay::MemberPath, operation: &crate::overlay::OperationName,
    branch: &str, source: crate::overlay::ContextSource, path: &crate::overlay::MemberPath,
    services: &super::ServiceIndex<'_>) -> Result<Value, RuntimeCatalogCompileError> {
    if !routing::valid_branch_name(branch) { return Err(RuntimeCatalogCompileError::Capability); }
    encode::map([(1, encode::path(services, target.as_str())?), (2, Value::Int(13)),
        (3, Value::Text(service.to_owned())), (4, Value::Text(operation.to_string())),
        (5, selected_path(services, path.as_str())?), (6, Value::Text(branch.to_owned())),
        (7, Value::Int(selection::source_tag(source)))])
}
