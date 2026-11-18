use super::{
    PagePlan, PageSizeBinding, RuntimeCatalogError, TokenBinding, TokenPlacement, TokenType,
    ValueRef, map, metadata, optional_u32, positive_u32, selection,
};
use crate::catalog::runtime::{PageSizeType, TokenSelection};
pub(super) fn page(map_value: crate::record::MapRef<'_>) -> Result<PagePlan, RuntimeCatalogError> {
    let items = selection::item(metadata::map_field(map_value, 1)?)?;
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

pub(super) fn token(value: ValueRef<'_>) -> Result<TokenBinding, RuntimeCatalogError> {
    let fields = map(value)?;
    let request_path = metadata::id_path(metadata::list(fields, 1)?)?;
    let response_path = metadata::id_path(metadata::list(fields, 2)?)?;
    let value_type = token_type(metadata::integer(fields, 3)?)?;
    let request_placement = placement(metadata::map_field(fields, 4)?)?;
    let response_placement = placement(metadata::map_field(fields, 5)?)?;
    let optional = token_optional(fields)?;
    let selection = token_selection(fields)?;
    metadata::only_fields(fields, &[1, 2, 3, 4, 5, 6, 7])?;
    Ok(TokenBinding {
        request_path,
        response_path,
        value_type,
        request_placement,
        response_placement,
        optional,
        selection,
    })
}

fn token_optional(fields: crate::record::MapRef<'_>) -> Result<bool, RuntimeCatalogError> {
    match fields.get_field(crate::record::FieldId::new(6)?)? {
        None => Ok(false),
        Some(ValueRef::Bool(value)) => Ok(value),
        _ => Err(RuntimeCatalogError::Capability),
    }
}

fn token_selection(
    fields: crate::record::MapRef<'_>,
) -> Result<TokenSelection, RuntimeCatalogError> {
    match fields.get_field(crate::record::FieldId::new(7)?)? {
        None | Some(ValueRef::Int(0)) => Ok(TokenSelection::Value),
        Some(ValueRef::Int(1)) => Ok(TokenSelection::LastListItem),
        _ => Err(RuntimeCatalogError::Capability),
    }
}

fn page_tokens(
    fields: crate::record::MapRef<'_>,
) -> Result<Vec<TokenBinding>, RuntimeCatalogError> {
    let tokens = metadata::bounded(metadata::list(fields, 2)?, token, 8)?;
    let duplicates = tokens.iter().enumerate().any(|(index, current)| {
        tokens
            .iter()
            .take(index)
            .any(|previous| previous.request_path == current.request_path)
    });
    if duplicates || (!tokens.is_empty() && tokens.iter().all(|token| token.optional)) {
        return Err(RuntimeCatalogError::Capability);
    }
    Ok(tokens)
}

fn page_size(map_value: crate::record::MapRef<'_>) -> Result<PageSizeBinding, RuntimeCatalogError> {
    let path = metadata::id_path(metadata::list(map_value, 1)?)?;
    let max = positive_u32(metadata::integer(map_value, 2)?)?;
    let placement = placement(metadata::map_field(map_value, 3)?)?;
    let value_type = match map_value.get_field(crate::record::FieldId::new(4)?)? {
        None => PageSizeType::Integer,
        Some(ValueRef::Int(0)) => PageSizeType::String,
        Some(ValueRef::Int(1)) => PageSizeType::Integer,
        _ => return Err(RuntimeCatalogError::Capability),
    };
    metadata::only_fields(map_value, &[1, 2, 3, 4])?;
    Ok(PageSizeBinding {
        path,
        max,
        value_type,
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
