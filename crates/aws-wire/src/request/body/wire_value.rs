use super::{
    wire::{Encoder, MAX_DEPTH, ValueTask, collect_list, collect_map, timestamp_format_name},
    wire_member::push_structure,
    wire_shape::{ListBinding, MapBinding, push_list, push_map},
    wire_value_scalar::{
        write_blob_value, write_bool_value, write_float_value, write_integer_value,
        write_null_value, write_text_value, write_timestamp_value,
    },
};
use crate::request::EncodeError;
use inventory_core::{
    catalog::{Service, Shape, Target},
    record::ValueRef,
};
#[derive(Clone, Copy)]
pub(super) struct Resolved<'a> {
    pub(super) kind: &'a str,
    pub(super) shape: Option<Shape<'a>>,
    pub(super) timestamp_format: Option<&'a str>,
    pub(super) media_type: Option<&'a str>,
}

pub(super) fn write_value<'a, 'b>(
    encoder: &mut Encoder<'a, 'b>,
    item: ValueTask<'a>,
) -> Result<(), EncodeError> {
    if item.depth > MAX_DEPTH {
        return Err(EncodeError::Limit("serialization depth"));
    }
    let resolved = resolve(encoder.service, item)?;
    match item.value {
        ValueRef::Null => write_null_value(encoder.protocol, item, resolved, encoder.body),
        ValueRef::Bool(value) => {
            write_bool_value(encoder.protocol, value, item, resolved, encoder.body)
        }
        ValueRef::Int(value) => {
            write_integer_value(encoder.protocol, value, item, resolved, encoder.body)
        }
        ValueRef::Float(value) => write_float_value(
            encoder.protocol,
            value.to_f64(),
            item,
            resolved,
            encoder.body,
        ),
        ValueRef::Text(value) => {
            write_text_value(encoder.protocol, value, item, resolved, encoder.body)
        }
        ValueRef::Bytes(value) => {
            write_blob_value(encoder.protocol, value, item, resolved, encoder.body)
        }
        ValueRef::Time(value) => {
            write_timestamp_value(encoder.protocol, value, item, resolved, encoder.body)
        }
        ValueRef::List(values) => write_list_value(encoder, values, item, resolved),
        ValueRef::Map(values) => write_map_value(encoder, values, item, resolved),
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
        _ => return Err(EncodeError::Unsupported("catalog target kind")),
    };
    let wire = shape.map(|shape| shape.wire()).transpose()?;
    if wire
        .map(|wire| wire.streaming())
        .transpose()?
        .unwrap_or(false)
    {
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
    let media_type = item
        .media_type
        .or(wire.map(|wire| wire.media_type()).transpose()?.flatten());
    Ok(Resolved {
        kind,
        shape,
        timestamp_format,
        media_type,
    })
}

fn write_list_value<'a>(
    encoder: &mut Encoder<'a, '_>,
    values: inventory_core::record::ListRef<'a>,
    item: ValueTask<'a>,
    resolved: Resolved<'a>,
) -> Result<(), EncodeError> {
    if !matches!(resolved.kind, "list" | "document") {
        return invalid_value(item);
    }
    check_container_depth(item)?;
    let binding = list_binding(resolved, item)?;
    push_list(encoder, collect_list(values, item.field)?, binding, item)
}

fn list_binding<'a>(
    resolved: Resolved<'a>,
    item: ValueTask<'a>,
) -> Result<ListBinding<'a>, EncodeError> {
    if resolved.kind != "list" {
        return Ok(ListBinding {
            element: Target::Prelude("document"),
            sparse: resolved.kind == "document",
            timestamp_format: None,
            media_type: None,
        });
    }
    let shape = resolved.shape.ok_or(EncodeError::InvalidValue {
        field: item.field,
        reason: "list has no shape",
    })?;
    modeled_list_binding(shape, item)
}

fn modeled_list_binding<'a>(
    shape: Shape<'a>,
    item: ValueTask<'a>,
) -> Result<ListBinding<'a>, EncodeError> {
    let wire = shape.element_wire()?;
    let element = shape.element()?.ok_or(EncodeError::InvalidValue {
        field: item.field,
        reason: "list has no element target",
    })?;
    Ok(ListBinding {
        element,
        sparse: shape.wire()?.sparse()?,
        timestamp_format: wire.timestamp_format()?.map(timestamp_format_name),
        media_type: wire.media_type()?,
    })
}

fn write_map_value<'a>(
    encoder: &mut Encoder<'a, '_>,
    values: inventory_core::record::MapRef<'a>,
    item: ValueTask<'a>,
    resolved: Resolved<'a>,
) -> Result<(), EncodeError> {
    if matches!(resolved.kind, "structure" | "union") {
        check_container_depth(item)?;
        return write_structure_value(encoder, values, item, resolved);
    }
    if !matches!(resolved.kind, "map" | "document") {
        return invalid_value(item);
    }
    check_container_depth(item)?;
    let binding = map_binding(resolved, item)?;
    push_map(encoder, collect_map(values, item.field)?, binding, item)
}

fn write_structure_value<'a>(
    encoder: &mut Encoder<'a, '_>,
    values: inventory_core::record::MapRef<'a>,
    item: ValueTask<'a>,
    resolved: Resolved<'a>,
) -> Result<(), EncodeError> {
    let shape = resolved.shape.ok_or(EncodeError::InvalidValue {
        field: item.field,
        reason: "structured prelude value",
    })?;
    push_structure(encoder, shape, values, resolved.kind == "union", item)
}

fn map_binding<'a>(
    resolved: Resolved<'a>,
    item: ValueTask<'a>,
) -> Result<MapBinding<'a>, EncodeError> {
    if resolved.kind != "map" {
        return Ok(MapBinding {
            key_target: Target::Prelude("string"),
            value_target: Target::Prelude("document"),
            sparse: resolved.kind == "document",
            timestamp_format: None,
            media_type: None,
        });
    }
    let shape = resolved.shape.ok_or(EncodeError::InvalidValue {
        field: item.field,
        reason: "map has no shape",
    })?;
    modeled_map_binding(shape, item)
}

fn modeled_map_binding<'a>(
    shape: Shape<'a>,
    item: ValueTask<'a>,
) -> Result<MapBinding<'a>, EncodeError> {
    let wire = shape.map_value_wire()?;
    let key_target = shape.map_key()?.ok_or(EncodeError::InvalidValue {
        field: item.field,
        reason: "map has no key target",
    })?;
    let value_target = shape.map_value()?.ok_or(EncodeError::InvalidValue {
        field: item.field,
        reason: "map has no value target",
    })?;
    Ok(MapBinding {
        key_target,
        value_target,
        sparse: shape.wire()?.sparse()?,
        timestamp_format: wire.timestamp_format()?.map(timestamp_format_name),
        media_type: wire.media_type()?,
    })
}

fn check_container_depth(item: ValueTask<'_>) -> Result<(), EncodeError> {
    if item.depth >= MAX_DEPTH {
        Err(EncodeError::Limit("serialization depth"))
    } else {
        Ok(())
    }
}

pub(super) fn invalid_value<T>(item: ValueTask<'_>) -> Result<T, EncodeError> {
    Err(EncodeError::InvalidValue {
        field: item.field,
        reason: "value does not match modeled shape",
    })
}
