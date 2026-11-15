use std::collections::BTreeMap;
use inventory_core::record::{Float, Time};
use crate::plans::model::{Model, ShapeRef, shape_type, target};
use std::ops::ControlFlow;

pub const MAX_DEPTH: usize = inventory_core::catalog::runtime::BindingValue::MAX_DEPTH;
pub const MAX_NODES: usize = inventory_core::catalog::runtime::BindingValue::MAX_NODES;
pub const MAX_COLLECTION: usize = inventory_core::catalog::runtime::BindingValue::MAX_COLLECTION;
pub const MAX_STRING: usize = inventory_core::catalog::runtime::BindingValue::MAX_STRING;
pub const MAX_TEXT_BYTES: usize = inventory_core::catalog::runtime::BindingValue::MAX_TEXT_BYTES;

/// Model-shaped request literals, compiled to numeric member IDs before execution.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum FixedValue {
    String(String),
    Integer(i64),
    Boolean(bool),
    Float(Float),
    Bytes(Vec<u8>),
    Timestamp(Time),
    /// Existing string-list shorthand; compiles to the same list representation.
    Strings(Vec<String>),
    List(Vec<Self>),
    Structure(BTreeMap<String, Self>),
    Map(BTreeMap<String, Self>),
}

impl FixedValue {
    pub fn list(values: impl IntoIterator<Item = Self>) -> Self {
        Self::List(values.into_iter().collect())
    }
    pub fn structure<K: Into<String>>(values: impl IntoIterator<Item = (K, Self)>) -> Result<Self, super::NameError> {
        object(values).map(Self::Structure)
    }
    pub fn map<K: Into<String>>(values: impl IntoIterator<Item = (K, Self)>) -> Result<Self, super::NameError> {
        object(values).map(Self::Map)
    }
}

fn object<K: Into<String>>(values: impl IntoIterator<Item = (K, FixedValue)>) -> Result<BTreeMap<String, FixedValue>, super::NameError> {
    values.into_iter().try_fold(BTreeMap::new(), |mut entries, (key, value)| {
        match entries.entry(key.into()) {
            std::collections::btree_map::Entry::Vacant(entry) => { entry.insert(value); }
            std::collections::btree_map::Entry::Occupied(entry) => {
                return Err(super::NameError { value: format!("duplicate fixed-input key: {}", entry.key()) });
            }
        }
        Ok(entries)
    })
}

