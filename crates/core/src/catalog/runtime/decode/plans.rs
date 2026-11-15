#[cfg(test)]
#[path = "plans/identity_tests.rs"]
mod identity_tests;
#[cfg(test)]
#[path = "plans/pagination_tests.rs"]
mod pagination_tests;
#[path = "plans/metadata_fields.rs"]
mod metadata_fields;
use metadata_fields::*;
mod bounds;
mod protocol_check;
#[cfg(test)]
mod tests;
use super::super::types::{
    BindingPlan, BindingSource, BindingValue, CallablePlan, EndpointClass, EndpointPathMode,
    EndpointTemplate, IdentityPart, IdentityPlan, ItemPlan, PagePlan, PageSizeBinding, PlanParts,
    PlanPurpose, PlanScope, ReviewProvenance, RuntimeProtocol, TokenBinding, TokenPlacement,
    TokenType,
};
use super::{RuntimeCatalogError, catalog_operation, list_value, map, metadata};
use crate::{
    catalog::Catalog,
    inventory::Kind,
    record::{Exclusions, ValueRef},
};
use bounds::{optional_u32, positive_u32};
pub(super) fn read<'a>(
    catalog: Catalog<'a>,
    list: crate::record::ListRef<'_>,
) -> Result<Vec<CallablePlan<'a>>, RuntimeCatalogError> {
    metadata::bounded(list, |value| plan(catalog, value), metadata::MAX_ITEMS)
}

fn plan<'a>(
    catalog: Catalog<'a>,
    value: ValueRef<'_>,
) -> Result<CallablePlan<'a>, RuntimeCatalogError> {
    let fields = map(value)?;
    metadata::only_fields(fields, &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13])?;
    let service_name = metadata::text(fields, 1)?;
    let operation_name = metadata::text(fields, 2)?;
    let (service, operation) = catalog_operation(catalog, &service_name, &operation_name)?;
    decode_plan(fields, service, operation, service_name, operation_name)
}

fn decode_plan<'a>(
    fields: crate::record::MapRef<'_>,
    service: crate::catalog::Service<'a>,
    operation: crate::catalog::Operation<'a>,
    service_name: String,
    operation_name: String,
) -> Result<CallablePlan<'a>, RuntimeCatalogError> {
    let kind = Kind::parse(&metadata::text(fields, 3)?)?;
    let scope = scope(metadata::integer(fields, 4)?)?;
    let items = item_plan(metadata::map_field(fields, 5)?)?;
    let identity = identity(metadata::map_field(fields, 6)?)?;
    let bindings = bindings(metadata::list(fields, 7)?)?;
    let pagination = page(metadata::map_field(fields, 8)?)?;
    let exclusions = exclusions(metadata::list(fields, 9)?)?;
    let endpoint = endpoint(metadata::map_field(fields, 10)?)?;
    let review = review(metadata::map_field(fields, 11)?)?;
    let purpose = purpose(metadata::map_field(fields, 12)?)?;
    let protocol = protocol(metadata::integer(fields, 13)?)?;
    protocol_check::validate(service, protocol)?;
    Ok(CallablePlan::new(PlanParts {
        service_name: service_name.into_boxed_str(),
        operation_name: operation_name.into_boxed_str(),
        service,
        operation,
        kind,
        scope,
        items,
        identity,
        bindings: bindings.into_boxed_slice(),
        pagination,
        exclusions,
        endpoint,
        review,
        purpose,
        protocol,
    }))
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
        1 => Ok(IdentityPlan::Template(
            metadata::bounded(metadata::list(map_value, 2)?, identity_part, 64)?.into_boxed_slice(),
        )),
        2 => Ok(IdentityPlan::Key(
            metadata::bounded(
                metadata::list(map_value, 2)?,
                |entry| metadata::selected_path(list_value(entry)?),
                8,
            )?
            .into_boxed_slice(),
        )),
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
            Ok(IdentityPart::Field(metadata::selected_path(metadata::list(fields, 2)?)?))
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
        _ => return Err(RuntimeCatalogError::Capability),
    };
    Ok(BindingPlan { path, source })
}

fn binding_value(value: ValueRef<'_>) -> Result<BindingValue, RuntimeCatalogError> {
    match value {
        ValueRef::Text(value) => Ok(BindingValue::String(value.to_owned())),
        ValueRef::Int(value) => Ok(BindingValue::Integer(value)),
        ValueRef::Bool(value) => Ok(BindingValue::Boolean(value)),
        ValueRef::List(values) => Ok(BindingValue::Strings(
            metadata::bounded(values, super::text_value, 16)?.into_boxed_slice(),
        )),
        _ => Err(RuntimeCatalogError::Capability),
    }
}

