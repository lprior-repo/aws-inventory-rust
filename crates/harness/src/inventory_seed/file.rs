mod value;

use std::collections::BTreeMap;

use super::config::SeedError;
use super::scenario::{
    FieldPath, SeedIdentity, SeedInput, SeedInputValue, SeedOutputCheck, SeedRelationship,
    SeedScenario, SeedScope, SeedStep, StepId,
};

const MAX_FILE_BYTES: usize = 1_048_576;
const MAX_STEPS_RAW: usize = 4096;
const MAX_RELATIONSHIPS: usize = 4096;
const MAX_INPUT_KEYS: usize = 64;
const MAX_CHECKS: usize = 64;

pub fn load(content: &str) -> Result<SeedScenario, SeedError> {
    if content.len() > MAX_FILE_BYTES {
        return Err(SeedError::Invalid("seed file exceeds size limit"));
    }
    let doc = toml::from_str::<toml::Value>(content).map_err(|error| {
        SeedError::Message(format!("seed file parse failed: {error}"))
    })?;
    validate_document(&doc)?;
    let service = doc.get("service").ok_or(SeedError::Invalid(
        "seed file has no service field",
    ))?;
    let service = match service {
        toml::Value::String(s) => s.clone(),
        _ => {
            return Err(SeedError::Invalid(
                "seed file service must be a string",
            ));
        }
    };
    let steps = doc
        .get("step")
        .and_then(|v| match v {
            toml::Value::Array(arr) => Some(arr.clone()),
            _ => None,
        })
        .ok_or(SeedError::Invalid("seed file has no steps"))?;
    let relationships = doc
        .get("relationship")
        .and_then(|v| match v {
            toml::Value::Array(arr) => Some(arr.clone()),
            _ => None,
        })
        .unwrap_or_default();
    validate_raw_steps(&steps)?;
    validate_raw_relationships(&relationships)?;
    convert(&service, &steps, &relationships)
}

fn validate_document(doc: &toml::Value) -> Result<(), SeedError> {
    let format = doc.get("format").ok_or(SeedError::Invalid(
        "seed file has no format field",
    ))?;
    match format {
        toml::Value::Integer(f) if *f == 1 => Ok(()),
        toml::Value::Integer(f) => Err(SeedError::Message(format!(
            "seed file has an unsupported format version: {f}"
        ))),
        _ => Err(SeedError::Invalid("seed file format must be an integer")),
    }
}

fn validate_raw_steps(steps: &[toml::Value]) -> Result<(), SeedError> {
    if steps.is_empty() || steps.len() > MAX_STEPS_RAW {
        return Err(SeedError::Invalid("seed file exceeds step limit"));
    }
    let mut seen = BTreeSet::new();
    for step in steps {
        let id = step
            .get("id")
            .ok_or(SeedError::Invalid("seed step has no id"))?;
        let id = match id {
            toml::Value::String(s) => s,
            _ => {
                return Err(SeedError::Invalid(
                    "seed step id must be a string",
                ));
            }
        };
        if id.is_empty() || id.len() > 128 {
            return Err(SeedError::Invalid(
                "seed step id is invalid",
            ));
        }
        if !seen.insert(id.clone()) {
            return Err(SeedError::Invalid(
                "seed file has a duplicate step id",
            ));
        }
        let operation = step
            .get("operation")
            .ok_or(SeedError::Invalid("seed step has no operation"))?;
        if !matches!(operation, toml::Value::String(s) if !s.is_empty()) {
            return Err(SeedError::Invalid(
                "seed step operation is invalid",
            ));
        }
        if let Some(repeat) = step.get("repeat") {
            match repeat {
                toml::Value::Integer(n) if *n > 0 && *n <= 4096 => {}
                toml::Value::Integer(_) => {
                    return Err(SeedError::Invalid(
                        "seed step repeat is invalid",
                    ));
                }
                _ => {
                    return Err(SeedError::Invalid(
                        "seed step repeat must be an integer",
                    ));
                }
            }
        }
        if let Some(input) = step.get("input") {
            match input {
                toml::Value::Table(table) if table.len() <= MAX_INPUT_KEYS => {}
                toml::Value::Table(_) => {
                    return Err(SeedError::Invalid(
                        "seed step has too many input fields",
                    ));
                }
                _ => {
                    return Err(SeedError::Invalid(
                        "seed step input must be a table",
                    ));
                }
            }
        }
        if let Some(check) = step.get("check") {
            match check {
                toml::Value::Table(table) if table.len() <= MAX_CHECKS => {}
                toml::Value::Table(_) => {
                    return Err(SeedError::Invalid(
                        "seed step has too many checks",
                    ));
                }
                _ => {
                    return Err(SeedError::Invalid(
                        "seed step check must be a table",
                    ));
                }
            }
        }
    }
    Ok(())
}

