#[cfg(test)]
#[path = "plans/identity_tests.rs"]
mod identity_tests;
#[path = "plans/metadata_fields.rs"]
mod metadata_fields;
#[cfg(test)]
#[path = "plans/pagination_tests.rs"]
mod pagination_tests;
use metadata_fields::*;
mod binding_alternatives;
mod bounds;
mod capabilities;
mod context;
mod dependencies;
mod emissions;
mod fixed_input;
mod identity_text;
mod map_exclusions;
mod model_links;
mod page_codec;
mod protocol_check;
mod routing;
mod row_policy;
mod selection;
mod stages;
mod tag_endpoint;
mod tags;
mod validation;
pub(super) use model_links::validate as validate_model_links;
use page_codec::page;
#[cfg(test)]
use page_codec::token;
#[cfg(test)]
mod tests;
use super::super::types::{
    BindingPlan, BindingSource, BindingValue, CallablePlan, EndpointClass, EndpointException,
    EndpointPathMode, EndpointTemplate, IdentityPart, IdentityPatternCache, IdentityPlan, ItemPlan,
    PagePlan, PageSizeBinding, PlanParts, PlanPurpose, PlanScope, ReviewProvenance,
    RuntimeProtocol, TokenBinding, TokenPlacement, TokenType,
};
use super::{RuntimeCatalogError, list_value, map, metadata, services::Services};
use crate::{
    inventory::Kind,
    record::{Exclusions, ValueRef},
};
use bounds::{optional_u32, positive_u32};
pub(super) fn read<'a>(
    services: &Services<'a>,
    list: crate::record::ListRef<'_>,
) -> Result<Vec<CallablePlan<'a>>, RuntimeCatalogError> {
    let mut cache = IdentityPatternCache::default();
    let mut plans = metadata::bounded(
        list,
        |value| plan(services, value, &mut cache),
        metadata::MAX_ITEMS,
    )?;
    dependencies::validate(&mut plans)?;
    Ok(plans)
}

fn plan<'a>(
    services: &Services<'a>,
    value: ValueRef<'_>,
    cache: &mut IdentityPatternCache<'a>,
) -> Result<CallablePlan<'a>, RuntimeCatalogError> {
    let fields = map(value)?;
    metadata::only_fields(
        fields,
        &[
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
        ],
    )?;
    let service_name = metadata::text(fields, 1)?;
    let operation_name = metadata::text(fields, 2)?;
    let (service, operation) = services.operation(&service_name, &operation_name)?;
    decode_plan(
        fields,
        service,
        operation,
        service_name,
        operation_name,
        cache,
    )
}

fn decode_plan<'a>(
    fields: crate::record::MapRef<'_>,
    service: crate::catalog::Service<'a>,
    operation: crate::catalog::Operation<'a>,
    service_name: String,
    operation_name: String,
    cache: &mut IdentityPatternCache<'a>,
) -> Result<CallablePlan<'a>, RuntimeCatalogError> {
    let kind = Kind::parse(&metadata::text(fields, 3)?)?;
    let scope = scope(metadata::integer(fields, 4)?)?;
    let items = selection::item(metadata::map_field(fields, 5)?)?;
    let tags = tags::read(fields, service, operation, &items)?;
    let purpose_fields = metadata::map_field(fields, 12)?;
    let purpose = if metadata::integer(purpose_fields, 1)? == 3 {
        PlanPurpose::TagEndpoint(tag_endpoint::read(purpose_fields, service, operation)?)
    } else {
        purpose(purpose_fields)?
    };
    let emission = selection::emission(fields)?;
    let identity = identity_for(metadata::map_field(fields, 6)?, &purpose, emission)?;
    let bindings = bindings(metadata::list(fields, 7)?)?;
    let pagination = page(metadata::map_field(fields, 8)?)?;
    validation::exclusions(service, operation, &items, metadata::list(fields, 9)?)?;
    let exclusions = map_exclusions::read(
        fields,
        service,
        operation,
        &items,
        exclusions(metadata::list(fields, 9)?)?,
    )?;
    let endpoint = endpoint(metadata::map_field(fields, 10)?)?;
    let review = review(metadata::map_field(fields, 11)?)?;
    let protocol = protocol(metadata::integer(fields, 13)?)?;
    let (projection, classification) = capabilities::read(fields, service, operation, &items)?;
    let resources = super::super::types::ResourceEmissions::new(
        super::super::types::ResourceEmissionPlan::primary(
            &kind,
            &items,
            &identity,
            tags.as_ref(),
            &exclusions,
            projection.as_ref(),
            classification.as_ref(),
        ),
        emissions::additional(fields, &service_name, service, operation)?,
    )?;
    let identity_text = resources
        .iter()
        .map(|emission| {
            Ok((
                Box::from(emission.key()),
                identity_text::compile(
                    service,
                    operation,
                    emission.items(),
                    emission.identity(),
                    cache,
                )?,
            ))
        })
        .collect::<Result<Box<[_]>, RuntimeCatalogError>>()?;
    protocol_check::validate(service, protocol)?;
    let parts = PlanParts {
        service_name: service_name.into_boxed_str(),
        operation_name: operation_name.into_boxed_str(),
        service,
        operation,
        kind,
        scope,
        items,
        identity,
        tags,
        bindings: bindings.into_boxed_slice(),
        pagination,
        exclusions,
        endpoint,
        review,
        purpose,
        protocol,
        facet_precedence: 0,
        routing: routing::read(fields)?,
        branches: routing::branches(fields)?,
        projection,
        classification,
        emission,
        resources,
        identity_text,
        row_policy: row_policy::read(fields)?,
        context_projection: selection::projection(fields)?,
    };
    validation::validate(&parts)?;
    Ok(CallablePlan::new(parts))
}

