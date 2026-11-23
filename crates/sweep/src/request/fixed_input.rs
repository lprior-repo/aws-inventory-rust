use crate::SweepError;
use inventory_core::{
    catalog::runtime::BindingValue,
    record::{Key, Value},
};
use std::ops::ControlFlow;

enum Work<'a> {
    Read(&'a BindingValue),
    List(usize),
    Object(Vec<Key>),
}

/// Materialize the already bounded and model-checked tree into the common aws-wire input.
pub(crate) fn value(literal: &BindingValue) -> Result<Value, SweepError> {
    let mut work = vec![Work::Read(literal)];
    let mut output = Vec::new();
    let outcome = (0..BindingValue::MAX_NODES.saturating_mul(2)).try_for_each(|_| {
        let Some(next) = work.pop() else {
            return ControlFlow::Break(Ok(()));
        };
        let result = (|| {
            match next {
                Work::Read(value) => read(value, &mut work, &mut output),
                Work::List(length) => {
                    let children = split(&mut output, length)?;
                    output.push(Value::List(children));
                }
                Work::Object(keys) => {
                    let children = split(&mut output, keys.len())?;
                    output.push(Value::Map(keys.into_iter().zip(children).collect()));
                }
            }
            Ok::<_, SweepError>(())
        })();
        match result {
            Ok(()) => ControlFlow::Continue(()),
            Err(error) => ControlFlow::Break(Err(error)),
        }
    });
    if let ControlFlow::Break(result) = outcome {
        result?;
    }
    if !work.is_empty() || output.len() != 1 {
        return Err(SweepError::Operation("fixed input exceeds compiled bounds"));
    }
    output
        .pop()
        .ok_or(SweepError::Operation("fixed input is absent"))
}
fn read<'a>(value: &'a BindingValue, work: &mut Vec<Work<'a>>, output: &mut Vec<Value>) {
    let scalar = match value {
        BindingValue::String(value) => Some(Value::Text(value.clone())),
        BindingValue::Integer(value) => Some(Value::Int(*value)),
        BindingValue::Boolean(value) => Some(Value::Bool(*value)),
        BindingValue::Float(value) => Some(Value::Float(*value)),
        BindingValue::Bytes(value) => Some(Value::Bytes(value.to_vec())),
        BindingValue::Timestamp(value) => Some(Value::Time(*value)),
        BindingValue::Strings(values) => Some(Value::List(
            values.iter().cloned().map(Value::Text).collect(),
        )),
        BindingValue::List(values) => {
            work.push(Work::List(values.len()));
            work.extend(values.iter().rev().map(Work::Read));
            None
        }
        BindingValue::Object(values) => {
            work.push(Work::Object(
                values.iter().map(|(key, _)| key.clone()).collect(),
            ));
            work.extend(values.iter().rev().map(|(_, value)| Work::Read(value)));
            None
        }
    };
    if let Some(value) = scalar {
        output.push(value);
    }
}
fn split(values: &mut Vec<Value>, length: usize) -> Result<Vec<Value>, SweepError> {
    let start = values
        .len()
        .checked_sub(length)
        .ok_or(SweepError::Operation("fixed input tree is inconsistent"))?;
    Ok(values.split_off(start))
}