fn validate_raw_relationships(
    relationships: &[toml::Value],
) -> Result<(), SeedError> {
    if relationships.len() > MAX_RELATIONSHIPS {
        return Err(SeedError::Invalid(
            "seed file exceeds relationship limit",
        ));
    }
    Ok(())
}

fn convert(
    service: &str,
    raw_steps: &[toml::Value],
    raw_relationships: &[toml::Value],
) -> Result<SeedScenario, SeedError> {
    let mut id_map = BTreeMap::new();
    let mut repeat_map = BTreeMap::new();
    let mut steps = Vec::new();

    let max_repeat = raw_steps
        .iter()
        .map(|raw_step| {
            let repeat = raw_step
                .get("repeat")
                .and_then(|v| match v {
                    toml::Value::Integer(i) => Some(*i),
                    _ => None,
                })
                .unwrap_or(1);
            u16::try_from(repeat).unwrap_or(1)
        })
        .max()
        .unwrap_or(1);

    for raw_step in raw_steps {
        let step_id_str = raw_step
            .get("id")
            .and_then(|v| match v {
                toml::Value::String(s) => Some(s),
                _ => None,
            })
            .ok_or(SeedError::Invalid("seed step has no id"))?;

        let repeat = raw_step
            .get("repeat")
            .and_then(|v| match v {
                toml::Value::Integer(i) => Some(*i),
                _ => None,
            })
            .unwrap_or(1);

        let repeat = u16::try_from(repeat).map_err(|_| {
            SeedError::Invalid("seed step repeat exceeds its limit")
        })?;

        repeat_map.insert(step_id_str.clone(), repeat);
    }

    for i in 0..max_repeat {
        for raw_step in raw_steps {
            let step_id_str = raw_step
                .get("id")
                .and_then(|v| match v {
                    toml::Value::String(s) => Some(s),
                    _ => None,
                })
                .ok_or(SeedError::Invalid("seed step has no id"))?;

            let repeat = *repeat_map.get(step_id_str).unwrap_or(&1);
            if i >= repeat {
                continue;
            }

            let step_id = StepId::new(u16::try_from(steps.len()).map_err(|_| {
                SeedError::Invalid("seed step id exceeds its limit")
            })?);

            let instance_id = format!("{step_id_str}-{i}");
            id_map.insert(instance_id.clone(), step_id);

            if i == 0 {
                id_map.insert(step_id_str.clone(), step_id);
            }

            let seed_step = convert_step(step_id, service, raw_step, &id_map, i, repeat)?;
            steps.push(seed_step);
        }
    }

    let mut relationships = Vec::new();
    for raw_rel in raw_relationships {
        let source_name = raw_rel
            .get("source")
            .and_then(|v| match v {
                toml::Value::String(s) => Some(s),
                _ => None,
            })
            .ok_or(SeedError::Invalid("seed relationship has no source"))?;

        let target_name = raw_rel
            .get("target")
            .and_then(|v| match v {
                toml::Value::String(s) => Some(s),
                _ => None,
            })
            .ok_or(SeedError::Invalid("seed relationship has no target"))?;

        let source_repeat = *repeat_map.get(source_name).ok_or(SeedError::Invalid(
            "seed relationship source is unknown",
        ))?;
        let target_repeat = *repeat_map.get(target_name).ok_or(SeedError::Invalid(
            "seed relationship target is unknown",
        ))?;

        if source_repeat != target_repeat {
            return Err(SeedError::Invalid(
                "seed relationship source and target repeat must match",
            ));
        }

        for i in 0..source_repeat {
            let source_instance = format!("{source_name}-{i}");
            let target_instance = format!("{target_name}-{i}");

            let source_step = id_map.get(&source_instance).ok_or(SeedError::Invalid(
                "seed relationship source instance is unknown",
            ))?;
            let target_step = id_map.get(&target_instance).ok_or(SeedError::Invalid(
                "seed relationship target instance is unknown",
            ))?;

            let relation = raw_rel
                .get("relation")
                .and_then(|v| match v {
                    toml::Value::String(s) => Some(s),
                    _ => None,
                })
                .ok_or(SeedError::Invalid("seed relationship has no relation"))?
                .to_string();

            let observed = raw_rel
                .get("observed")
                .and_then(|v| match v {
                    toml::Value::String(s) => Some(s),
                    _ => None,
                })
                .ok_or(SeedError::Invalid("seed relationship has no observed path"))?;

            relationships.push(SeedRelationship::new(
                *source_step,
                *target_step,
                relation,
                FieldPath::new(&split_path(observed))?,
            ));
        }
    }

    SeedScenario::with_relationships(steps, relationships)
}