fn page(map_value: crate::record::MapRef<'_>) -> Result<PagePlan, RuntimeCatalogError> {
    let items = item_plan(metadata::map_field(map_value, 1)?)?;
    let tokens = page_tokens(map_value)?;
    let page_size = match map_value.get_field(crate::record::FieldId::new(3)?)? {
        Some(value) => Some(page_size(map(value)?)?),
        None => None,
    };
    let max_page_size = optional_u32(map_value, 4)?;
    let truncated_path = match map_value.get_field(crate::record::FieldId::new(5)?)? {
        Some(ValueRef::List(path)) => Some(metadata::id_path(path)?),
        Some(ValueRef::Null) | None => None,
        _ => return Err(RuntimeCatalogError::Capability),
    };
    metadata::only_fields(map_value, &[1, 2, 3, 4, 5])?;
    Ok(PagePlan {
        items,
        tokens: tokens.into_boxed_slice(),
        page_size,
        max_page_size,
        truncated_path,
    })
}

fn token(value: ValueRef<'_>) -> Result<TokenBinding, RuntimeCatalogError> {
    let fields = map(value)?;
    let request_path = metadata::id_path(metadata::list(fields, 1)?)?;
    let response_path = metadata::id_path(metadata::list(fields, 2)?)?;
    let value_type = token_type(metadata::integer(fields, 3)?)?;
    let request_placement = placement(metadata::map_field(fields, 4)?)?;
    let response_placement = placement(metadata::map_field(fields, 5)?)?;
    let optional = token_optional(fields)?;
    metadata::only_fields(fields, &[1, 2, 3, 4, 5, 6])?;
    Ok(TokenBinding {
        request_path,
        response_path,
        value_type,
        request_placement,
        response_placement,
        optional,
    })
}

fn token_optional(fields: crate::record::MapRef<'_>) -> Result<bool, RuntimeCatalogError> {
    match fields.get_field(crate::record::FieldId::new(6)?)? {
        None => Ok(false),
        Some(ValueRef::Bool(value)) => Ok(value),
        _ => Err(RuntimeCatalogError::Capability),
    }
}

fn page_tokens(fields: crate::record::MapRef<'_>) -> Result<Vec<TokenBinding>, RuntimeCatalogError> {
    let tokens = metadata::bounded(metadata::list(fields, 2)?, token, 8)?;
    if !tokens.is_empty() && tokens.iter().all(|token| token.optional) {
        return Err(RuntimeCatalogError::Capability);
    }
    Ok(tokens)
}

fn page_size(map_value: crate::record::MapRef<'_>) -> Result<PageSizeBinding, RuntimeCatalogError> {
    let path = metadata::id_path(metadata::list(map_value, 1)?)?;
    let max = positive_u32(metadata::integer(map_value, 2)?)?;
    let placement = placement(metadata::map_field(map_value, 3)?)?;
    metadata::only_fields(map_value, &[1, 2, 3])?;
    Ok(PageSizeBinding {
        path,
        max,
        placement,
    })
}

fn token_type(value: i64) -> Result<TokenType, RuntimeCatalogError> {
    match value {
        0 => Ok(TokenType::String),
        1 => Ok(TokenType::Integer),
        2 => Ok(TokenType::Boolean),
        _ => Err(RuntimeCatalogError::Capability),
    }
}

fn placement(map_value: crate::record::MapRef<'_>) -> Result<TokenPlacement, RuntimeCatalogError> {
    let placement = match metadata::integer(map_value, 1)? {
        0 => TokenPlacement::Path(metadata::text(map_value, 2)?),
        1 => TokenPlacement::Query(metadata::text(map_value, 2)?),
        2 => TokenPlacement::Header(metadata::text(map_value, 2)?),
        3 => TokenPlacement::Body(metadata::id_path(metadata::list(map_value, 2)?)?),
        _ => return Err(RuntimeCatalogError::Capability),
    };
    metadata::only_fields(map_value, &[1, 2])?;
    Ok(placement)
}

fn exclusions(list: crate::record::ListRef<'_>) -> Result<Exclusions, RuntimeCatalogError> {
    let paths = metadata::bounded(
        list,
        |value| metadata::id_path(list_value(value)?).map(|path| path.into_vec()),
        4096,
    )?;
    Ok(Exclusions::new(paths)?)
}
