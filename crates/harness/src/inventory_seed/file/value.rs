use super::{expand::Bindings, *};
use inventory_core::record::{Float, Value};

enum Task<'a> {
    Visit(&'a toml::Value),
    Finish(Option<Vec<String>>, usize),
}

pub(super) fn measure(value: &toml::Value) -> Result<(usize, usize), SeedError> {
    let mut pending = vec![(value, 0_usize)];
    let (mut count, mut bytes) = (0_usize, 0_usize);
    (0..257)
        .find_map(|_| {
            let Some((value, depth)) = pending.pop() else {
                return Some(Ok((count, bytes)));
            };
            if depth > 8 {
                return Some(Err(SeedError::Invalid(
                    "seed input exceeds nesting depth 8",
                )));
            }
            count = count.saturating_add(1);
            measure_node(value, depth, &mut pending)
                .and_then(|size| super::bounds::add(bytes, size, 1))
                .map(|size| bytes = size)
                .err()
                .map(Err)
        })
        .ok_or(SeedError::Invalid("seed input exceeds node limit 256"))?
}

fn measure_node<'a>(
    value: &'a toml::Value,
    depth: usize,
    pending: &mut Vec<(&'a toml::Value, usize)>,
) -> Result<usize, SeedError> {
    let next = depth.saturating_add(1);
    match value {
        toml::Value::Array(items) if items.len() <= 256 => {
            pending.extend(items.iter().map(|value| (value, next)));
            Ok(0)
        }
        toml::Value::Table(items) if items.len() <= 256 => {
            pending.extend(items.values().map(|value| (value, next)));
            items
                .keys()
                .try_fold(0, |total, key| super::bounds::add(total, key.len(), 1))
        }
        toml::Value::Array(_) | toml::Value::Table(_) => {
            Err(SeedError::Invalid("seed container exceeds node limit"))
        }
        toml::Value::String(text) => super::bounds::add(0, text.len(), 2),
        _ => Ok(0),
    }
}

pub(super) fn convert(
    value: &toml::Value,
    bindings: &Bindings<'_>,
    position: usize,
    index: u16,
) -> Result<SeedInputValue, SeedError> {
    let mut tasks = vec![Task::Visit(value)];
    let mut values = Vec::new();
    (0..514)
        .find_map(|_| advance(&mut tasks, &mut values, bindings, position, index))
        .ok_or(SeedError::Invalid("seed input exceeds node limit"))?
}

fn visit<'a>(
    value: &'a toml::Value,
    tasks: &mut Vec<Task<'a>>,
    values: &mut Vec<SeedInputValue>,
    bindings: &Bindings<'_>,
    position: usize,
    index: u16,
) -> Result<(), SeedError> {
    match value {
        toml::Value::Array(items) => {
            tasks.push(Task::Finish(None, items.len()));
            tasks.extend(items.iter().rev().map(Task::Visit));
            return Ok(());
        }
        toml::Value::Table(items) => visit_table(items, tasks, values, bindings, position, index)?,
        value => values.push(scalar(value, index)?),
    }
    Ok(())
}

fn visit_table<'a>(
    items: &'a toml::map::Map<String, toml::Value>,
    tasks: &mut Vec<Task<'a>>,
    values: &mut Vec<SeedInputValue>,
    bindings: &Bindings<'_>,
    position: usize,
    index: u16,
) -> Result<(), SeedError> {
    if items.contains_key("from") || items.contains_key("path") {
        keys(items, &["from", "path"])?;
        values.push(SeedInputValue::PreviousOutput {
            step: bindings.source(field(items, "from")?, position, index)?,
            path: path(field(items, "path")?)?,
        });
        return Ok(());
    }
    let names = names(items)?;
    tasks.push(Task::Finish(Some(names), items.len()));
    tasks.extend(items.values().rev().map(Task::Visit));
    Ok(())
}

