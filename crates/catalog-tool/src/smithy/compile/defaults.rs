use super::CompileError;
use crate::smithy::model::{Shape, ShapeKind};
use inventory_core::{catalog::CatalogKey, record::Value};
use serde_json::Value as Json;
use std::collections::BTreeMap;

mod value;

use value::normalize;

pub(super) fn fields(
    traits: &BTreeMap<String, Json>,
    target: &str,
    shapes: &BTreeMap<String, Shape>,
) -> Result<Vec<(CatalogKey, Value)>, CompileError> {
    let mut fields = default_field(traits, target, shapes)?;
    if let Some(field) = optional_field(traits)? {
        fields.push(field);
    }
    Ok(fields)
}

fn default_field(
    traits: &BTreeMap<String, Json>,
    target: &str,
    shapes: &BTreeMap<String, Shape>,
) -> Result<Vec<(CatalogKey, Value)>, CompileError> {
    traits
        .get("smithy.api#default")
        .map(|value| normalize(value, shape_kind(target, shapes)))
        .transpose()
        .map(|value| {
            value
                .into_iter()
                .map(|value| (CatalogKey::DefaultValue, value))
                .collect()
        })
}

fn optional_field(
    traits: &BTreeMap<String, Json>,
) -> Result<Option<(CatalogKey, Value)>, CompileError> {
    let Some(value) = traits.get("smithy.api#clientOptional") else {
        return Ok(None);
    };
    if !matches!(value, Json::Object(fields) if fields.is_empty()) {
        return Err(CompileError::Trait(
            "clientOptional must be an empty marker trait".into(),
        ));
    }
    Ok(Some((CatalogKey::ClientOptional, Value::Bool(true))))
}

fn shape_kind<'a>(target: &str, shapes: &'a BTreeMap<String, Shape>) -> Option<&'a str> {
    shapes
        .get(target)
        .and_then(|shape| match &shape.kind {
            ShapeKind::Blob => Some("blob"),
            ShapeKind::Boolean => Some("boolean"),
            ShapeKind::String => Some("string"),
            ShapeKind::Byte => Some("byte"),
            ShapeKind::Short => Some("short"),
            ShapeKind::Integer => Some("integer"),
            ShapeKind::Long => Some("long"),
            ShapeKind::Float => Some("float"),
            ShapeKind::Double => Some("double"),
            ShapeKind::BigInteger => Some("bigInteger"),
            ShapeKind::BigDecimal => Some("bigDecimal"),
            ShapeKind::Timestamp => Some("timestamp"),
            ShapeKind::Enum { .. } => Some("string"),
            ShapeKind::IntEnum { .. } => Some("integer"),
            _ => None,
        })
        .or_else(|| prelude_scalar(target))
}

fn prelude_scalar(target: &str) -> Option<&'static str> {
    match target.strip_prefix("smithy.api#")? {
        "Blob" => Some("blob"),
        "Boolean" | "PrimitiveBoolean" => Some("boolean"),
        "String" => Some("string"),
        "Byte" | "PrimitiveByte" => Some("byte"),
        "Short" | "PrimitiveShort" => Some("short"),
        "Integer" | "PrimitiveInteger" => Some("integer"),
        "Long" | "PrimitiveLong" => Some("long"),
        "Float" | "PrimitiveFloat" => Some("float"),
        "Double" | "PrimitiveDouble" => Some("double"),
        "Timestamp" => Some("timestamp"),
        "BigInteger" => Some("bigInteger"),
        "BigDecimal" => Some("bigDecimal"),
        _ => None,
    }
}

#[cfg(test)]
#[path = "defaults/tests.rs"]
mod tests;
