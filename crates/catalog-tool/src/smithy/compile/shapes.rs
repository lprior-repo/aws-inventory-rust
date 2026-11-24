use super::{
    CompileError,
    encode::{map, name, reference},
    metadata,
};
use crate::smithy::{
    Registry,
    model::{Member, Shape, ShapeKind},
};
use inventory_core::{
    catalog::{CatalogKey, kind_code},
    record::Value,
};
use std::collections::BTreeMap;
mod enum_values;
mod references;

pub(super) fn encode(
    id: &str,
    shape: &Shape,
    registry: &Registry,
    indexes: &BTreeMap<&str, usize>,
    all_shapes: &BTreeMap<String, Shape>,
) -> Result<Value, CompileError> {
    let mut fields = base_fields(id, shape)?;
    let members = members(shape, registry, indexes, all_shapes)?;
    if !members.is_empty() {
        fields.push((CatalogKey::Members, Value::List(members)));
    }
    fields.extend(targets(&shape.kind, indexes)?);
    fields.extend(metadata::shape_fields(&shape.traits)?);
    fields.extend(enum_values::field(shape)?);
    fields.extend(references::fields(shape)?);
    map(fields)
}

fn base_fields(id: &str, shape: &Shape) -> Result<Vec<(CatalogKey, Value)>, CompileError> {
    let wire_kind = match &shape.kind {
        ShapeKind::String if shape.traits.contains_key("smithy.api#enum") => "enum",
        otherwise => kind(otherwise)?,
    };
    let code = kind_code(wire_kind).ok_or(CompileError::Kind)?;
    let mut fields = vec![
        (CatalogKey::Name, name(id)),
        (
            CatalogKey::Kind,
            Value::Int(i64::try_from(code).map_err(|_| CompileError::Kind)?),
        ),
    ];
    if let Some(template) = arn(id, shape)? {
        fields.push((CatalogKey::Arn, Value::Text(template)));
    }
    Ok(fields)
}

fn arn(id: &str, shape: &Shape) -> Result<Option<String>, CompileError> {
    let Some(value) = find_trait(&shape.traits, "aws.api#arn") else {
        return Ok(None);
    };
    let template = value
        .get("template")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| CompileError::Arn(id.to_owned()))?;
    Ok(Some(template.to_owned()))
}

fn members(
    shape: &Shape,
    registry: &Registry,
    indexes: &BTreeMap<&str, usize>,
    all_shapes: &BTreeMap<String, Shape>,
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
            Ok((id, member_value(name, member, id, indexes, all_shapes)?))
        })
        .collect::<Result<BTreeMap<_, _>, CompileError>>()?;
    Ok(ordered.into_values().collect())
}

fn member_value(
    name: &str,
    member: &Member,
    id: inventory_core::record::FieldId,
    indexes: &BTreeMap<&str, usize>,
    all_shapes: &BTreeMap<String, Shape>,
) -> Result<Value, CompileError> {
    let target = member
        .target
        .as_deref()
        .ok_or_else(|| CompileError::Target(name.into()))?;
    let mut fields = vec![
        (CatalogKey::MemberId, Value::Int(i64::from(id.value()))),
        (CatalogKey::MemberTarget, reference(target, indexes)?),
        (
            CatalogKey::MemberRequired,
            Value::Bool(has_trait(&member.traits, "smithy.api#required")),
        ),
    ];
    fields.extend(metadata::member_fields(member, target, all_shapes)?);
    fields.extend(references::traits(&member.traits)?);
    map(fields)
}

fn targets(
    kind: &ShapeKind,
    indexes: &BTreeMap<&str, usize>,
) -> Result<Vec<(CatalogKey, Value)>, CompileError> {
    match kind {
        ShapeKind::List { member } => {
            let mut fields = vec![(CatalogKey::Element, reference(&member.target, indexes)?)];
            fields.push(metadata::pseudo_field(
                &member.traits,
                CatalogKey::ElementWire,
            )?);
            Ok(fields)
        }
        ShapeKind::Map { key, value } => map_targets(key, value, indexes),
        _ => Ok(Vec::new()),
    }
}

fn map_targets(
    key: &crate::smithy::model::Target,
    value: &crate::smithy::model::Target,
    indexes: &BTreeMap<&str, usize>,
) -> Result<Vec<(CatalogKey, Value)>, CompileError> {
    let mut fields = vec![
        (CatalogKey::MapKey, reference(&key.target, indexes)?),
        (CatalogKey::MapValue, reference(&value.target, indexes)?),
    ];
    fields.push(metadata::pseudo_field(&key.traits, CatalogKey::MapKeyWire)?);
    fields.push(metadata::pseudo_field(
        &value.traits,
        CatalogKey::MapValueWire,
    )?);
    Ok(fields)
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
        ShapeKind::Enum { .. } => Ok("enum"),
        ShapeKind::IntEnum { .. } => Ok("intEnum"),
        ShapeKind::Service => Ok("service"),
        ShapeKind::Operation { .. } => Ok("operation"),
        ShapeKind::Resource { .. } => Ok("resource"),
        _ => Err(CompileError::Kind),
    }
}

fn find_trait<'a>(
    traits: &'a BTreeMap<String, serde_json::Value>,
    id: &str,
) -> Option<&'a serde_json::Value> {
    traits.get(id)
}
fn has_trait(traits: &BTreeMap<String, serde_json::Value>, id: &str) -> bool {
    find_trait(traits, id).is_some()
}