pub(crate) fn matches(model: &Model<'_>, member: &serde_json::Value, shape: ShapeRef<'_>, value: &FixedValue) -> bool {
    let mut pending = vec![(member, shape, value, 0usize)];
    let mut nodes = 0usize;
    let mut bytes = 0usize;
    let outcome = (0..MAX_NODES).try_for_each(|_| {
        let Some((member, shape, value, depth)) = pending.pop() else { return ControlFlow::Break(true); };
        let result = (|| {
        nodes = nodes.checked_add(1).ok_or(())?;
        if depth > MAX_DEPTH { return Err(()); }
        let kind = shape_type(shape.value).ok_or(())?;
        match value {
            FixedValue::String(text) if matches!(kind, "string" | "enum") => {
                text_bound(text.len(), &mut bytes)?;
                if !string_matches(member, shape.value, text) { return Err(()); }
            }
            FixedValue::Integer(number) if integer_matches(kind, *number) => {
                if !range(member, *number) || !range(shape.value, *number)
                    || (kind == "intEnum" && !integer_enum_matches(shape.value, *number)) { return Err(()); }
            }
            FixedValue::Boolean(_) if kind == "boolean" => (),
            FixedValue::Float(number) if matches!(kind, "float" | "double") && number.to_f64().is_finite()
                && (kind == "double" || number.to_f64().abs() <= f64::from(f32::MAX)) => {
                if !float_range(member, number.to_f64()) || !float_range(shape.value, number.to_f64()) { return Err(()); }
            }
            FixedValue::Bytes(value) if kind == "blob" => {
                text_bound(value.len(), &mut bytes)?;
                if !length_matches(member, value.len()) || !length_matches(shape.value, value.len()) { return Err(()); }
            }
            FixedValue::Timestamp(_) if kind == "timestamp" => (),
            FixedValue::Strings(values) if kind == "list" => {
                collection(member, shape.value, values.len())?;
                let (edge, child) = child(model, shape.value, "member")?;
                nodes = nodes.checked_add(values.len()).ok_or(())?;
                if nodes > MAX_NODES || (depth >= MAX_DEPTH && !values.is_empty()) { return Err(()); }
                values.iter().try_for_each(|text| {
                    text_bound(text.len(), &mut bytes)?;
                    if !matches!(shape_type(child.value), Some("string" | "enum")) || !string_matches(edge, child.value, text) { return Err(()); }
                    Ok(())
                })?;
            }
            FixedValue::List(values) if kind == "list" => {
                collection(member, shape.value, values.len())?;
                let (edge, child) = child(model, shape.value, "member")?;
                values.iter().rev().for_each(|value| pending.push((edge, child, value, depth.saturating_add(1))));
            }
            FixedValue::Structure(values) if kind == "structure" => {
                collection(member, shape.value, values.len())?;
                let members = shape.value.get("members").and_then(serde_json::Value::as_object).ok_or(())?;
                if members.iter().any(|(name, member)| member.get("traits").and_then(|traits| traits.get("smithy.api#required")).is_some() && !values.contains_key(name)) { return Err(()); }
                values.iter().try_for_each(|(name, value)| {
                    text_bound(name.len(), &mut bytes)?;
                    let edge = members.get(name).ok_or(())?;
                    let child = target(edge).and_then(|id| model.shape(id)).ok_or(())?;
                    pending.push((edge, child, value, depth.saturating_add(1)));
                    Ok(())
                })?;
            }
            FixedValue::Map(values) if kind == "map" => {
                collection(member, shape.value, values.len())?;
                let (key_edge, key) = child(model, shape.value, "key")?;
                let (edge, child) = child(model, shape.value, "value")?;
                values.iter().try_for_each(|(name, value)| {
                    text_bound(name.len(), &mut bytes)?;
                    if !matches!(shape_type(key.value), Some("string" | "enum")) || !string_matches(key_edge, key.value, name) { return Err(()); }
                    pending.push((edge, child, value, depth.saturating_add(1)));
                    Ok(())
                })?;
            }
            _ => return Err(()),
        }
        if pending.len().saturating_add(nodes) > MAX_NODES { return Err(()); }
        Ok(())
        })();
        match result { Ok(()) => ControlFlow::Continue(()), Err(()) => ControlFlow::Break(false) }
    });
    !matches!(outcome, ControlFlow::Break(false)) && pending.is_empty()
}

pub(crate) fn bounded(value: &FixedValue) -> bool {
    let mut pending = vec![(value, 0usize)];
    let mut nodes = 0usize;
    let mut bytes = 0usize;
    let outcome = (0..MAX_NODES).try_for_each(|_| {
        let Some((value, depth)) = pending.pop() else { return ControlFlow::Break(true); };
        let result = (|| {
            nodes = nodes.checked_add(1).ok_or(())?;
            if depth > MAX_DEPTH || nodes > MAX_NODES { return Err(()); }
            match value {
                FixedValue::String(text) => text_bound(text.len(), &mut bytes)?,
                FixedValue::Bytes(value) => text_bound(value.len(), &mut bytes)?,
                FixedValue::Float(value) if !value.to_f64().is_finite() => return Err(()),
                FixedValue::Integer(_) | FixedValue::Boolean(_) | FixedValue::Float(_) | FixedValue::Timestamp(_) => (),
                FixedValue::Strings(values) => {
                    if values.len() > MAX_COLLECTION || (depth >= MAX_DEPTH && !values.is_empty()) { return Err(()); }
                    nodes = nodes.checked_add(values.len()).ok_or(())?;
                    values.iter().try_for_each(|text| text_bound(text.len(), &mut bytes))?;
                }
                FixedValue::List(values) => {
                    if values.len() > MAX_COLLECTION { return Err(()); }
                    values.iter().rev().for_each(|value| pending.push((value, depth.saturating_add(1))));
                }
                FixedValue::Structure(values) | FixedValue::Map(values) => {
                    if values.len() > MAX_COLLECTION { return Err(()); }
                    values.iter().try_for_each(|(key, value)| {
                        text_bound(key.len(), &mut bytes)?;
                        pending.push((value, depth.saturating_add(1)));
                        Ok(())
                    })?;
                }
            }
            if nodes.saturating_add(pending.len()) > MAX_NODES { return Err(()); }
            Ok(())
        })();
        match result { Ok(()) => ControlFlow::Continue(()), Err(()) => ControlFlow::Break(false) }
    });
    !matches!(outcome, ControlFlow::Break(false)) && pending.is_empty()
}