fn identity_for(
    fields: crate::record::MapRef<'_>,
    purpose: &PlanPurpose,
    emission: super::super::Emission,
) -> Result<IdentityPlan, RuntimeCatalogError> {
    if matches!(
        purpose,
        PlanPurpose::TagEndpoint(_) | PlanPurpose::EndpointDiscovery
    ) || emission == super::super::Emission::DiscoveryOnly
    {
        metadata::only_fields(fields, &[1, 2])?;
        if metadata::integer(fields, 1)? != 2 || metadata::list(fields, 2)?.count()? != 0 {
            return Err(RuntimeCatalogError::Capability);
        }
        return Ok(IdentityPlan::Key(Box::new([])));
    }
    selection::identity(fields)
}

fn scope(value: i64) -> Result<PlanScope, RuntimeCatalogError> {
    match value {
        0 => Ok(PlanScope::Region),
        1 => Ok(PlanScope::Account),
        2 => Ok(PlanScope::HomeRegion),
        _ => Err(RuntimeCatalogError::Capability),
    }
}

fn item_plan(map_value: crate::record::MapRef<'_>) -> Result<ItemPlan, RuntimeCatalogError> {
    metadata::only_fields(map_value, &[1, 2])?;
    match metadata::integer(map_value, 1)? {
        0 if map_value
            .get_field(crate::record::FieldId::new(2)?)?
            .is_none() =>
        {
            Ok(ItemPlan::Root)
        }
        1 => Ok(ItemPlan::Path(metadata::id_path(metadata::list(
            map_value, 2,
        )?)?)),
        _ => Err(RuntimeCatalogError::Capability),
    }
}

fn identity(map_value: crate::record::MapRef<'_>) -> Result<IdentityPlan, RuntimeCatalogError> {
    let identity_type = metadata::integer(map_value, 1)?;
    metadata::only_fields(map_value, &[1, 2])?;
    match identity_type {
        0 => Ok(IdentityPlan::Arn(metadata::selected_path(metadata::list(
            map_value, 2,
        )?)?)),
        1 => {
            let parts = metadata::bounded(metadata::list(map_value, 2)?, identity_part, 64)?;
            if parts.is_empty() {
                return Err(RuntimeCatalogError::Capability);
            }
            Ok(IdentityPlan::Template(parts.into_boxed_slice()))
        }
        2 => {
            let paths = metadata::bounded(
                metadata::list(map_value, 2)?,
                |entry| metadata::selected_path(list_value(entry)?),
                8,
            )?;
            if paths.is_empty() {
                return Err(RuntimeCatalogError::Capability);
            }
            Ok(IdentityPlan::Key(paths.into_boxed_slice()))
        }
        _ => Err(RuntimeCatalogError::Capability),
    }
}

fn identity_part(value: ValueRef<'_>) -> Result<IdentityPart, RuntimeCatalogError> {
    let fields = map(value)?;
    let kind = metadata::integer(fields, 1)?;
    match kind {
        0 => {
            metadata::only_fields(fields, &[1, 2])?;
            Ok(IdentityPart::Literal(metadata::text(fields, 2)?))
        }
        1 => {
            metadata::only_fields(fields, &[1, 2])?;
            Ok(IdentityPart::Field(metadata::selected_path(
                metadata::list(fields, 2)?,
            )?))
        }
        2..=4 => {
            metadata::only_fields(fields, &[1])?;
            match kind {
                2 => Ok(IdentityPart::Partition),
                3 => Ok(IdentityPart::Account),
                _ => Ok(IdentityPart::Region),
            }
        }
        _ => Err(RuntimeCatalogError::Capability),
    }
}

