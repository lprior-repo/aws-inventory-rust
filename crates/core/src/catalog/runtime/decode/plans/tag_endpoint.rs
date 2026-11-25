use super::{RuntimeCatalogError, metadata, tags};
use crate::{
    catalog::runtime::{
        CallablePlan, ItemPlan, PlanPurpose, TagEndpointParentPlan, TagEndpointPlan,
        TagEndpointRequestPlan, TagEndpointSource, TagPlan,
    },
    catalog::{Operation, Service},
    record::{FieldId, MapRef, ValueRef},
};

pub(super) fn read(
    fields: MapRef<'_>,
    service: Service<'_>,
    operation: Operation<'_>,
) -> Result<TagEndpointPlan, RuntimeCatalogError> {
    metadata::only_fields(fields, &[1, 2, 4])?;
    let parents = metadata::bounded(
        metadata::list(fields, 2)?,
        |entry| parent(super::super::map(entry)?, service, operation),
        64,
    )?;
    if parents.is_empty() {
        return Err(RuntimeCatalogError::Capability);
    }
    if parents.iter().enumerate().any(|(index, parent)| {
        parents
            .iter()
            .take(index)
            .any(|prior| prior.operation == parent.operation)
    }) {
        return Err(RuntimeCatalogError::Capability);
    }
    let tags = selector(metadata::map_field(fields, 4)?, service, operation)?;
    Ok(TagEndpointPlan {
        parents: parents.into_boxed_slice(),
        tags,
    })
}

fn parent(
    fields: MapRef<'_>,
    service: Service<'_>,
    operation: Operation<'_>,
) -> Result<TagEndpointParentPlan, RuntimeCatalogError> {
    metadata::only_fields(fields, &[1, 2])?;
    let name = metadata::text(fields, 1)?;
    if name.is_empty() || name.len() > 128 || name == operation.name()? {
        return Err(RuntimeCatalogError::Capability);
    }
    let request = match metadata::field(fields, 2)? {
        ValueRef::Null => None,
        ValueRef::Map(request) => {
            metadata::only_fields(request, &[1, 2, 3])?;
            let input = metadata::id_path(metadata::list(request, 1)?)?;
            string(service, tags::resolve(service, operation.input()?, &input)?)?;
            let parent = metadata::selected_path(metadata::list(request, 2)?)?;
            let source = match metadata::integer(request, 3)? {
                0 => TagEndpointSource::Response,
                1 => TagEndpointSource::Request,
                2 => TagEndpointSource::OwnerIdentity,
                _ => return Err(RuntimeCatalogError::Capability),
            };
            Some(TagEndpointRequestPlan {
                input,
                parent,
                source,
            })
        }
        _ => return Err(RuntimeCatalogError::Capability),
    };
    Ok(TagEndpointParentPlan {
        operation: name.into_boxed_str(),
        request,
    })
}

fn selector(
    fields: MapRef<'_>,
    service: Service<'_>,
    operation: Operation<'_>,
) -> Result<TagPlan, RuntimeCatalogError> {
    let path = metadata::id_path(metadata::list(fields, 2)?)?;
    let shape = tags::shape(service, tags::resolve(service, operation.output()?, &path)?)?;
    match metadata::integer(fields, 1)? {
        0 if shape.kind()? == "list" => {
            metadata::only_fields(fields, &[1, 2, 3, 4])?;
            let key = field(fields, 3)?;
            let value = field(fields, 4)?;
            if key == value {
                return Err(RuntimeCatalogError::Capability);
            }
            let item = tags::shape(
                service,
                shape.element()?.ok_or(RuntimeCatalogError::Capability)?,
            )?;
            if item.kind()? != "structure" {
                return Err(RuntimeCatalogError::Capability);
            }
            string(service, tags::member(item, key)?.target)?;
            string(service, tags::member(item, value)?.target)?;
            Ok(TagPlan::List { path, key, value })
        }
        1 if shape.kind()? == "map" => {
            metadata::only_fields(fields, &[1, 2])?;
            string(
                service,
                shape.map_key()?.ok_or(RuntimeCatalogError::Capability)?,
            )?;
            string(
                service,
                shape.map_value()?.ok_or(RuntimeCatalogError::Capability)?,
            )?;
            Ok(TagPlan::Map { path })
        }
        _ => Err(RuntimeCatalogError::Capability),
    }
}

fn field(fields: MapRef<'_>, number: u32) -> Result<FieldId, RuntimeCatalogError> {
    let value = u32::try_from(metadata::integer(fields, number)?)
        .map_err(|_| RuntimeCatalogError::Capability)?;
    FieldId::new(value).map_err(RuntimeCatalogError::from)
}

fn string(service: Service<'_>, target: crate::catalog::Target) -> Result<(), RuntimeCatalogError> {
    match tags::target_kind(service, target)? {
        "string" | "enum" => Ok(()),
        _ => Err(RuntimeCatalogError::Capability),
    }
}

pub(super) fn validate_tag(parts: &super::PlanParts<'_>) -> Result<(), RuntimeCatalogError> {
    if parts.items != ItemPlan::Root
        || parts.tags.is_some()
        || parts.bindings.iter().any(|binding| {
            !matches!(
                binding.source,
                super::BindingSource::Account
                    | super::BindingSource::Region
                    | super::BindingSource::Fixed(_)
            )
        })
    {
        return Err(RuntimeCatalogError::Capability);
    }
    let PlanPurpose::TagEndpoint(endpoint) = &parts.purpose else {
        return Err(RuntimeCatalogError::Capability);
    };
    for request in endpoint
        .parents
        .iter()
        .filter_map(|parent| parent.request.as_ref())
    {
        if parts
            .bindings
            .iter()
            .any(|binding| overlaps(&request.input, &binding.path))
            || parts
                .pagination
                .tokens
                .iter()
                .any(|token| overlaps(&request.input, &token.request_path))
            || parts
                .pagination
                .page_size
                .as_ref()
                .is_some_and(|size| overlaps(&request.input, &size.path))
        {
            return Err(RuntimeCatalogError::Capability);
        }
    }
    Ok(())
}

fn overlaps(left: &[FieldId], right: &[FieldId]) -> bool {
    left.starts_with(right) || right.starts_with(left)
}

pub(super) fn validate_parents(plans: &[CallablePlan<'_>]) -> Result<(), RuntimeCatalogError> {
    plans.iter().try_for_each(|child| {
        let PlanPurpose::TagEndpoint(endpoint) = child.purpose() else {
            return Ok(());
        };
        endpoint.parents.iter().try_for_each(|source| {
            let parent = plans
                .iter()
                .find(|parent| {
                    parent.service_name() == child.service_name()
                        && parent.operation_name() == source.operation.as_ref()
                        && matches!(parent.purpose(), PlanPurpose::Enumeration)
                        && parent.emission() != crate::catalog::runtime::Emission::DiscoveryOnly
                })
                .ok_or(RuntimeCatalogError::Capability)?;
            source
                .request
                .as_ref()
                .map_or(Ok(()), |request| match request.source {
                    TagEndpointSource::Response => super::validation::source_compatible(
                        parent,
                        &request.parent,
                        child,
                        &request.input,
                    ),
                    TagEndpointSource::Request => string(
                        parent.service(),
                        tags::resolve(
                            parent.service(),
                            parent.operation().input()?,
                            &request.parent,
                        )?,
                    ),
                    TagEndpointSource::OwnerIdentity if request.parent.is_empty() => Ok(()),
                    TagEndpointSource::OwnerIdentity => Err(RuntimeCatalogError::Capability),
                })
        })
    })
}