fn convert_step(
    id: StepId,
    service: &str,
    raw_step: &toml::Value,
    id_map: &BTreeMap<String, StepId>,
    instance_index: u16,
    repeat_count: u16,
) -> Result<SeedStep, SeedError> {
    let operation = raw_step
        .get("operation")
        .and_then(|v| match v {
            toml::Value::String(s) => Some(s),
            _ => None,
        })
        .ok_or(SeedError::Invalid("seed step has no operation"))?;
    let kind = raw_step
        .get("kind")
        .and_then(|v| match v {
            toml::Value::String(s) => Some(s),
            _ => None,
        })
        .cloned()
        .unwrap_or_default();
    let identity = raw_step
        .get("identity")
        .and_then(|v| match v {
            toml::Value::String(s) => Some(s),
            _ => None,
        })
        .ok_or(SeedError::Invalid("seed step has no identity"))?;
    let identity: SeedIdentity = FieldPath::new(&split_path(identity))?.into();
    let scope = match raw_step
        .get("scope")
        .and_then(|v| match v {
            toml::Value::String(s) => Some(s),
            _ => None,
        })
    {
        Some(s) if s == "account" => SeedScope::Account,
        Some(s) if s == "region" => SeedScope::Regional,
        None => SeedScope::Regional,
        Some(other) => {
            let message = format!("seed step has an unknown scope: {other}");
            return Err(SeedError::Message(message));
        }
    };
    let inputs = convert_inputs(raw_step, id_map, instance_index, repeat_count)?;
    let checks = convert_checks(raw_step, id_map)?;
    let mut seed_step = SeedStep::new(id, service.to_string(), operation.to_string(), kind, identity, inputs);
    if !checks.is_empty() {
        seed_step = seed_step.with_response_checks(checks);
    }
    seed_step = seed_step.with_scope(scope);
    Ok(seed_step)
}

fn convert_inputs(
    raw_step: &toml::Value,
    id_map: &BTreeMap<String, StepId>,
    instance_index: u16,
    repeat_count: u16,
) -> Result<Vec<SeedInput>, SeedError> {
    let mut inputs = Vec::new();
    let input_table = raw_step
        .get("input")
        .and_then(|v| match v {
            toml::Value::Table(table) => Some(table),
            _ => None,
        });
    if let Some(input) = input_table {
        for (field, value) in input {
            let path = FieldPath::new(&split_path(field))?;
            let input_value = value::convert(value, id_map, instance_index, repeat_count)?;
            inputs.push(SeedInput::new(path, input_value));
        }
    }
    Ok(inputs)
}

fn convert_checks(
    raw_step: &toml::Value,
    id_map: &BTreeMap<String, StepId>,
) -> Result<Vec<SeedOutputCheck>, SeedError> {
    let mut checks = Vec::new();
    let check_table = raw_step
        .get("check")
        .and_then(|v| match v {
            toml::Value::Table(table) => Some(table),
            _ => None,
        });
    if let Some(check) = check_table {
        for (output, input) in check {
            let output_path = FieldPath::new(&split_path(output))?;
            let input_path = value::convert(input, id_map, 0, 1)?;
            let input_path = match input_path {
                SeedInputValue::Text(s) => FieldPath::new(&split_path(&s))?,
                _ => {
                    return Err(SeedError::Invalid(
                        "seed check input must be a field path string",
                    ));
                }
            };
            checks.push(SeedOutputCheck::new(output_path, input_path));
        }
    }
    Ok(checks)
}

fn convert_relationship(
    id_map: &BTreeMap<String, StepId>,
    rel: &toml::Value,
) -> Result<SeedRelationship, SeedError> {
    let source = rel
        .get("source")
        .and_then(|v| match v {
            toml::Value::String(s) => Some(s),
            _ => None,
        })
        .ok_or(SeedError::Invalid(
            "seed relationship has no source",
        ))?;
    let target = rel
        .get("target")
        .and_then(|v| match v {
            toml::Value::String(s) => Some(s),
            _ => None,
        })
        .ok_or(SeedError::Invalid(
            "seed relationship has no target",
        ))?;
    let relation = rel
        .get("relation")
        .and_then(|v| match v {
            toml::Value::String(s) => Some(s),
            _ => None,
        })
        .ok_or(SeedError::Invalid(
            "seed relationship has no relation",
        ))?;
    let observed = rel
        .get("observed")
        .and_then(|v| match v {
            toml::Value::String(s) => Some(s),
            _ => None,
        })
        .ok_or(SeedError::Invalid(
            "seed relationship has no observed path",
        ))?;
    let source_id = id_map
        .get(source)
        .ok_or(SeedError::Invalid(
            "seed relationship source step is missing",
        ))?
        .clone();
    let target_id = id_map
        .get(target)
        .ok_or(SeedError::Invalid(
            "seed relationship target step is missing",
        ))?
        .clone();
    let observed_path = FieldPath::new(&split_path(observed))?;
    Ok(SeedRelationship::new(
        source_id,
        target_id,
        relation.to_string(),
        observed_path,
    ))
}

fn split_path(path: &str) -> Vec<&str> {
    path.split('.').collect()
}

use std::collections::BTreeSet;