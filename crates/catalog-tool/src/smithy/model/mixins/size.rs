use super::{Member, Shape, ShapeKind};
use std::collections::BTreeMap;

const MAX_MIXIN_BYTES: usize = 256 * 1024 * 1024;

pub(super) fn reserve(budget: &mut usize, bytes: usize) -> Result<(), String> {
    *budget = budget
        .checked_add(bytes)
        .filter(|size| *size <= MAX_MIXIN_BYTES)
        .ok_or_else(size_error)?;
    Ok(())
}

pub(super) fn size_error() -> String {
    "mixin expansion exceeds 268435456 bytes".to_owned()
}

pub(super) fn shape_size(shape: &Shape) -> Result<usize, String> {
    let mut bytes = traits_size(&shape.traits)?;
    if let ShapeKind::Structure { members } | ShapeKind::Union { members } = &shape.kind {
        for (name, member) in members {
            member_size(name, member, &mut bytes)?;
        }
    }
    Ok(bytes)
}

fn member_size(name: &str, member: &Member, bytes: &mut usize) -> Result<(), String> {
    add_size(bytes, string_size(name)?)?;
    if let Some(target) = &member.target {
        add_size(bytes, string_size(target)?)?;
    }
    add_size(bytes, traits_size(&member.traits)?)
}

fn traits_size(traits: &BTreeMap<String, serde_json::Value>) -> Result<usize, String> {
    let mut bytes = 0;
    for (name, value) in traits {
        add_size(&mut bytes, string_size(name)?)?;
        add_size(&mut bytes, json_size(value)?)?;
    }
    Ok(bytes)
}

fn json_size(root: &serde_json::Value) -> Result<usize, String> {
    let mut bytes = 0usize;
    let mut stack = vec![root];
    while let Some(value) = stack.pop() {
        json_node(value, &mut stack, &mut bytes)?;
    }
    Ok(bytes)
}

fn json_node<'a>(
    value: &'a serde_json::Value,
    stack: &mut Vec<&'a serde_json::Value>,
    bytes: &mut usize,
) -> Result<(), String> {
    add_size(bytes, 32)?;
    match value {
        serde_json::Value::String(text) => add_size(bytes, string_size(text)?)?,
        serde_json::Value::Array(items) => stack.extend(items),
        serde_json::Value::Object(items) => push_object(items, stack, bytes)?,
        _ => {}
    }
    Ok(())
}

fn push_object<'a>(
    items: &'a serde_json::Map<String, serde_json::Value>,
    stack: &mut Vec<&'a serde_json::Value>,
    bytes: &mut usize,
) -> Result<(), String> {
    for (key, value) in items {
        add_size(bytes, string_size(key)?)?;
        stack.push(value);
    }
    Ok(())
}

fn string_size(text: &str) -> Result<usize, String> {
    text.len().checked_add(24).ok_or_else(size_error)
}

fn add_size(total: &mut usize, size: usize) -> Result<(), String> {
    *total = total.checked_add(size).ok_or_else(size_error)?;
    if *total > MAX_MIXIN_BYTES {
        return Err(size_error());
    }
    Ok(())
}
