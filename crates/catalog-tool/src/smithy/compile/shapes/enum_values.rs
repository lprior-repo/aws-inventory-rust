use crate::smithy::{
    compile::CompileError,
    model::{Shape, ShapeKind},
};
use inventory_core::{
    catalog::{CatalogKey, runtime::BindingValue},
    record::Value,
};

const MAX_VALUES: usize = 65_536;

pub(super) fn field(shape: &Shape) -> Result<Option<(CatalogKey, Value)>, CompileError> {
    let mut bytes = 0usize;
    let values = match &shape.kind {
        ShapeKind::Enum { members } | ShapeKind::IntEnum { members } => {
            if members.len() > MAX_VALUES {
                return Err(invalid());
            }
            let integer = matches!(&shape.kind, ShapeKind::IntEnum { .. });
            members
                .values()
                .map(|member| {
                    modern(
                        member
                            .traits
                            .get("smithy.api#enumValue")
                            .ok_or_else(invalid)?,
                        integer,
                        &mut bytes,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?
        }
        ShapeKind::String => {
            let Some(values) = shape.traits.get("smithy.api#enum") else {
                return Ok(Some((CatalogKey::EnumValues, Value::Null)));
            };
            legacy(values, &mut bytes)?
        }
        _ => return Ok(None),
    };
    Ok(Some((CatalogKey::EnumValues, Value::List(values))))
}
fn modern(
    value: &serde_json::Value,
    integer: bool,
    bytes: &mut usize,
) -> Result<Value, CompileError> {
    match value {
        serde_json::Value::String(value) if !integer => text(value, bytes),
        serde_json::Value::Number(value) if integer => Ok(Value::Int(
            value
                .as_i64()
                .filter(|value| i32::try_from(*value).is_ok())
                .ok_or_else(invalid)?,
        )),
        _ => Err(invalid()),
    }
}
fn legacy(values: &serde_json::Value, bytes: &mut usize) -> Result<Vec<Value>, CompileError> {
    let values = values
        .as_array()
        .filter(|values| values.len() <= MAX_VALUES)
        .ok_or_else(invalid)?;
    values
        .iter()
        .map(|value| {
            text(
                value
                    .get("value")
                    .and_then(serde_json::Value::as_str)
                    .ok_or_else(invalid)?,
                bytes,
            )
        })
        .collect()
}
fn text(value: &str, bytes: &mut usize) -> Result<Value, CompileError> {
    *bytes = bytes.checked_add(value.len()).ok_or_else(invalid)?;
    if value.len() > BindingValue::MAX_STRING || *bytes > BindingValue::MAX_TEXT_BYTES {
        return Err(invalid());
    }
    Ok(Value::Text(value.to_owned()))
}
fn invalid() -> CompileError {
    CompileError::Trait("smithy.api#enumValue or smithy.api#enum".into())
}
