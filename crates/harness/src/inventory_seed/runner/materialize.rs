use super::*;
use crate::inventory_seed::model::{PreparedTemplate, Selector};
use inventory_core::record::Key;

pub(super) fn output_value(
    state: &SeedState,
    step: super::super::StepId,
    path: &[Selector],
) -> Result<Value, SeedError> {
    let output = state.accepted_output(step).ok_or_else(|| {
        SeedError::Message(format!(
            "seed binding output from step {} is not available",
            step.0
        ))
    })?;
    Ok(value_path(output.record.root(), path)?.to_value()?)
}

pub(super) fn template(state: &SeedState, template: &PreparedTemplate) -> Result<Value, SeedError> {
    template
        .bindings
        .iter()
        .try_fold(template.value.clone(), |mut value, binding| {
            let replacement = output_value(state, binding.step, &binding.path)?;
            let slot = select(&mut value, &binding.destination)?;
            if !matches!(slot, Value::Null) {
                return Err(SeedError::Invalid(
                    "nested seed binding does not select a placeholder",
                ));
            }
            *slot = replacement;
            Ok(value)
        })
}

fn select<'a>(value: &'a mut Value, path: &[Selector]) -> Result<&'a mut Value, SeedError> {
    path.iter().try_fold(value, |value, selector| {
        let value = match (value, selector) {
            (Value::List(items), Selector::Index(index)) => items.get_mut(*index),
            (Value::Map(items), Selector::Field(field)) => items.get_mut(&Key::Field(*field)),
            (Value::Map(items), Selector::Name(name)) => {
                items.iter_mut().find_map(|(key, value)| {
                    if matches!(key, Key::Name(key) if key == name) {
                        Some(value)
                    } else {
                        None
                    }
                })
            }
            _ => None,
        };
        value.ok_or(SeedError::Invalid(
            "nested seed destination is missing or has the wrong container type",
        ))
    })
}
