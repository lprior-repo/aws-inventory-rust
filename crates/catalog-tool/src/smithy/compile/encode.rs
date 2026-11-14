//! Record writer integration; deterministic BTreeMap keys, shape IDs and registry IDs.
use super::{CompileError, Registry, shapes};
use crate::smithy::model::{ModelFile, Shape, ShapeKind, Target};
use inventory_core::{
    catalog::{CatalogKey, SHAPE_KINDS},
    record::{Key, Record, Value},
};
use std::collections::BTreeMap;

pub(super) fn map(
    entries: impl IntoIterator<Item = (CatalogKey, Value)>,
) -> Result<Value, CompileError> {
    entries
        .into_iter()
        .map(|(key, value)| Ok((Key::Field(key.field()?), value)))
        .collect::<Result<_, CompileError>>()
        .map(Value::Map)
}
pub(super) fn service(
    model: &ModelFile,
    registry: &Registry,
    service_name: &str,
) -> Result<Record, CompileError> {
    let indexes: BTreeMap<_, _> = model
        .shapes
        .keys()
        .enumerate()
        .map(|(index, id)| (id.as_str(), index))
        .collect();
    let shapes = shape_values(model, registry, &indexes)?;
    let operations = operation_values(model, &indexes)?;
    let mut fields = service_fields(model, service_name)?;
    fields.extend([
        (CatalogKey::Operations, Value::List(operations)),
        (CatalogKey::Shapes, Value::List(shapes)),
    ]);
    Ok(Record::encode(&map(fields)?)?)
}

fn shape_values(
    model: &ModelFile,
    registry: &Registry,
    indexes: &BTreeMap<&str, usize>,
) -> Result<Vec<Value>, CompileError> {
    model
        .shapes
        .iter()
        .map(|(id, shape)| {
            shapes::encode(id, shape, registry, indexes).map_err(|error| error.in_shape(id))
        })
        .collect()
}

fn operation_values(
    model: &ModelFile,
    indexes: &BTreeMap<&str, usize>,
) -> Result<Vec<Value>, CompileError> {
    model
        .shapes
        .iter()
        .filter(|(_, shape)| matches!(shape.kind, ShapeKind::Operation { .. }))
        .map(|(id, shape)| operation(id, shape, model, indexes).map_err(|error| error.in_shape(id)))
        .collect()
}

fn service_fields(
    model: &ModelFile,
    service_name: &str,
) -> Result<Vec<(CatalogKey, Value)>, CompileError> {
    let (id, service) = service_shape(model)?;
    let protocols = service
        .traits
        .keys()
        .filter(|id| id.contains("protocols#"))
        .map(|id| Value::Text(id.to_owned()))
        .collect();
    let resources = model
        .shapes
        .iter()
        .filter(|(_, shape)| matches!(shape.kind, ShapeKind::Resource))
        .map(|(id, _)| name(id))
        .collect();
    Ok(vec![
        (CatalogKey::Name, Value::Text(service_name.to_owned())),
        (CatalogKey::ServiceShape, Value::Text(id.to_owned())),
        (CatalogKey::Protocols, Value::List(protocols)),
        (CatalogKey::Resources, Value::List(resources)),
    ])
}

pub(super) fn service_shape(model: &ModelFile) -> Result<(&str, &Shape), CompileError> {
    model
        .shapes
        .iter()
        .find(|(_, shape)| matches!(shape.kind, ShapeKind::Service))
        .map(|(id, shape)| (id.as_str(), shape))
        .ok_or(CompileError::Kind)
}
pub(super) fn name(id: &str) -> Value {
    Value::Text(short_name(id).to_owned())
}
pub(super) fn short_name(id: &str) -> &str {
    match id.rsplit_once('#') {
        Some((_, name)) => name,
        None => id,
    }
}
pub(super) fn reference(id: &str, indexes: &BTreeMap<&str, usize>) -> Result<Value, CompileError> {
    if let Some(index) = indexes.get(id) {
        return i64::try_from(*index)
            .map(Value::Int)
            .map_err(|_| CompileError::Target(id.into()));
    }
    let prelude = match id.strip_prefix("smithy.api#") {
        Some("PrimitiveBoolean") => "boolean",
        Some("PrimitiveByte") => "byte",
        Some("PrimitiveShort") => "short",
        Some("PrimitiveInteger") => "integer",
        Some("PrimitiveLong") => "long",
        Some("PrimitiveFloat") => "float",
        Some("PrimitiveDouble") => "double",
        Some("Unit") => "unit",
        Some(name) => return prelude_reference(name, id),
        None => return Err(CompileError::Target(id.into())),
    };
    prelude_reference(prelude, id)
}
fn prelude_reference(name: &str, id: &str) -> Result<Value, CompileError> {
    let code = SHAPE_KINDS
        .iter()
        .position(|kind| kind.eq_ignore_ascii_case(name))
        .filter(|n| *n <= 13)
        .ok_or_else(|| CompileError::Target(id.into()))?;
    let raw = i64::try_from(code)
        .ok()
        .and_then(|n| n.checked_add(1))
        .and_then(i64::checked_neg)
        .ok_or_else(|| CompileError::Target(id.into()))?;
    Ok(Value::Int(raw))
}
fn operation(
    id: &str,
    shape: &Shape,
    model: &ModelFile,
    indexes: &BTreeMap<&str, usize>,
) -> Result<Value, CompileError> {
    let ShapeKind::Operation { input, output } = &shape.kind else {
        return Err(CompileError::Kind);
    };
    map([
        (CatalogKey::Name, name(id)),
        (
            CatalogKey::Flags,
            Value::Int(flags(short_name(id), shape, input, model)),
        ),
        (CatalogKey::Input, optional_target(input, indexes)?),
        (CatalogKey::Output, optional_target(output, indexes)?),
    ])
}

fn flags(name: &str, shape: &Shape, input: &Option<Target>, model: &ModelFile) -> i64 {
    let read = [
        "List", "Describe", "Get", "BatchGet", "Search", "Scan", "Query", "Lookup", "Select",
    ]
    .iter()
    .any(|prefix| name.starts_with(prefix));
    [
        (shape.traits.contains_key("smithy.api#readonly"), 1),
        (shape.traits.contains_key("smithy.api#paginated"), 2),
        (name.starts_with("List") || name.starts_with("Describe"), 4),
        (input_free(input, model), 8),
        (read, 16),
    ]
    .into_iter()
    .fold(
        0,
        |flags, (present, mask)| {
            if present { flags | mask } else { flags }
        },
    )
}
fn optional_target(
    target: &Option<Target>,
    indexes: &BTreeMap<&str, usize>,
) -> Result<Value, CompileError> {
    reference(
        target
            .as_ref()
            .map_or("smithy.api#Unit", |t| t.target.as_str()),
        indexes,
    )
}
fn input_free(input: &Option<Target>, model: &ModelFile) -> bool {
    match input
        .as_ref()
        .and_then(|t| model.shapes.get(&t.target))
        .map(|s| &s.kind)
    {
        Some(ShapeKind::Structure { members }) => members
            .values()
            .all(|member| !member.traits.contains_key("smithy.api#required")),
        _ => true,
    }
}
