use super::*;

pub(super) fn scalar(
    service: Service<'_>,
    target: Target,
    kind: &str,
    value: &FixedValue,
    bytes: &mut usize,
) -> Result<Value, RuntimeCatalogCompileError> {
    match value {
        FixedValue::String(text) if matches!(kind, "string" | "enum") => {
            text_value(service, target, text, bytes)
        }
        FixedValue::Integer(number) if integer(kind, *number) => {
            integer_value(service, target, *number)
        }
        FixedValue::Boolean(value) if kind == "boolean" => Ok(Value::Bool(*value)),
        FixedValue::Float(number) if floating(kind, number.to_f64()) => Ok(Value::Float(*number)),
        FixedValue::Bytes(value) if kind == "blob" => {
            text_bound(value.len(), bytes)?;
            Ok(Value::Bytes(value.clone()))
        }
        FixedValue::Timestamp(value) if kind == "timestamp" => Ok(Value::Time(*value)),
        _ => Err(unsupported()),
    }
}

fn floating(kind: &str, value: f64) -> bool {
    matches!(kind, "float" | "double")
        && value.is_finite()
        && (kind == "double" || value.abs() <= f64::from(f32::MAX))
}

fn text_value(
    service: Service<'_>,
    target: Target,
    text: &str,
    bytes: &mut usize,
) -> Result<Value, RuntimeCatalogCompileError> {
    text_bound(text.len(), bytes)?;
    enum_value(service, target, ValueRef::Text(text))?;
    Ok(Value::Text(text.to_owned()))
}

fn integer_value(
    service: Service<'_>,
    target: Target,
    number: i64,
) -> Result<Value, RuntimeCatalogCompileError> {
    enum_value(service, target, ValueRef::Int(number))?;
    Ok(Value::Int(number))
}

pub(super) fn shape(
    service: Service<'_>,
    target: Target,
) -> Result<Shape<'_>, RuntimeCatalogCompileError> {
    match target {
        Target::Shape(index) => service.shape(index)?.ok_or_else(unsupported),
        _ => Err(unsupported()),
    }
}

pub(super) fn text_bound(
    length: usize,
    bytes: &mut usize,
) -> Result<(), RuntimeCatalogCompileError> {
    *bytes = bytes.checked_add(length).ok_or_else(unsupported)?;
    if length > BindingValue::MAX_STRING || *bytes > BindingValue::MAX_TEXT_BYTES {
        Err(unsupported())
    } else {
        Ok(())
    }
}

fn integer(kind: &str, value: i64) -> bool {
    match kind {
        "byte" => i8::try_from(value).is_ok(),
        "short" => i16::try_from(value).is_ok(),
        "integer" | "intEnum" => i32::try_from(value).is_ok(),
        "long" => true,
        _ => false,
    }
}

pub(super) fn enum_value(
    service: Service<'_>,
    target: Target,
    value: ValueRef<'_>,
) -> Result<(), RuntimeCatalogCompileError> {
    if let Target::Shape(_) = target
        && !shape(service, target)?.accepts_enum_value(value)?
    {
        return Err(unsupported());
    }
    Ok(())
}
