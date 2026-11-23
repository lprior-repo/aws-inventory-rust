use super::*;
use inventory_core::record::Key;

enum Task<'a> {
    Visit(&'a PreparedInputValue),
    Finish(Option<Vec<Key>>, usize),
}

pub(super) fn materialize(
    state: &SeedState,
    value: &PreparedInputValue,
) -> Result<Value, SeedError> {
    let mut tasks = vec![Task::Visit(value)];
    let mut values = Vec::new();
    (0..514)
        .find_map(|_| {
            let result = match tasks.pop() {
                Some(Task::Visit(value)) => visit(state, value, &mut tasks, &mut values),
                Some(Task::Finish(keys, count)) => finish(keys, count, &mut values),
                None => {
                    return Some(
                        values
                            .pop()
                            .ok_or(SeedError::Invalid("seed request value is missing")),
                    );
                }
            };
            result.err().map(Err)
        })
        .ok_or(SeedError::Invalid("seed request exceeds node limit"))?
}

fn visit<'a>(
    state: &SeedState,
    value: &'a PreparedInputValue,
    tasks: &mut Vec<Task<'a>>,
    values: &mut Vec<Value>,
) -> Result<(), SeedError> {
    match value {
        PreparedInputValue::List(items) => {
            tasks.push(Task::Finish(None, items.len()));
            tasks.extend(items.iter().rev().map(Task::Visit));
        }
        PreparedInputValue::Structure(fields) => {
            tasks.push(Task::Finish(
                Some(fields.iter().map(|(key, _)| key.clone()).collect()),
                fields.len(),
            ));
            tasks.extend(fields.iter().rev().map(|(_, value)| Task::Visit(value)));
        }
        value => values.push(scalar(state, value)?),
    }
    Ok(())
}

fn finish(keys: Option<Vec<Key>>, count: usize, values: &mut Vec<Value>) -> Result<(), SeedError> {
    let start = values
        .len()
        .checked_sub(count)
        .ok_or(SeedError::Invalid("seed request container is incomplete"))?;
    let children = values.split_off(start);
    values.push(match keys {
        Some(keys) => Value::Map(keys.into_iter().zip(children).collect()),
        None => Value::List(children),
    });
    Ok(())
}

fn scalar(state: &SeedState, value: &PreparedInputValue) -> Result<Value, SeedError> {
    Ok(match value {
        PreparedInputValue::Text(text) => Value::Text(text.clone()),
        PreparedInputValue::Boolean(value) => Value::Bool(*value),
        PreparedInputValue::Integer(value) => Value::Int(*value),
        PreparedInputValue::Literal(value) => value.clone(),
        PreparedInputValue::Template(template) => materialize::template(state, template)?,
        PreparedInputValue::PreviousOutput { step, path } => {
            materialize::output_value(state, *step, path)?
        }
        _ => {
            return Err(SeedError::Invalid(
                "seed request container was not materialized",
            ));
        }
    })
}
