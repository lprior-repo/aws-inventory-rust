use super::{MemberPath, OperationName, OverlayError, Rule, error::Context};
use std::collections::BTreeMap;
use toml::Value;

pub(super) fn spelling(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Array(_) => "array".to_owned(),
        Value::Table(_) => "table".to_owned(),
        Value::Integer(number) => number.to_string(),
        Value::Float(number) => number.to_string(),
        Value::Boolean(boolean) => boolean.to_string(),
        Value::Datetime(_) => "datetime".to_owned(),
    }
}
pub(super) fn required<'a>(
    context: &Context<'_>,
    key: &str,
    value: Option<&'a Value>,
) -> Result<&'a Value, OverlayError> {
    value.ok_or_else(|| context.error(key, "", Rule::Missing))
}
pub(super) fn string<'a>(
    context: &Context<'_>,
    key: &str,
    value: &'a Value,
) -> Result<&'a str, OverlayError> {
    value
        .as_str()
        .ok_or_else(|| context.error(key, spelling(value), Rule::WrongType { expected: "string" }))
}
pub(super) fn text(
    context: &Context<'_>,
    key: &str,
    value: Option<&Value>,
) -> Result<Option<String>, OverlayError> {
    value
        .map(|value| string(context, key, value).map(str::to_owned))
        .transpose()
}
pub(super) fn path(
    context: &Context<'_>,
    key: &str,
    value: &Value,
) -> Result<MemberPath, OverlayError> {
    let text = string(context, key, value)?;
    MemberPath::new(text).map_err(|_| context.error(key, text, Rule::MemberPath))
}
pub(super) fn optional_path(
    context: &Context<'_>,
    key: &str,
    value: Option<&Value>,
) -> Result<Option<MemberPath>, OverlayError> {
    value.map(|value| path(context, key, value)).transpose()
}
pub(super) fn paths(
    context: &Context<'_>,
    key: &str,
    value: Option<&Value>,
) -> Result<Vec<MemberPath>, OverlayError> {
    list(context, key, value, 0, 64, path)
}
pub(super) fn operation(
    context: &Context<'_>,
    key: &str,
    value: &Value,
) -> Result<OperationName, OverlayError> {
    let text = string(context, key, value)?;
    OperationName::new(text).map_err(|_| context.error(key, text, Rule::OperationName))
}
pub(super) fn number(
    context: &Context<'_>,
    key: &str,
    value: &Value,
    min: i64,
    max: i64,
) -> Result<i64, OverlayError> {
    let number = value.as_integer().ok_or_else(|| {
        context.error(
            key,
            spelling(value),
            Rule::WrongType {
                expected: "integer",
            },
        )
    })?;
    if !(min..=max).contains(&number) {
        return Err(context.error(key, number, Rule::Range { min, max }));
    }
    Ok(number)
}
pub(super) fn array<'a>(
    context: &Context<'_>,
    key: &str,
    value: &'a Value,
    min: usize,
    max: usize,
) -> Result<&'a [Value], OverlayError> {
    let values = array_values(context, key, value)?;
    if !(min..=max).contains(&values.len()) {
        return Err(context.error(key, values.len(), Rule::Count { min, max }));
    }
    if let Some((_, value)) = values.iter().enumerate().find(|(index, value)| {
        values
            .iter()
            .take(*index)
            .any(|previous| previous == *value)
    }) {
        return Err(context.error(key, spelling(value), Rule::Duplicate));
    }
    Ok(values)
}
fn array_values<'a>(
    context: &Context<'_>,
    key: &str,
    value: &'a Value,
) -> Result<&'a [Value], OverlayError> {
    value
        .as_array()
        .map(Vec::as_slice)
        .ok_or_else(|| context.error(key, spelling(value), Rule::WrongType { expected: "array" }))
}
pub(super) fn list<T>(
    context: &Context<'_>,
    key: &str,
    value: Option<&Value>,
    min: usize,
    max: usize,
    parse: impl Fn(&Context<'_>, &str, &Value) -> Result<T, OverlayError>,
) -> Result<Vec<T>, OverlayError> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };
    array(context, key, value, min, max)?
        .iter()
        .try_fold(Vec::new(), |mut output, value| {
            output
                .try_reserve(1)
                .map_err(|_| context.error(key, "", Rule::Resource))?;
            output.push(parse(context, key, value)?);
            Ok(output)
        })
}
pub(super) fn table(
    context: &Context<'_>,
    key: &str,
    value: Option<Value>,
) -> Result<toml::Table, OverlayError> {
    match value {
        None => Ok(toml::Table::new()),
        Some(Value::Table(table)) => Ok(table),
        Some(value) => {
            Err(context.error(key, spelling(&value), Rule::WrongType { expected: "table" }))
        }
    }
}
pub(super) fn map<T>(
    context: &Context<'_>,
    key: &str,
    value: Option<Value>,
    parse: impl Fn(&Context<'_>, Value) -> Result<T, OverlayError>,
) -> Result<BTreeMap<MemberPath, T>, OverlayError> {
    let child = context.child(key);
    table(context, key, value)?
        .into_iter()
        .map(|(key, value)| {
            let path =
                MemberPath::new(&key).map_err(|_| child.error(&key, &key, Rule::MemberPath))?;
            Ok((path, parse(&child.child(&key), value)?))
        })
        .collect()
}
pub(super) fn not_self(
    context: &Context<'_>,
    key: &str,
    name: &OperationName,
) -> Result<(), OverlayError> {
    if context.operation == Some(name.as_str()) {
        return Err(context.error(key, name, Rule::SelfReference));
    }
    Ok(())
}
