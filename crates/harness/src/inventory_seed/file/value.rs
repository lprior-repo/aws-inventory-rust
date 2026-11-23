use std::collections::BTreeMap;

use super::super::config::SeedError;
use super::super::scenario::{FieldPath, SeedInputValue, StepId};
use super::split_path;

const MAX_NESTING: usize = 8;

pub fn convert(
    value: &toml::Value,
    id_map: &BTreeMap<String, StepId>,
    instance_index: u16,
    repeat_count: u16,
) -> Result<SeedInputValue, SeedError> {
    convert_at_depth(value, 0, id_map, instance_index, repeat_count)
}

fn convert_at_depth(
    value: &toml::Value,
    depth: usize,
    id_map: &BTreeMap<String, StepId>,
    instance_index: u16,
    repeat_count: u16,
) -> Result<SeedInputValue, SeedError> {
    let next = depth
        .checked_add(1)
        .ok_or(SeedError::Invalid("seed input nesting is too deep"))?;
    if next > MAX_NESTING {
        return Err(SeedError::Invalid("seed input nesting is too deep"));
    }
    match value {
        toml::Value::String(s) => Ok(SeedInputValue::Text(s.clone())),
        toml::Value::Integer(i) => Ok(SeedInputValue::Integer(*i)),
        toml::Value::Float(f) => {
            let v = inventory_core::record::Value::Float(
                inventory_core::record::Float::new(*f),
            );
            Ok(SeedInputValue::Literal(v))
        }
        toml::Value::Boolean(b) => Ok(SeedInputValue::Boolean(*b)),
        toml::Value::Array(items) => {
            let converted = items
                .iter()
                .map(|item| convert_at_depth(item, depth, id_map, instance_index, repeat_count))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(SeedInputValue::List(converted))
        }
        toml::Value::Table(table) => {
            if table.len() == 2
                && table.contains_key("from")
                && table.contains_key("path")
            {
                let from = match table.get("from") {
                    Some(toml::Value::String(s)) => s.clone(),
                    _ => {
                        return Err(SeedError::Invalid(
                            "seed binding from must be a string",
                        ));
                    }
                };
                let path = match table.get("path") {
                    Some(toml::Value::String(s)) => s.clone(),
                    _ => {
                        return Err(SeedError::Invalid(
                            "seed binding path must be a string",
                        ));
                    }
                };
                let step = if instance_index > 0 {
                    let instance_name = format!("{from}-{instance_index}");
                    id_map
                        .get(&instance_name)
                        .ok_or(SeedError::Invalid(
                            "seed binding references an unknown step",
                        ))?
                        .clone()
                } else {
                    id_map
                        .get(&from)
                        .ok_or(SeedError::Invalid(
                            "seed binding references an unknown step",
                        ))?
                        .clone()
                };
                Ok(SeedInputValue::PreviousOutput {
                    step,
                    path: FieldPath::new(&split_path(&path))?,
                })
            } else {
                let mut fields = Vec::new();
                for (name, val) in table {
                    if name.is_empty() || name.len() > 128 {
                        return Err(SeedError::Invalid(
                            "seed structure field name is invalid",
                        ));
                    }
                    let converted = convert_at_depth(val, depth, id_map, instance_index, repeat_count)?;
                    fields.push((name.clone(), converted));
                }
                Ok(SeedInputValue::Structure(fields))
            }
        }
        _ => Err(SeedError::Invalid("seed input has an unsupported type")),
    }
}