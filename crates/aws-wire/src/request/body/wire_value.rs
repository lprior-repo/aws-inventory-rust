use crate::request::{EncodeError, Protocol, common::BoundedBody};
use inventory_core::{catalog::{Service, Shape, Target}, record::{ValueRef}};
use super::{
    wire::{MAX_DEPTH, Task, ValueTask, collect_list, collect_map, is_text_media_type, timestamp_format_name},
    wire_scalar::{json_or_cbor_text, validate_integer, write_bool, write_float, write_integer, write_null},
    wire_shape::{push_list, push_map},
    wire_member::push_structure,
    wire_decimal::{write_big_decimal, write_big_integer},
    wire_time::write_time,
};
#[derive(Clone, Copy)]
struct Resolved<'a> {
    kind: &'a str,
    shape: Option<Shape<'a>>,
    timestamp_format: Option<&'a str>,
    media_type: Option<&'a str>,
}

pub(super) fn write_value(
    service: Service<'_>,
    protocol: Protocol,
    item: ValueTask<'_>,
    idempotency_token: Option<&str>,
    body: &mut BoundedBody,
    pending: &mut Vec<Task<'_>>,
) -> Result<(), EncodeError> {
    if item.depth > MAX_DEPTH {
        return Err(EncodeError::Limit("serialization depth"));
    }
    let resolved = resolve(service, item)?;
    match item.value {
        ValueRef::Null => write_null_value(protocol, item, resolved, body),
        ValueRef::Bool(value) => write_bool_value(protocol, value, item, resolved, body),
        ValueRef::Int(value) => write_integer_value(protocol, value, item, resolved, body),
        ValueRef::Float(value) => write_float_value(protocol, value.to_f64(), item, resolved, body),
        ValueRef::Text(value) => write_text_value(protocol, value, item, resolved, body),
        ValueRef::Bytes(value) => write_blob_value(protocol, value, item, resolved, body),
        ValueRef::Time(value) => write_timestamp_value(protocol, value, item, resolved, body),
        ValueRef::List(values) => write_list_value(protocol, values, item, resolved, body, pending),
        ValueRef::Map(values) => write_map_value(
            service, protocol, values, item, resolved, idempotency_token, body, pending,
        ),
    }
}

fn resolve<'a>(service: Service<'a>, item: ValueTask<'a>) -> Result<Resolved<'a>, EncodeError> {
    let (kind, shape) = match item.target {
        Target::Shape(index) => {
            let shape = service.shape(index)?.ok_or(EncodeError::InvalidValue {
                field: item.field,
                reason: "missing target shape",
            })?;
            (shape.kind()?, Some(shape))
        }
        Target::Prelude(name) => (name, None),
    };
    let wire = shape.map(|shape| shape.wire()).transpose()?;
    if wire.is_some_and(|wire| wire.streaming()) {
        return Err(EncodeError::Unsupported("streaming shape"));
    }
    let timestamp_format = match item.timestamp_format {
        Some(format) => Some(format),
        None => wire
            .map(|wire| wire.timestamp_format())
            .transpose()?
            .flatten()
            .map(timestamp_format_name),
    };
    let media_type = item.media_type.or(wire.map(|wire| wire.media_type()).transpose()?.flatten());
    Ok(Resolved { kind, shape, timestamp_format, media_type })
}

