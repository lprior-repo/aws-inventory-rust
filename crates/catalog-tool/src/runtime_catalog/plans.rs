mod binding_alternatives;
pub(super) mod bindings;
use bindings::{binding_record, binding_value, binding_values};
mod capabilities;
mod compile;
pub(super) mod emissions;
mod fixed_input;
pub(super) mod identity;
mod map_exclusions;
mod page;
mod routing;
mod row_policy;
mod selection;
mod service;
mod tag_endpoint;
mod tags;
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
        _ => Err(RuntimeCatalogCompileError::Path {
            path: "scope".to_owned(),
        }),
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
        None => Err(RuntimeCatalogCompileError::Path {
            path: "item selector".to_owned(),
        }),
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
        Some(
            value @ (Identity::Choice(_) | Identity::CorrelatedKey(_) | Identity::MapArn { .. }),
        ) => selection::identity(value, service, services),
        _ => Err(RuntimeCatalogCompileError::Path {
            path: "identity value".to_owned(),
        }),
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

fn endpoint_value(
    endpoint: &VerifiedEndpoint,
    operation: &str,
) -> Result<Value, RuntimeCatalogCompileError> {
    let DerivedEndpoint::Resolved {
        class,
        url_template: url,
        signing_name,
        signing_region,
    } = endpoint
        .operation_endpoint(operation)
        .ok_or(RuntimeCatalogCompileError::Path {
            path: "endpoint resolution".to_owned(),
        })?
    else {
        return Err(RuntimeCatalogCompileError::Path {
            path: "endpoint derivation".to_owned(),
        });
    };
    let class = match class {
        ServiceClass::Global => 0,
        ServiceClass::Regional => 1,
    };
    let exceptions = endpoint
        .exceptions()
        .iter()
        .map(encode_endpoint_exception)
        .collect::<Result<Vec<_>, _>>()?;
    let mut endpoint_map = encode::map([
        (1, Value::Text(url.clone())),
        (2, Value::Text(signing_region.clone())),
        (3, Value::Text(signing_name.clone())),
        (4, Value::Int(class)),
        (5, Value::Int(0)),
    ])?;
    if let (true, Value::Map(fields)) = (!exceptions.is_empty(), &mut endpoint_map) {
        fields.insert(
            inventory_core::record::Key::Field(inventory_core::record::FieldId::new(6)?),
            Value::List(exceptions),
        );
    }
    Ok(endpoint_map)
}

fn encode_endpoint_exception(
    exception: &crate::endpoints::EndpointException,
) -> Result<Value, RuntimeCatalogCompileError> {
    encode::map([
        (1, Value::Text(exception.region.clone())),
        (2, Value::Text(exception.url.clone())),
        (3, Value::Text(exception.signing_name.clone())),
        (4, Value::Text(exception.signing_region.clone())),
    ])
}

fn review_value(
    review: &crate::overlay::ReviewedService,
    origin: crate::overlay::Origin,
) -> Result<Value, RuntimeCatalogCompileError> {
    let evidence = review.evidence();
    let reviewer = match origin {
        crate::overlay::Origin::Hand => review.reviewer(),
        crate::overlay::Origin::Generated => "generated",
    };
    encode::map([
        (1, Value::Text("catalog/aws-overlay".to_owned())),
        (2, Value::Text(reviewer.to_owned())),
        (3, Value::Text(review.reviewed().to_owned())),
        (4, Value::Text(env!("CARGO_PKG_VERSION").to_owned())),
        (5, Value::Int(i64::from(evidence.machine_reviewed))),
        (6, Value::Int(i64::from(evidence.manually_reviewed))),
        (7, Value::Int(i64::from(evidence.fixture_tested))),
        (8, Value::Int(i64::from(evidence.emulator_tested))),
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
        return Err(RuntimeCatalogCompileError::Path {
            path: "region purpose".to_owned(),
        });
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
    .ok_or(RuntimeCatalogCompileError::Path {
        path: "protocol".to_owned(),
    })
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
