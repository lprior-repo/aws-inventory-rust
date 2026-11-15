use super::{FieldPath, SeedError, StepId};
use inventory_core::record::{Key, Value};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SeedValuePart { Field(String), Index(usize) }

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SeedNestedBinding {
    pub destination: Vec<SeedValuePart>,
    pub step: StepId,
    pub path: FieldPath,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SeedTemplate {
    pub value: Value,
    pub bindings: Vec<SeedNestedBinding>,
}

impl SeedTemplate {
    pub(in crate::inventory_seed) fn validate(&self) -> Result<(), SeedError> {
        let count = super::super::literal::validate_template(&self.value)?;
        if count != self.bindings.len() || count == 0 || count > 256 {
            return Err(SeedError::Invalid("seed template must bind every placeholder exactly once"));
        }
        self.bindings.iter().enumerate().try_for_each(|(index, binding)| {
            if binding.destination.len() > 8 || self.bindings.iter().take(index)
                .any(|prior| prior.destination == binding.destination)
                || !matches!(select(&self.value, &binding.destination)?, Value::Null) {
                return Err(SeedError::Invalid("seed template contains an invalid placeholder path"));
            }
            Ok(())
        })
    }
}

fn select<'a>(value: &'a Value, path: &[SeedValuePart]) -> Result<&'a Value, SeedError> {
    path.iter().try_fold(value, |value, part| {
        let next = match (value, part) {
            (Value::Map(items), SeedValuePart::Field(name)) => items.iter().find_map(|(key, value)| {
                if matches!(key, Key::Name(key) if key == name) { Some(value) } else { None }
            }),
            (Value::List(items), SeedValuePart::Index(index)) => items.get(*index),
            _ => None,
        };
        next.ok_or(SeedError::Invalid("seed template placeholder is missing"))
    })
}
