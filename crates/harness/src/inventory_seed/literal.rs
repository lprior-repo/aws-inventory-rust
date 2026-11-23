mod model;
use super::SeedError;
use inventory_core::record::{Key, Value};
pub(super) use model::{resolve, resolve_template};

pub(super) fn validate(value: &Value) -> Result<(), SeedError> {
    validate_nodes(value, false).map(|_| ())
}

pub(super) fn validate_template(value: &Value) -> Result<usize, SeedError> {
    validate_nodes(value, true)
}

fn validate_nodes(value: &Value, placeholders: bool) -> Result<usize, SeedError> {
    let mut pending = vec![(value, 0_usize)];
    let mut count = 0_usize;
    for _ in 0..256 {
        let Some((value, depth)) = pending.pop() else {
            return Ok(count);
        };
        if depth > 8 {
            return Err(SeedError::Invalid("seed literal exceeds its depth limit"));
        }
        if matches!(value, Value::Null) && placeholders {
            count = count
                .checked_add(1)
                .ok_or(SeedError::Invalid("seed placeholder count overflow"))?;
        } else {
            validate_node(value, depth, &mut pending)?;
        }
    }
    if pending.is_empty() {
        Ok(count)
    } else {
        Err(SeedError::Invalid("seed literal exceeds its node limit"))
    }
}

fn validate_node<'a>(
    value: &'a Value,
    depth: usize,
    pending: &mut Vec<(&'a Value, usize)>,
) -> Result<(), SeedError> {
    let next = depth
        .checked_add(1)
        .ok_or(SeedError::Invalid("seed literal depth overflow"))?;
    match value {
        Value::Text(value) if value.len() <= super::MAX_VALUE_BYTES => Ok(()),
        Value::Bool(_) | Value::Int(_) => Ok(()),
        Value::Float(value) if value.to_f64().is_finite() => Ok(()),
        Value::List(items) if items.len() <= 256 => {
            pending.extend(items.iter().map(|item| (item, next)));
            Ok(())
        }
        Value::Map(items) if items.len() <= 256 => {
            if items
                .keys()
                .any(|key| !matches!(key, Key::Name(name) if !name.is_empty() && name.len() <= 128))
            {
                return Err(SeedError::Invalid(
                    "seed literal map keys must be bounded names",
                ));
            }
            pending.extend(items.values().map(|item| (item, next)));
            Ok(())
        }
        _ => Err(SeedError::Invalid("seed literal exceeds its value bounds")),
    }
}

enum Action<T> {
    Visit(T, usize),
    Finish(Option<Vec<Key>>, usize),
}

fn finish(keys: Option<Vec<Key>>, length: usize, values: &mut Vec<Value>) -> Result<(), SeedError> {
    let offset = values
        .len()
        .checked_sub(length)
        .ok_or(SeedError::Invalid("seed literal container is incomplete"))?;
    if keys.as_ref().is_some_and(|keys| keys.len() != length) {
        return Err(SeedError::Invalid(
            "seed literal container keys are incomplete",
        ));
    }
    let children = values.split_off(offset);
    let value = match keys {
        Some(keys) => Value::Map(keys.into_iter().zip(children).collect()),
        None => Value::List(children),
    };
    values.push(value);
    Ok(())
}
