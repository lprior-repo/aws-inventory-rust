use super::{StoreError, scalar, unreviewed};
use inventory_core::{
    catalog::runtime::{BindingSource, BindingTransform, BindingValue, CallablePlan, ExecutionBranch},
    record::{FieldId, Key, KeyRef, MapRef, ValueRef},
};

#[derive(Clone, Debug)]
pub(super) struct FixedBinding {
    path: Box<[FieldId]>,
    value: BindingValue,
}

pub(super) fn compile(plan: &CallablePlan<'_>, branch: Option<&ExecutionBranch>) -> Result<Vec<FixedBinding>, StoreError> {
    plan.effective_bindings(branch).filter_map(|binding| {
        match fixed_value(&binding.source) {
            Ok(Some(mut value)) => {
                normalize(&mut value);
                Some(Ok(FixedBinding { path: binding.path.clone(), value }))
            }
            Ok(None) => None,
            Err(error) => Some(Err(error)),
        }
    }).collect()
}

pub(super) fn validate(bindings: &[FixedBinding], request: &[u8]) -> Result<(), StoreError> {
    if bindings.is_empty() { return Ok(()); }
    let root = MapRef::from_bytes_bounded(request)?;
    for binding in bindings {
        let Some(value) = scalar(root, &binding.path)? else { return Err(unreviewed()); };
        if !same(&binding.value, value)? { return Err(unreviewed()); }
    }
    Ok(())
}

fn fixed_value(source: &BindingSource) -> Result<Option<BindingValue>, StoreError> {
    match source {
        BindingSource::Fixed(value) => Ok(Some(value.clone())),
        BindingSource::Transformed { source, transform } => {
            let Some(value) = fixed_value(source)? else { return Ok(None); };
            let BindingValue::String(mut value) = value else { return Err(unreviewed()); };
            match transform {
                BindingTransform::StripPrefix(prefix) => {
                    if value.starts_with(prefix.as_ref()) { value.drain(..prefix.len()); }
                }
            }
            if value.is_empty() || value.contains('/') { return Err(unreviewed()); }
            Ok(Some(BindingValue::String(value)))
        }
        _ => Ok(None),
    }
}

// Fixed trees are already depth/node bounded by the catalog decoder. Sort object
// keys once here so checking a leased request walks each bounded tree only once.
fn normalize(value: &mut BindingValue) {
    match value {
        BindingValue::List(values) => values.iter_mut().for_each(normalize),
        BindingValue::Object(values) => {
            values.sort_unstable_by(|left, right| left.0.cmp(&right.0));
            values.iter_mut().for_each(|(_, value)| normalize(value));
        }
        _ => {}
    }
}

fn same(expected: &BindingValue, actual: ValueRef<'_>) -> Result<bool, StoreError> {
    let equal = match (expected, actual) {
        (BindingValue::String(expected), ValueRef::Text(actual)) => expected.as_str() == actual,
        (BindingValue::Integer(expected), ValueRef::Int(actual)) => *expected == actual,
        (BindingValue::Boolean(expected), ValueRef::Bool(actual)) => *expected == actual,
        (BindingValue::Float(expected), ValueRef::Float(actual)) => *expected == actual,
        (BindingValue::Bytes(expected), ValueRef::Bytes(actual)) => expected.as_ref() == actual,
        (BindingValue::Timestamp(expected), ValueRef::Time(actual)) => *expected == actual,
        (BindingValue::Strings(expected), ValueRef::List(actual)) => {
            if actual.count()? != expected.len() { return Ok(false); }
            for (expected, actual) in expected.iter().zip(actual.iter()?) {
                if !matches!(actual?, ValueRef::Text(actual) if actual == expected.as_str()) { return Ok(false); }
            }
            true
        }
        (BindingValue::List(expected), ValueRef::List(actual)) => {
            if actual.count()? != expected.len() { return Ok(false); }
            for (expected, actual) in expected.iter().zip(actual.iter()?) {
                if !same(expected, actual?)? { return Ok(false); }
            }
            true
        }
        (BindingValue::Object(expected), ValueRef::Map(actual)) => {
            if actual.count()? != expected.len() { return Ok(false); }
            for ((expected_key, expected), actual) in expected.iter().zip(actual.iter()?) {
                let (actual_key, actual) = actual?;
                let expected_key = match expected_key { Key::Field(field) => KeyRef::Field(*field), Key::Name(name) => KeyRef::Name(name) };
                if expected_key != actual_key || !same(expected, actual)? { return Ok(false); }
            }
            true
        }
        _ => false,
    };
    Ok(equal)
}
