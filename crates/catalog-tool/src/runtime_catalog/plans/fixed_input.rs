use super::RuntimeCatalogCompileError;
use crate::overlay::FixedValue;
use inventory_core::{
    catalog::runtime::BindingValue,
    catalog::{Service, Shape, Target},
    record::{Key, Value, ValueRef},
};
mod collections;
mod scalar;
use collections::{list, map, strings, structure};
use scalar::{enum_value, scalar, shape, text_bound};

fn unsupported() -> RuntimeCatalogCompileError {
    RuntimeCatalogCompileError::Capability
}

struct Budget {
    nodes: usize,
    bytes: usize,
}
enum Work<'a> {
    Read(&'a FixedValue, Target, usize),
    List(usize),
    Object(Vec<Key>),
}
struct State<'a, 'w> {
    work: &'w mut Vec<Work<'a>>,
    output: &'w mut Vec<Value>,
    budget: &'w mut Budget,
}

pub(super) fn compile(
    service: Service<'_>,
    target: Target,
    value: &FixedValue,
    path: &str,
) -> Result<Value, RuntimeCatalogCompileError> {
    compile_value(service, target, value).map_err(|error| match error {
        RuntimeCatalogCompileError::Capability | RuntimeCatalogCompileError::Path { .. } => {
            RuntimeCatalogCompileError::Path {
                path: path.to_owned(),
            }
        }
        error => error,
    })
}

fn compile_value(
    service: Service<'_>,
    target: Target,
    value: &FixedValue,
) -> Result<Value, RuntimeCatalogCompileError> {
    let mut work = vec![Work::Read(value, target, 0)];
    let mut output = Vec::new();
    let mut budget = Budget { nodes: 0, bytes: 0 };
    let mut state = State {
        work: &mut work,
        output: &mut output,
        budget: &mut budget,
    };
    drain(service, &mut state)?;
    if !state.work.is_empty() || state.output.len() != 1 {
        return Err(unsupported());
    }
    state.output.pop().ok_or_else(unsupported)
}

fn drain(
    service: Service<'_>,
    state: &mut State<'_, '_>,
) -> Result<(), RuntimeCatalogCompileError> {
    for _ in 0..BindingValue::MAX_NODES.saturating_mul(2) {
        let Some(next) = state.work.pop() else {
            return Ok(());
        };
        process(service, next, state)?;
        if state.work.len().saturating_add(state.budget.nodes)
            > BindingValue::MAX_NODES.saturating_mul(2)
        {
            return Err(unsupported());
        }
    }
    Ok(())
}

fn process<'a>(
    service: Service<'_>,
    next: Work<'a>,
    state: &mut State<'a, '_>,
) -> Result<(), RuntimeCatalogCompileError> {
    match next {
        Work::Read(value, target, depth) => {
            read_checked(service, target, value, depth, state)?;
        }
        Work::List(length) => {
            let children = split(state.output, length)?;
            state.output.push(Value::List(children));
        }
        Work::Object(keys) => {
            let children = split(state.output, keys.len())?;
            state
                .output
                .push(Value::Map(keys.into_iter().zip(children).collect()));
        }
    }
    Ok(())
}

fn read_checked<'a>(
    service: Service<'_>,
    target: Target,
    value: &'a FixedValue,
    depth: usize,
    state: &mut State<'a, '_>,
) -> Result<(), RuntimeCatalogCompileError> {
    state.budget.nodes = state.budget.nodes.checked_add(1).ok_or_else(unsupported)?;
    if state.budget.nodes > BindingValue::MAX_NODES || depth > BindingValue::MAX_DEPTH {
        return Err(unsupported());
    }
    read(service, target, value, depth, state)
}

fn read<'a>(
    service: Service<'_>,
    target: Target,
    value: &'a FixedValue,
    depth: usize,
    state: &mut State<'a, '_>,
) -> Result<(), RuntimeCatalogCompileError> {
    let value = match (value, super::super::model_path::kind(service, target)?) {
        (FixedValue::Strings(values), "list") => strings(service, target, values, depth, state)?,
        (FixedValue::List(values), "list") => {
            return list(service, target, values, depth, state);
        }
        (FixedValue::Structure(values), "structure") => {
            return structure(service, target, values, depth, state);
        }
        (FixedValue::Map(values), "map") => {
            return map(service, target, values, depth, state);
        }
        (value, kind) => scalar(service, target, kind, value, &mut state.budget.bytes)?,
    };
    state.output.push(value);
    Ok(())
}

fn split(values: &mut Vec<Value>, length: usize) -> Result<Vec<Value>, RuntimeCatalogCompileError> {
    let start = values.len().checked_sub(length).ok_or_else(unsupported)?;
    Ok(values.split_off(start))
}
