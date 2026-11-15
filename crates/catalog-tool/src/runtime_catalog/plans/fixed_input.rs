use super::RuntimeCatalogCompileError;
use crate::overlay::FixedValue;
use inventory_core::{catalog::{Service, Shape, Target}, catalog::runtime::BindingValue, record::{Key, Value, ValueRef}};
use std::ops::ControlFlow;

struct Budget { nodes: usize, bytes: usize }

enum Work<'a> { Read(&'a FixedValue, Target, usize), List(usize), Object(Vec<Key>) }

struct State<'a, 'w> {
    work: &'w mut Vec<Work<'a>>,
    output: &'w mut Vec<Value>,
    budget: &'w mut Budget,
}

pub(super) fn compile(service: Service<'_>, target: Target, value: &FixedValue) -> Result<Value, RuntimeCatalogCompileError> {
    let mut work = vec![Work::Read(value, target, 0)];
    let mut output = Vec::new();
    let mut budget = Budget { nodes: 0, bytes: 0 };
    let outcome = (0..BindingValue::MAX_NODES.saturating_mul(2)).try_for_each(|_| {
        let Some(next) = work.pop() else { return ControlFlow::Break(Ok(())); };
        let result = (|| {
        match next {
            Work::Read(value, target, depth) => {
                budget.nodes = budget.nodes.checked_add(1).ok_or(RuntimeCatalogCompileError::Capability)?;
                if budget.nodes > BindingValue::MAX_NODES || depth > BindingValue::MAX_DEPTH { return Err(RuntimeCatalogCompileError::Capability); }
                read(service, target, value, depth, &mut State { work: &mut work, output: &mut output, budget: &mut budget })?;
            }
            Work::List(length) => { let children = split(&mut output, length)?; output.push(Value::List(children)); }
            Work::Object(keys) => { let children = split(&mut output, keys.len())?; output.push(Value::Map(keys.into_iter().zip(children).collect())); }
        }
        if work.len().saturating_add(budget.nodes) > BindingValue::MAX_NODES.saturating_mul(2) { return Err(RuntimeCatalogCompileError::Capability); }
        Ok(())
        })();
        match result { Ok(()) => ControlFlow::Continue(()), Err(error) => ControlFlow::Break(Err(error)) }
    });
    if let ControlFlow::Break(result) = outcome { result?; }
    if !work.is_empty() || output.len() != 1 { return Err(RuntimeCatalogCompileError::Capability); }
    output.pop().ok_or(RuntimeCatalogCompileError::Capability)
}

