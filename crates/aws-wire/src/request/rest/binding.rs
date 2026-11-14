mod header;
mod protected;

use super::{EncodeError, RequestContext, field_value, uri, write_header};
use http::header::HeaderMap;
use inventory_core::{catalog::HttpBinding, record::ValueRef};

pub(super) fn headers(
    context: RequestContext<'_>,
    target: &mut String,
    headers: &mut HeaderMap,
) -> Result<(), EncodeError> {
    let input = context.operation.input()?;
    let inventory_core::catalog::Target::Shape(index) = input else {
        return Ok(());
    };
    let structure = context
        .service
        .shape(index)?
        .ok_or(EncodeError::MissingInput)?;
    let protected = protected::names(context, structure)?;
    for member in structure.members()? {
        let member = member?;
        let value = match field_value(context, member)? {
            Some(value) if !matches!(value, ValueRef::Null) => value,
            _ if member.wire()?.idempotency_token()? => match context.idempotency_token {
                Some(value) => ValueRef::Text(value),
                None => continue,
            },
            _ => continue,
        };
        match member.wire()?.binding()? {
            HttpBinding::Header(header) => {
                let format = uri::target_timestamp(
                    context,
                    member.target,
                    member.wire()?.timestamp_format()?,
                )?;
                write_header(
                    headers,
                    header,
                    &header::value(context, value, member.target, member.id, format)?,
                )?
            }
            HttpBinding::PrefixHeaders(prefix) => {
                header::prefix(headers, prefix, value, member.id)?
            }
            HttpBinding::Query(query) => uri::query::append_member_query(
                context,
                target,
                query,
                value,
                member.target,
                member.id,
                member.wire()?.timestamp_format()?,
            )?,
            HttpBinding::QueryParams => {
                query_params(context, target, value, member.target, member.id, &protected)?
            }
            HttpBinding::Payload
            | HttpBinding::ResponseCode
            | HttpBinding::Label
            | HttpBinding::Document => {}
        }
    }
    Ok(())
}

fn query_params(
    context: RequestContext<'_>,
    target: &mut String,
    value: ValueRef<'_>,
    map_target: inventory_core::catalog::Target,
    field: inventory_core::record::FieldId,
    protected: &[&str],
) -> Result<(), EncodeError> {
    let ValueRef::Map(map) = value else {
        return Err(EncodeError::InvalidValue {
            field: Some(field),
            reason: "query parameters must be a map",
        });
    };
    let (value_target, sparse) = query_map_value_target(context, map_target, field)?;
    for (index, entry) in map.iter()?.enumerate() {
        if index == uri::query::MAX_QUERY_ITEMS {
            return Err(EncodeError::Limit("frames"));
        }
        let (key, value) = entry?;
        let inventory_core::record::KeyRef::Name(key) = key else {
            return Err(EncodeError::InvalidValue {
                field: Some(field),
                reason: "query parameter key must be text",
            });
        };
        if protected.binary_search(&key).is_ok() {
            continue;
        }

        if matches!(value, ValueRef::Null) {
            if sparse {
                continue;
            }
            return Err(EncodeError::InvalidValue {
                field: Some(field),
                reason: "null in non-sparse query map",
            });
        }
        uri::query::append_member_query(context, target, key, value, value_target, field, None)?;
    }
    Ok(())
}

fn query_map_value_target(
    context: RequestContext<'_>,
    map_target: inventory_core::catalog::Target,
    field: inventory_core::record::FieldId,
) -> Result<(inventory_core::catalog::Target, bool), EncodeError> {
    let inventory_core::catalog::Target::Shape(index) = map_target else {
        return Err(EncodeError::InvalidValue {
            field: Some(field),
            reason: "query parameters target must be a map shape",
        });
    };
    let shape = context
        .service
        .shape(index)?
        .ok_or(EncodeError::MissingInput)?;
    let value_target = shape.map_value()?.ok_or(EncodeError::InvalidValue {
        field: Some(field),
        reason: "query parameters shape has no map value",
    })?;
    let value_wire = shape.map_value_wire()?;
    if shape.map_key_wire()?.streaming()? || value_wire.streaming()? {
        return Err(EncodeError::Unsupported("streaming query parameters"));
    }
    Ok((value_target, shape.wire()?.sparse()?))
}