fn child<'a>(model: &Model<'a>, shape: &'a serde_json::Value, key: &str) -> Result<(&'a serde_json::Value, ShapeRef<'a>), ()> {
    let edge = shape.get(key).ok_or(())?;
    Ok((edge, target(edge).and_then(|id| model.shape(id)).ok_or(())?))
}
fn text_bound(length: usize, bytes: &mut usize) -> Result<(), ()> {
    *bytes = bytes.checked_add(length).ok_or(())?;
    if length > MAX_STRING || *bytes > MAX_TEXT_BYTES { Err(()) } else { Ok(()) }
}
fn collection(member: &serde_json::Value, shape: &serde_json::Value, length: usize) -> Result<(), ()> {
    if length > MAX_COLLECTION || !length_matches(member, length) || !length_matches(shape, length) { Err(()) } else { Ok(()) }
}
fn length_matches(shape: &serde_json::Value, length: usize) -> bool {
    i64::try_from(length).is_ok_and(|length| bounds(shape, "smithy.api#length", length))
}
pub(crate) fn range(shape: &serde_json::Value, number: i64) -> bool { bounds(shape, "smithy.api#range", number) }
fn bounds(shape: &serde_json::Value, trait_name: &str, value: i64) -> bool {
    let range = shape.get("traits").and_then(|traits| traits.get(trait_name));
    let bound = |name| range.and_then(|range| range.get(name)).and_then(serde_json::Value::as_i64);
    bound("min").is_none_or(|min| value >= min) && bound("max").is_none_or(|max| value <= max)
}
fn float_range(shape: &serde_json::Value, value: f64) -> bool {
    let range = shape.get("traits").and_then(|traits| traits.get("smithy.api#range"));
    let bound = |name| range.and_then(|range| range.get(name)).and_then(serde_json::Value::as_f64);
    bound("min").is_none_or(|min| value >= min) && bound("max").is_none_or(|max| value <= max)
}
fn integer_matches(kind: &str, value: i64) -> bool {
    match kind {
        "byte" => i8::try_from(value).is_ok(),
        "short" => i16::try_from(value).is_ok(),
        "integer" | "intEnum" => i32::try_from(value).is_ok(),
        "long" => true,
        _ => false,
    }
}
fn integer_enum_matches(shape: &serde_json::Value, value: i64) -> bool {
    shape.get("members").and_then(serde_json::Value::as_object).is_some_and(|members| members.values().any(|member| {
        member.get("traits").and_then(|traits| traits.get("smithy.api#enumValue")).and_then(serde_json::Value::as_i64) == Some(value)
    }))
}
fn string_matches(member: &serde_json::Value, shape: &serde_json::Value, text: &str) -> bool {
    let length = text.chars().count();
    if !length_matches(member, length) || !length_matches(shape, length) { return false; }
    if !string_traits(member, text) || !string_traits(shape, text) { return false; }
    if shape_type(shape) != Some("enum") { return true; }
    shape.get("members").and_then(serde_json::Value::as_object).is_some_and(|members| members.values().any(|member| {
        member.get("traits").and_then(|traits| traits.get("smithy.api#enumValue")).and_then(serde_json::Value::as_str) == Some(text)
    }))
}
fn string_traits(shape: &serde_json::Value, text: &str) -> bool {
    let traits = shape.get("traits");
    let legacy_enum = traits.and_then(|traits| traits.get("smithy.api#enum"));
    if legacy_enum.is_some_and(|values| !values.as_array().is_some_and(|values| values.iter().any(|value| value.get("value").and_then(serde_json::Value::as_str) == Some(text)))) { return false; }
    let Some(pattern) = traits.and_then(|traits| traits.get("smithy.api#pattern")) else { return true; };
    let Some(pattern) = pattern.as_str().filter(|pattern| pattern.len() <= MAX_STRING) else { return false; };
    regex::RegexBuilder::new(pattern).size_limit(MAX_TEXT_BYTES).dfa_size_limit(MAX_TEXT_BYTES)
        .build().is_ok_and(|pattern| pattern.is_match(text))
}
