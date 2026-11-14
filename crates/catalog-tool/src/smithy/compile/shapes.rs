//! Shape compilation: targets become local indexes and members become stable IDs.
use super::{
    CompileError,
    encode::{map, name, reference},
};
use crate::smithy::{
    Registry,
    model::{Shape, ShapeKind},
};
use inventory_core::{
    catalog::{CatalogKey, kind_code},
    record::Value,
};
use std::collections::BTreeMap;

pub(super) fn encode(
    id: &str,
    shape: &Shape,
    registry: &Registry,
    indexes: &BTreeMap<&str, usize>,
) -> Result<Value, CompileError> {
    let code = kind_code(kind(&shape.kind)?).ok_or(CompileError::Kind)?;
    let fields = [
        (CatalogKey::Name, name(id)),
        (
            CatalogKey::Kind,
            Value::Int(i64::try_from(code).map_err(|_| CompileError::Kind)?),
        ),
        (
            CatalogKey::Members,
            Value::List(members(shape, registry, indexes)?),
        ),
        (CatalogKey::Arn, arn(id, shape)?),
    ];
    map(fields.into_iter().chain(targets(&shape.kind, indexes)?))
}

fn arn(id: &str, shape: &Shape) -> Result<Value, CompileError> {
    match shape.traits.get("aws.api#arn") {
        None => Ok(Value::Null),
        Some(value) => Ok(Value::Text(
            value
                .get("template")
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| CompileError::Arn(id.into()))?
                .to_owned(),
        )),
    }
}
fn members(
    shape: &Shape,
    registry: &Registry,
    indexes: &BTreeMap<&str, usize>,
) -> Result<Vec<Value>, CompileError> {
    let source = match &shape.kind {
        ShapeKind::Structure { members } | ShapeKind::Union { members } => members,
        _ => return Ok(Vec::new()),
    };
    let ordered = source
        .iter()
        .map(|(name, member)| {
            let id = registry
                .id(name)
                .ok_or_else(|| CompileError::MissingMember(name.clone()))?;
            Ok((id, member_value(name, member, id, indexes)?))
        })
        .collect::<Result<BTreeMap<_, _>, CompileError>>()?;
    Ok(ordered.into_values().collect())
}

fn member_value(
    name: &str,
    member: &crate::smithy::model::Member,
    id: inventory_core::record::FieldId,
    indexes: &BTreeMap<&str, usize>,
) -> Result<Value, CompileError> {
    let target = member
        .target
        .as_deref()
        .ok_or_else(|| CompileError::Target(name.into()))?;
    Ok(Value::List(vec![
        Value::Int(i64::from(id.value())),
        reference(target, indexes)?,
        Value::Bool(member.traits.contains_key("smithy.api#required")),
    ]))
}
fn targets(
    kind: &ShapeKind,
    indexes: &BTreeMap<&str, usize>,
) -> Result<Vec<(CatalogKey, Value)>, CompileError> {
    match kind {
        ShapeKind::List { member } => Ok(vec![(
            CatalogKey::Element,
            reference(&member.target, indexes)?,
        )]),
        ShapeKind::Map { key, value } => Ok(vec![
            (CatalogKey::MapKey, reference(&key.target, indexes)?),
            (CatalogKey::MapValue, reference(&value.target, indexes)?),
        ]),
        _ => Ok(Vec::new()),
    }
}
fn kind(kind: &ShapeKind) -> Result<&'static str, CompileError> {
    match kind {
        ShapeKind::Blob => Ok("blob"),
        ShapeKind::Boolean => Ok("boolean"),
        ShapeKind::String => Ok("string"),
        ShapeKind::Byte => Ok("byte"),
        ShapeKind::Short => Ok("short"),
        ShapeKind::Integer => Ok("integer"),
        ShapeKind::Long => Ok("long"),
        ShapeKind::Float => Ok("float"),
        ShapeKind::Double => Ok("double"),
        ShapeKind::BigInteger => Ok("bigInteger"),
        ShapeKind::BigDecimal => Ok("bigDecimal"),
        ShapeKind::Timestamp => Ok("timestamp"),
        ShapeKind::Document => Ok("document"),
        _ => aggregate_kind(kind),
    }
}
fn aggregate_kind(kind: &ShapeKind) -> Result<&'static str, CompileError> {
    match kind {
        ShapeKind::Structure { .. } => Ok("structure"),
        ShapeKind::Union { .. } => Ok("union"),
        ShapeKind::List { .. } => Ok("list"),
        ShapeKind::Map { .. } => Ok("map"),
        ShapeKind::Enum => Ok("enum"),
        ShapeKind::IntEnum => Ok("intEnum"),
        ShapeKind::Service => Ok("service"),
        ShapeKind::Operation { .. } => Ok("operation"),
        ShapeKind::Resource => Ok("resource"),
        _ => Err(CompileError::Kind),
    }
}