fn text_value(text: &str, index: u16) -> Result<SeedInputValue, SeedError> {
    let n = index
        .checked_add(1)
        .ok_or(SeedError::Invalid("seed repeat index overflow"))?;
    let expanded = text.replace("{n}", &n.to_string());
    if expanded.is_empty() || expanded.len() > super::super::MAX_VALUE_BYTES {
        return Err(SeedError::Invalid("seed text exceeds its value bound"));
    }
    Ok(SeedInputValue::Text(expanded))
}

fn finish(
    keys: Option<Vec<String>>,
    count: usize,
    values: &mut Vec<SeedInputValue>,
) -> Result<(), SeedError> {
    let start = values
        .len()
        .checked_sub(count)
        .ok_or(SeedError::Invalid("seed container is incomplete"))?;
    let children = values.split_off(start);
    values.push(match keys {
        Some(keys) => SeedInputValue::Structure(keys.into_iter().zip(children).collect()),
        None => SeedInputValue::List(children),
    });
    Ok(())
}

fn names(items: &toml::map::Map<String, toml::Value>) -> Result<Vec<String>, SeedError> {
    items
        .keys()
        .map(|name| {
            if name.is_empty() || name.len() > 128 {
                return Err(SeedError::Invalid("seed field name exceeds its bound"));
            }
            Ok(name.clone())
        })
        .collect()
}

fn scalar(value: &toml::Value, index: u16) -> Result<SeedInputValue, SeedError> {
    Ok(match value {
        toml::Value::String(text) => return text_value(text, index),
        toml::Value::Integer(value) => SeedInputValue::Integer(*value),
        toml::Value::Boolean(value) => SeedInputValue::Boolean(*value),
        toml::Value::Float(value) if value.is_finite() => {
            SeedInputValue::Literal(Value::Float(Float::new(*value)))
        }
        _ => return Err(SeedError::Invalid("seed input has an unsupported value")),
    })
}

fn advance<'a>(
    tasks: &mut Vec<Task<'a>>,
    values: &mut Vec<SeedInputValue>,
    bindings: &Bindings<'_>,
    position: usize,
    index: u16,
) -> Option<Result<SeedInputValue, SeedError>> {
    let result = match tasks.pop() {
        Some(Task::Visit(value)) => visit(value, tasks, values, bindings, position, index),
        Some(Task::Finish(keys, count)) => finish(keys, count, values),
        None => {
            return Some(
                values
                    .pop()
                    .ok_or(SeedError::Invalid("seed value is missing")),
            );
        }
    };
    result.err().map(Err)
}

#[cfg(test)]
mod text_bounds {
    use super::*;
    use crate::inventory_seed::MAX_VALUE_BYTES;

    #[test]
    fn given_text_at_file_bound_when_converted_then_exact_bytes_are_preserved()
    -> Result<(), SeedError> {
        let text = "x".repeat(MAX_VALUE_BYTES);
        let value = text_value(&text, 0)?;
        assert_eq!(value, SeedInputValue::Text(text));
        Ok(())
    }

    #[test]
    fn given_text_over_file_bound_when_converted_then_it_is_rejected() {
        let text = "x".repeat(MAX_VALUE_BYTES.saturating_add(1));
        assert!(matches!(
            text_value(&text, 0),
            Err(SeedError::Invalid("seed text exceeds its value bound"))
        ));
    }

    #[test]
    fn given_empty_text_when_converted_then_it_remains_rejected() {
        assert!(matches!(
            text_value("", 0),
            Err(SeedError::Invalid("seed text exceeds its value bound"))
        ));
    }

    #[test]
    fn given_large_document_text_when_validated_then_literal_bound_matches_seed_text()
    -> Result<(), SeedError> {
        let text = "x".repeat(MAX_VALUE_BYTES);
        crate::inventory_seed::literal::validate(&Value::Text(text))?;
        let text = "x".repeat(MAX_VALUE_BYTES.saturating_add(1));
        assert!(crate::inventory_seed::literal::validate(&Value::Text(text)).is_err());
        Ok(())
    }
}