fn bindings(list: crate::record::ListRef<'_>) -> Result<Vec<BindingPlan>, RuntimeCatalogError> {
    metadata::bounded(list, binding, 4096)
}

fn binding(value: ValueRef<'_>) -> Result<BindingPlan, RuntimeCatalogError> {
    let fields = map(value)?;
    let path = metadata::id_path(metadata::list(fields, 1)?)?;
    let source_type = metadata::integer(fields, 2)?;
    let source = match source_type {
        0 => {
            metadata::only_fields(fields, &[1, 2])?;
            BindingSource::Account
        }
        1 => {
            metadata::only_fields(fields, &[1, 2])?;
            BindingSource::Region
        }
        2 => {
            metadata::only_fields(fields, &[1, 2, 3])?;
            BindingSource::Fixed(binding_value(metadata::field(fields, 3)?)?)
        }
        3 => {
            metadata::only_fields(fields, &[1, 2, 3])?;
            BindingSource::ParentIdentity {
                path: metadata::selected_path(metadata::list(fields, 3)?)?,
            }
        }
        4 => {
            metadata::only_fields(fields, &[1, 2, 3, 4, 5])?;
            BindingSource::Operation {
                service: metadata::text(fields, 3)?.into_boxed_str(),
                operation: metadata::text(fields, 4)?.into_boxed_str(),
                path: metadata::selected_path(metadata::list(fields, 5)?)?,
            }
        }
        5..=7 => selection::binding(fields)?,
        8 => {
            metadata::only_fields(fields, &[1, 2, 3, 4, 5, 6, 7])?;
            let arn_service = metadata::text(fields, 6)?;
            let resource_kind = match (arn_service.as_str(), metadata::integer(fields, 7)?) {
                ("sns", 0) => super::super::ArnResourceKind::SnsSubscription,
                ("sqs", 1) => super::super::ArnResourceKind::SqsQueue,
                ("sns", 2) => super::super::ArnResourceKind::SnsTopic,
                _ => return Err(RuntimeCatalogError::Capability),
            };
            BindingSource::OperationArn {
                service: metadata::text(fields, 3)?.into_boxed_str(),
                operation: metadata::text(fields, 4)?.into_boxed_str(),
                path: metadata::selected_path(metadata::list(fields, 5)?)?,
                arn_service: arn_service.into_boxed_str(),
                resource_kind,
            }
        }
        9 => selection::transformed(fields)?,
        10 => selection::url_binding(fields)?,
        11 => {
            metadata::only_fields(fields, &[1, 2])?;
            BindingSource::RegionList
        }
        12 => binding_alternatives::decode(fields)?,
        13 => {
            metadata::only_fields(fields, &[1, 2, 3, 4, 5, 6, 7])?;
            let branch = metadata::text(fields, 6)?;
            if !routing::valid_branch_name(&branch) {
                return Err(RuntimeCatalogError::Capability);
            }
            BindingSource::ContextFromBranch {
                service: metadata::text(fields, 3)?.into_boxed_str(),
                operation: metadata::text(fields, 4)?.into_boxed_str(),
                branch: branch.into_boxed_str(),
                source: match metadata::integer(fields, 7)? {
                    5 => crate::catalog::runtime::ContextSource::SelectedItem,
                    6 => crate::catalog::runtime::ContextSource::Response,
                    7 => crate::catalog::runtime::ContextSource::Request,
                    _ => return Err(RuntimeCatalogError::Capability),
                },
                path: metadata::selected_path(metadata::list(fields, 5)?)?,
            }
        }
        _ => return Err(RuntimeCatalogError::Capability),
    };
    Ok(BindingPlan { path, source })
}

fn binding_value(value: ValueRef<'_>) -> Result<BindingValue, RuntimeCatalogError> {
    fixed_input::decode(value)
}

fn exclusions(list: crate::record::ListRef<'_>) -> Result<Exclusions, RuntimeCatalogError> {
    let paths = metadata::bounded(
        list,
        |value| metadata::id_path(list_value(value)?).map(|path| path.into_vec()),
        4096,
    )?;
    Ok(Exclusions::new(paths)?)
}
