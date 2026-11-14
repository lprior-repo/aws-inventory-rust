use super::{CompileError, Registry, operation, service, shapes};
use crate::smithy::model::{ModelFile, Shape, ShapeKind, Target};
use inventory_core::{catalog::{CatalogKey, SHAPE_KINDS}, record::{Key, Record, Value}};
use std::collections::BTreeMap;

pub(super) fn map(entries: impl IntoIterator<Item = (CatalogKey, Value)>) -> Result<Value, CompileError> {
    entries.into_iter().map(|(key, value)| Ok((Key::Field(key.field()?), value)))
        .collect::<Result<_, CompileError>>().map(Value::Map)
}

pub(super) fn service(model: &ModelFile, registry: &Registry, service_name: &str) -> Result<Record, CompileError> {
    let indexes = model.shapes.keys().enumerate().map(|(index, id)| (id.as_str(), index)).collect::<BTreeMap<_, _>>();
    let shapes = model.shapes.iter().map(|(id, shape)| encode_shape(id, shape, registry, &indexes).map_err(|error| error.in_shape(id))).collect::<Result<Vec<_>, _>>()?;
    let operations = model.shapes.iter().filter(|(_, shape)| matches!(&shape.kind, ShapeKind::Operation { .. }))
        .map(|(id, shape)| operation::encode(id, shape, model, &indexes).map_err(|error| error.in_shape(id)))
        .collect::<Result<Vec<_>, _>>()?;
    let mut fields = service::fields(model, service_name, &indexes)?;
    fields.extend([(CatalogKey::Operations, Value::List(operations)), (CatalogKey::Shapes, Value::List(shapes))]);
    Ok(Record::encode(&map(fields)?)?)
}
fn encode_shape(id: &str, shape: &Shape, registry: &Registry, indexes: &BTreeMap<&str, usize>) -> Result<Value, CompileError> {
    crate::smithy::model::validate_shape_traits(shape).map_err(CompileError::Trait)?;
    shapes::encode(id, shape, registry, indexes)
}

pub(super) fn service_shape(model: &ModelFile) -> Result<(&str, &Shape), CompileError> { service::service_shape(model) }

pub(super) fn name(id: &str) -> Value { Value::Text(short_name(id).to_owned()) }

pub(super) fn short_name(id: &str) -> &str { id.rsplit_once('#').map_or(id, |(_, name)| name) }

pub(super) fn optional_target(target: &Option<Target>, indexes: &BTreeMap<&str, usize>) -> Result<Value, CompileError> {
    reference(target.as_ref().map_or("smithy.api#Unit", |target| target.target.as_str()), indexes)
}

pub(super) fn reference(id: &str, indexes: &BTreeMap<&str, usize>) -> Result<Value, CompileError> {
    if let Some(index) = indexes.get(id) { return i64::try_from(*index).map(Value::Int).map_err(|_| CompileError::Target(id.into())); }
    let prelude = match id.strip_prefix("smithy.api#") {
        Some("PrimitiveBoolean") => "boolean", Some("PrimitiveByte") => "byte", Some("PrimitiveShort") => "short",
        Some("PrimitiveInteger") => "integer", Some("PrimitiveLong") => "long", Some("PrimitiveFloat") => "float",
        Some("PrimitiveDouble") => "double", Some("Unit") => "unit", Some(name) => return prelude_reference(name, id),
        None => return Err(CompileError::Target(id.into())),
    };
    prelude_reference(prelude, id)
}

fn prelude_reference(name: &str, id: &str) -> Result<Value, CompileError> {
    let code = SHAPE_KINDS.iter().position(|kind| kind.eq_ignore_ascii_case(name)).filter(|code| *code <= 13).ok_or_else(|| CompileError::Target(id.into()))?;
    let raw = i64::try_from(code).ok().and_then(|code| code.checked_add(1)).and_then(i64::checked_neg).ok_or_else(|| CompileError::Target(id.into()))?;
    Ok(Value::Int(raw))
}