fn write_null_value(
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

fn write_bool_value(
    protocol: Protocol, value: bool, item: ValueTask<'_>, resolved: Resolved<'_>, body: &mut BoundedBody,
) -> Result<(), EncodeError> {
    if resolved.kind == "boolean" || resolved.kind == "document" {
        write_bool(protocol, value, body)
    } else {
        invalid_value(item)
    }
}

fn write_integer_value(
    protocol: Protocol, value: i64, item: ValueTask<'_>, resolved: Resolved<'_>, body: &mut BoundedBody,
) -> Result<(), EncodeError> {
    if !matches!(resolved.kind, "byte" | "short" | "integer" | "long" | "bigInteger" | "intEnum" | "document") {
        return invalid_value(item);
    }
    if resolved.kind == "bigInteger" {
        return write_big_integer(protocol, value, body);
    }
    validate_integer(resolved.kind, value, item.field)?;
    write_integer(protocol, value, body)
}

fn write_float_value(
    protocol: Protocol, value: f64, item: ValueTask<'_>, resolved: Resolved<'_>, body: &mut BoundedBody,
) -> Result<(), EncodeError> {
    if !matches!(resolved.kind, "float" | "double" | "document") {
        return invalid_value(item);
    }
    if resolved.kind == "float" && value.is_finite() && value.abs() > f64::from(f32::MAX) {
        return Err(EncodeError::InvalidValue { field: item.field, reason: "float is out of range" });
    }
    write_float(protocol, resolved.kind, value, body)
}

fn write_text_value(
    protocol: Protocol, value: &str, item: ValueTask<'_>, resolved: Resolved<'_>, body: &mut BoundedBody,
) -> Result<(), EncodeError> {
    if resolved.kind == "bigDecimal" {
        return write_big_decimal(protocol, value, item.field, body);
    }
    if matches!(resolved.kind, "string" | "enum" | "document") {
        json_or_cbor_text(protocol, value, body)
    } else {
        invalid_value(item)
    }
}

fn write_blob_value(
    protocol: Protocol, value: &[u8], item: ValueTask<'_>, resolved: Resolved<'_>, body: &mut BoundedBody,
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
    value: &[u8], item: ValueTask<'_>, media_type: Option<&str>, body: &mut BoundedBody,
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

fn write_timestamp_value(
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

fn write_list_value(
    protocol: Protocol,
    values: inventory_core::record::ListRef<'_>,
    item: ValueTask<'_>,
    resolved: Resolved<'_>,
    body: &mut BoundedBody,
    pending: &mut Vec<Task<'_>>,
) -> Result<(), EncodeError> {
    if !matches!(resolved.kind, "list" | "document") {
        return invalid_value(item);
    }
    check_container_depth(item)?;
    let binding = list_binding(resolved, item)?;
    push_list(body, protocol, collect_list(values, item.field)?, binding.0, binding.1, binding.2, binding.3, item, pending)
}

fn list_binding(
    resolved: Resolved<'_>,
    item: ValueTask<'_>,
) -> Result<(Target, bool, Option<&str>, Option<&str>), EncodeError> {
    if resolved.kind != "list" {
        return Ok((Target::Prelude("document"), false, None, None));
    }
    let shape = resolved.shape.ok_or(EncodeError::InvalidValue { field: item.field, reason: "list has no shape" })?;
    let wire = shape.element_wire()?;
    let element = shape.element()?.ok_or(EncodeError::InvalidValue {
        field: item.field,
        reason: "list has no element target",
    })?;
    Ok((element, wire.sparse(), wire.timestamp_format()?.map(timestamp_format_name), wire.media_type()?))
}

fn write_map_value(
    service: Service<'_>,
    protocol: Protocol,
    values: inventory_core::record::MapRef<'_>,
    item: ValueTask<'_>,
    resolved: Resolved<'_>,
    idempotency_token: Option<&str>,
    body: &mut BoundedBody,
    pending: &mut Vec<Task<'_>>,
) -> Result<(), EncodeError> {
    if matches!(resolved.kind, "structure" | "union") {
        check_container_depth(item)?;
        return write_structure_value(service, protocol, values, item, resolved, idempotency_token, body, pending);
    }
    if !matches!(resolved.kind, "map" | "document") {
        return invalid_value(item);
    }
    check_container_depth(item)?;
    let binding = map_binding(resolved, item)?;
    push_map(body, protocol, collect_map(values, item.field)?, binding.0, binding.1, binding.2, binding.3, binding.4, item, pending)
}

fn write_structure_value(
    service: Service<'_>, protocol: Protocol, values: inventory_core::record::MapRef<'_>,
    item: ValueTask<'_>, resolved: Resolved<'_>, idempotency_token: Option<&str>,
    body: &mut BoundedBody, pending: &mut Vec<Task<'_>>,
) -> Result<(), EncodeError> {
    let shape = resolved.shape.ok_or(EncodeError::InvalidValue {
        field: item.field,
        reason: "structured prelude value",
    })?;
    push_structure(body, service, protocol, shape, values, resolved.kind == "union", idempotency_token, item, pending)
}

fn map_binding(
    resolved: Resolved<'_>,
    item: ValueTask<'_>,
) -> Result<(Target, Target, bool, Option<&str>, Option<&str>), EncodeError> {
    if resolved.kind != "map" {
        return Ok((Target::Prelude("string"), Target::Prelude("document"), false, None, None));
    }
    let shape = resolved.shape.ok_or(EncodeError::InvalidValue { field: item.field, reason: "map has no shape" })?;
    let wire = shape.map_value_wire()?;
    let key = shape.map_key()?.ok_or(EncodeError::InvalidValue { field: item.field, reason: "map has no key target" })?;
    let value = shape.map_value()?.ok_or(EncodeError::InvalidValue { field: item.field, reason: "map has no value target" })?;
    Ok((key, value, wire.sparse(), wire.timestamp_format()?.map(timestamp_format_name), wire.media_type()?))
}

fn check_container_depth(item: ValueTask<'_>) -> Result<(), EncodeError> {
    if item.depth >= MAX_DEPTH { Err(EncodeError::Limit("serialization depth")) } else { Ok(()) }
}

fn invalid_value<T>(item: ValueTask<'_>) -> Result<T, EncodeError> {
    Err(EncodeError::InvalidValue { field: item.field, reason: "value does not match modeled shape" })
}
