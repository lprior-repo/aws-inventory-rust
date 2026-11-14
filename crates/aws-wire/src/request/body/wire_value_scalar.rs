use super::{
    wire::{ValueTask, is_text_media_type},
    wire_decimal::{write_big_decimal, write_big_integer},
    wire_scalar::{
        json_or_cbor_text, validate_integer, write_bool, write_float, write_integer, write_null,
    },
    wire_time::write_time,
    wire_value::{Resolved, invalid_value},
};
use crate::request::{EncodeError, Protocol, common::BoundedBody};
pub(super) fn write_null_value(
    protocol: Protocol,
    item: ValueTask<'_>,
    resolved: Resolved<'_>,
    body: &mut BoundedBody,
) -> Result<(), EncodeError> {
    if resolved.kind == "document" || item.sparse {
        write_null(protocol, body)
    } else {
        invalid_value(item)
    }
}

pub(super) fn write_bool_value(
    protocol: Protocol,
    value: bool,
    item: ValueTask<'_>,
    resolved: Resolved<'_>,
    body: &mut BoundedBody,
) -> Result<(), EncodeError> {
    if resolved.kind == "boolean" || resolved.kind == "document" {
        write_bool(protocol, value, body)
    } else {
        invalid_value(item)
    }
}

pub(super) fn write_integer_value(
    protocol: Protocol,
    value: i64,
    item: ValueTask<'_>,
    resolved: Resolved<'_>,
    body: &mut BoundedBody,
) -> Result<(), EncodeError> {
    if !matches!(
        resolved.kind,
        "byte" | "short" | "integer" | "long" | "bigInteger" | "intEnum" | "document"
    ) {
        return invalid_value(item);
    }
    if resolved.kind == "bigInteger" {
        return write_big_integer(protocol, value, body);
    }
    validate_integer(resolved.kind, value, item.field)?;
    write_integer(protocol, value, body)
}

pub(super) fn write_float_value(
    protocol: Protocol,
    value: f64,
    item: ValueTask<'_>,
    resolved: Resolved<'_>,
    body: &mut BoundedBody,
) -> Result<(), EncodeError> {
    if !matches!(resolved.kind, "float" | "double" | "document") {
        return invalid_value(item);
    }
    if resolved.kind == "float" && value.is_finite() && value.abs() > f64::from(f32::MAX) {
        return Err(EncodeError::InvalidValue {
            field: item.field,
            reason: "float is out of range",
        });
    }
    write_float(protocol, resolved.kind, value, body)
}

fn float_sentinel(value: &str, item: ValueTask<'_>) -> Result<f64, EncodeError> {
    match value {
        "NaN" => Ok(f64::NAN),
        "Infinity" => Ok(f64::INFINITY),
        "-Infinity" => Ok(f64::NEG_INFINITY),
        _ => Err(EncodeError::InvalidValue {
            field: item.field,
            reason: "invalid float sentinel",
        }),
    }
}

pub(super) fn write_text_value(
    protocol: Protocol,
    value: &str,
    item: ValueTask<'_>,
    resolved: Resolved<'_>,
    body: &mut BoundedBody,
) -> Result<(), EncodeError> {
    if matches!(resolved.kind, "float" | "double") {
        return write_float(protocol, resolved.kind, float_sentinel(value, item)?, body);
    }
    if resolved.kind == "bigDecimal" {
        return write_big_decimal(protocol, value, item.field, body);
    }
    if matches!(resolved.kind, "string" | "enum" | "document") {
        json_or_cbor_text(protocol, value, body)
    } else {
        invalid_value(item)
    }
}

pub(super) fn write_blob_value(
    protocol: Protocol,
    value: &[u8],
    item: ValueTask<'_>,
    resolved: Resolved<'_>,
    body: &mut BoundedBody,
) -> Result<(), EncodeError> {
    if resolved.kind != "blob" {
        return invalid_value(item);
    }
    if protocol == Protocol::RpcV2Cbor {
        return super::cbor::bytes(body, value);
    }
    write_json_blob(value, item, resolved.media_type, body)
}

fn write_json_blob(
    value: &[u8],
    item: ValueTask<'_>,
    media_type: Option<&str>,
    body: &mut BoundedBody,
) -> Result<(), EncodeError> {
    if media_type.is_some_and(is_text_media_type) {
        let text = std::str::from_utf8(value).map_err(|_| EncodeError::InvalidValue {
            field: item.field,
            reason: "text media type blob is not UTF-8",
        })?;
        return super::json::string(body, text);
    }
    body.write(b"\"")?;
    super::json::base64(body, value)?;
    body.write(b"\"")
}

pub(super) fn write_timestamp_value(
    protocol: Protocol,
    value: inventory_core::record::Time,
    item: ValueTask<'_>,
    resolved: Resolved<'_>,
    body: &mut BoundedBody,
) -> Result<(), EncodeError> {
    if resolved.kind == "timestamp" {
        write_time(protocol, value, resolved.timestamp_format, item.field, body)
    } else {
        invalid_value(item)
    }
}