fn read<'a>(service: Service<'_>, target: Target, value: &'a FixedValue, depth: usize, state: &mut State<'a, '_>) -> Result<(), RuntimeCatalogCompileError> {
    let kind = super::super::model_path::kind(service, target)?;
    let scalar = match value {
        FixedValue::Strings(values) if kind == "list" => Some(strings(service, target, values, depth, state)?),
        FixedValue::List(values) if kind == "list" => { list(service, target, values, depth, state)?; None }
        FixedValue::Structure(values) if kind == "structure" => { structure(service, target, values, depth, state)?; None }
        FixedValue::Map(values) if kind == "map" => { map(service, target, values, depth, state)?; None }
        _ => Some(scalar(service, target, kind, value, &mut state.budget.bytes)?),
    };
    if let Some(value) = scalar { state.output.push(value); }
    Ok(())
}
fn scalar(service: Service<'_>, target: Target, kind: &str, value: &FixedValue, bytes: &mut usize) -> Result<Value, RuntimeCatalogCompileError> {
    match value {
        FixedValue::String(text) if matches!(kind, "string" | "enum") => {
            text_bound(text.len(), bytes)?;
            enum_value(service, target, ValueRef::Text(text))?;
            Ok(Value::Text(text.clone()))
        }
        FixedValue::Integer(number) if integer(kind, *number) => {
            enum_value(service, target, ValueRef::Int(*number))?;
            Ok(Value::Int(*number))
        }
        FixedValue::Boolean(value) if kind == "boolean" => Ok(Value::Bool(*value)),
        FixedValue::Float(number) if matches!(kind, "float" | "double") && number.to_f64().is_finite()
            && (kind == "double" || number.to_f64().abs() <= f64::from(f32::MAX)) => Ok(Value::Float(*number)),
        FixedValue::Bytes(value) if kind == "blob" => { text_bound(value.len(), bytes)?; Ok(Value::Bytes(value.clone())) }
        FixedValue::Timestamp(value) if kind == "timestamp" => Ok(Value::Time(*value)),
        _ => Err(RuntimeCatalogCompileError::Capability),
    }
}
fn strings(service: Service<'_>, target: Target, values: &[String], depth: usize, state: &mut State<'_, '_>) -> Result<Value, RuntimeCatalogCompileError> {
    collection(values.len())?;
    let child = shape(service, target)?.element()?.ok_or(RuntimeCatalogCompileError::Capability)?;
    if !matches!(super::super::model_path::kind(service, child)?, "string" | "enum") || (depth >= BindingValue::MAX_DEPTH && !values.is_empty()) { return Err(RuntimeCatalogCompileError::Capability); }
    state.budget.nodes = state.budget.nodes.checked_add(values.len()).ok_or(RuntimeCatalogCompileError::Capability)?;
    if state.budget.nodes > BindingValue::MAX_NODES { return Err(RuntimeCatalogCompileError::Capability); }
    Ok(Value::List(values.iter().map(|text| {
        text_bound(text.len(), &mut state.budget.bytes)?;
        enum_value(service, child, ValueRef::Text(text))?;
        Ok(Value::Text(text.clone()))
    }).collect::<Result<_, RuntimeCatalogCompileError>>()?))
}
fn list<'a>(service: Service<'_>, target: Target, values: &'a [FixedValue], depth: usize, state: &mut State<'a, '_>) -> Result<(), RuntimeCatalogCompileError> {
    collection(values.len())?;
    let child = shape(service, target)?.element()?.ok_or(RuntimeCatalogCompileError::Capability)?;
    state.work.push(Work::List(values.len()));
    state.work.extend(values.iter().rev().map(|value| Work::Read(value, child, depth.saturating_add(1))));
    Ok(())
}
fn structure<'a>(service: Service<'_>, target: Target, values: &'a std::collections::BTreeMap<String, FixedValue>, depth: usize, state: &mut State<'a, '_>) -> Result<(), RuntimeCatalogCompileError> {
    collection(values.len())?;
    let shape = shape(service, target)?;
    let members = shape.members()?.collect::<Result<Vec<_>, _>>()?;
    if members.iter().any(|member| member.required && member.name().map_or(true, |name| !values.contains_key(name))) { return Err(RuntimeCatalogCompileError::Capability); }
    let children = values.iter().map(|(name, value)| {
        text_bound(name.len(), &mut state.budget.bytes)?;
        let member = members.iter().find(|member| member.name().is_ok_and(|member_name| member_name == name)).ok_or(RuntimeCatalogCompileError::Capability)?;
        Ok((Key::Field(member.id), value, member.target))
    }).collect::<Result<Vec<_>, RuntimeCatalogCompileError>>()?;
    state.work.push(Work::Object(children.iter().map(|(key, _, _)| key.clone()).collect()));
    state.work.extend(children.into_iter().rev().map(|(_, value, target)| Work::Read(value, target, depth.saturating_add(1))));
    Ok(())
}
fn map<'a>(service: Service<'_>, target: Target, values: &'a std::collections::BTreeMap<String, FixedValue>, depth: usize, state: &mut State<'a, '_>) -> Result<(), RuntimeCatalogCompileError> {
    collection(values.len())?;
    let shape = shape(service, target)?;
    let key = shape.map_key()?.ok_or(RuntimeCatalogCompileError::Capability)?;
    if !matches!(super::super::model_path::kind(service, key)?, "string" | "enum") { return Err(RuntimeCatalogCompileError::Capability); }
    let child = shape.map_value()?.ok_or(RuntimeCatalogCompileError::Capability)?;
    values.keys().try_for_each(|name| {
        text_bound(name.len(), &mut state.budget.bytes)?;
        enum_value(service, key, ValueRef::Text(name))
    })?;
    state.work.push(Work::Object(values.keys().cloned().map(Key::Name).collect()));
    state.work.extend(values.values().rev().map(|value| Work::Read(value, child, depth.saturating_add(1))));
    Ok(())
}
fn shape(service: Service<'_>, target: Target) -> Result<Shape<'_>, RuntimeCatalogCompileError> {
    match target { Target::Shape(index) => service.shape(index)?.ok_or(RuntimeCatalogCompileError::Capability), _ => Err(RuntimeCatalogCompileError::Capability) }
}
fn split(values: &mut Vec<Value>, length: usize) -> Result<Vec<Value>, RuntimeCatalogCompileError> {
    let start = values.len().checked_sub(length).ok_or(RuntimeCatalogCompileError::Capability)?;
    Ok(values.split_off(start))
}
fn text_bound(length: usize, bytes: &mut usize) -> Result<(), RuntimeCatalogCompileError> {
    *bytes = bytes.checked_add(length).ok_or(RuntimeCatalogCompileError::Capability)?;
    if length > BindingValue::MAX_STRING || *bytes > BindingValue::MAX_TEXT_BYTES { Err(RuntimeCatalogCompileError::Capability) } else { Ok(()) }
}
fn collection(length: usize) -> Result<(), RuntimeCatalogCompileError> {
    if length > BindingValue::MAX_COLLECTION { Err(RuntimeCatalogCompileError::Capability) } else { Ok(()) }
}
fn integer(kind: &str, value: i64) -> bool {
    match kind { "byte" => i8::try_from(value).is_ok(), "short" => i16::try_from(value).is_ok(), "integer" | "intEnum" => i32::try_from(value).is_ok(), "long" => true, _ => false }
}
fn enum_value(service: Service<'_>, target: Target, value: ValueRef<'_>) -> Result<(), RuntimeCatalogCompileError> {
    if let Target::Shape(_) = target {
        if !shape(service, target)?.accepts_enum_value(value)? { return Err(RuntimeCatalogCompileError::Capability); }
    }
    Ok(())
}
