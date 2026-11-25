use crate::plans::model::{Model, ShapeRef, shape_type, target};
use inventory_core::record::{Float, Time};
use std::collections::BTreeMap;
use std::ops::ControlFlow;

pub const MAX_DEPTH: usize = inventory_core::catalog::runtime::BindingValue::MAX_DEPTH;
pub const MAX_NODES: usize = inventory_core::catalog::runtime::BindingValue::MAX_NODES;
pub const MAX_COLLECTION: usize = inventory_core::catalog::runtime::BindingValue::MAX_COLLECTION;
pub const MAX_STRING: usize = inventory_core::catalog::runtime::BindingValue::MAX_STRING;
pub const MAX_TEXT_BYTES: usize = inventory_core::catalog::runtime::BindingValue::MAX_TEXT_BYTES;

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum FixedValue {
    String(String),
    Integer(i64),
    Boolean(bool),
    Float(Float),
    Bytes(Vec<u8>),
    Timestamp(Time),
    Strings(Vec<String>),
    List(Vec<Self>),
    Structure(BTreeMap<String, Self>),
    Map(BTreeMap<String, Self>),
}

impl FixedValue {
    pub fn list(values: impl IntoIterator<Item = Self>) -> Self {
        Self::List(values.into_iter().collect())
    }
    pub fn structure<K: Into<String>>(
        values: impl IntoIterator<Item = (K, Self)>,
    ) -> Result<Self, super::NameError> {
        object(values).map(Self::Structure)
    }
    pub fn map<K: Into<String>>(
        values: impl IntoIterator<Item = (K, Self)>,
    ) -> Result<Self, super::NameError> {
        object(values).map(Self::Map)
    }
}

fn object<K: Into<String>>(
    values: impl IntoIterator<Item = (K, FixedValue)>,
) -> Result<BTreeMap<String, FixedValue>, super::NameError> {
    values
        .into_iter()
        .try_fold(BTreeMap::new(), |mut entries, (key, value)| {
            match entries.entry(key.into()) {
                std::collections::btree_map::Entry::Vacant(entry) => {
                    entry.insert(value);
                }
                std::collections::btree_map::Entry::Occupied(entry) => {
                    return Err(super::NameError {
                        value: format!("duplicate fixed-input key: {}", entry.key()),
                    });
                }
            }
            Ok(entries)
        })
}

type MatchEntry<'a> = (&'a serde_json::Value, ShapeRef<'a>, &'a FixedValue, usize);

struct MatchWalk<'a> {
    pending: Vec<MatchEntry<'a>>,
    nodes: usize,
    bytes: usize,
}

pub(crate) fn matches(
    model: &Model<'_>,
    member: &serde_json::Value,
    shape: ShapeRef<'_>,
    value: &FixedValue,
) -> bool {
    let mut walk = MatchWalk {
        pending: vec![(member, shape, value, 0usize)],
        nodes: 0,
        bytes: 0,
    };
    let outcome = (0..MAX_NODES).try_for_each(|_| {
        let Some(entry) = walk.pending.pop() else {
            return ControlFlow::Break(true);
        };
        match walk.visit(model, entry) {
            Ok(()) => ControlFlow::Continue(()),
            Err(()) => ControlFlow::Break(false),
        }
    });
    !matches!(outcome, ControlFlow::Break(false)) && walk.pending.is_empty()
}

impl<'a> MatchWalk<'a> {
    fn visit(
        &mut self,
        model: &Model<'a>,
        (member, shape, value, depth): MatchEntry<'a>,
    ) -> Result<(), ()> {
        self.nodes = self.nodes.checked_add(1).ok_or(())?;
        if depth > MAX_DEPTH {
            return Err(());
        }
        let kind = shape_type(shape.value).ok_or(())?;
        match value {
            FixedValue::Strings(values) if kind == "list" => {
                self.strings(model, member, shape.value, values, depth)?;
            }
            FixedValue::List(values) if kind == "list" => {
                collection(member, shape.value, values.len())?;
                let (edge, child) = child(model, shape.value, "member")?;
                values.iter().rev().for_each(|value| {
                    self.pending
                        .push((edge, child, value, depth.saturating_add(1)))
                });
            }
            FixedValue::Structure(values) if kind == "structure" => {
                self.structure(model, member, shape.value, values, depth)?;
            }
            FixedValue::Map(values) if kind == "map" => {
                self.map(model, member, shape.value, values, depth)?;
            }
            FixedValue::Strings(_)
            | FixedValue::List(_)
            | FixedValue::Structure(_)
            | FixedValue::Map(_) => return Err(()),
            scalar => self.scalar(member, shape.value, kind, scalar)?,
        }
        if self.pending.len().saturating_add(self.nodes) > MAX_NODES {
            return Err(());
        }
        Ok(())
    }

    fn scalar(
        &mut self,
        member: &serde_json::Value,
        shape: &serde_json::Value,
        kind: &str,
        value: &FixedValue,
    ) -> Result<(), ()> {
        match value {
            FixedValue::String(text) if matches!(kind, "string" | "enum") => {
                text_bound(text.len(), &mut self.bytes)?;
                if !string_matches(member, shape, text) {
                    return Err(());
                }
            }
            FixedValue::Integer(number) if integer_matches(kind, *number) => {
                if !range(member, *number)
                    || !range(shape, *number)
                    || (kind == "intEnum" && !integer_enum_matches(shape, *number))
                {
                    return Err(());
                }
            }
            FixedValue::Boolean(_) if kind == "boolean" => (),
            FixedValue::Float(number)
                if matches!(kind, "float" | "double")
                    && number.to_f64().is_finite()
                    && (kind == "double" || number.to_f64().abs() <= f64::from(f32::MAX)) =>
            {
                if !float_range(member, number.to_f64()) || !float_range(shape, number.to_f64()) {
                    return Err(());
                }
            }
            FixedValue::Bytes(value) if kind == "blob" => {
                text_bound(value.len(), &mut self.bytes)?;
                if !length_matches(member, value.len()) || !length_matches(shape, value.len()) {
                    return Err(());
                }
            }
            FixedValue::Timestamp(_) if kind == "timestamp" => (),
            _ => return Err(()),
        }
        Ok(())
    }

    fn strings(
        &mut self,
        model: &Model<'a>,
        member: &'a serde_json::Value,
        shape: &'a serde_json::Value,
        values: &[String],
        depth: usize,
    ) -> Result<(), ()> {
        collection(member, shape, values.len())?;
        let (edge, child) = child(model, shape, "member")?;
        self.nodes = self.nodes.checked_add(values.len()).ok_or(())?;
        if self.nodes > MAX_NODES || (depth >= MAX_DEPTH && !values.is_empty()) {
            return Err(());
        }
        values.iter().try_for_each(|text| {
            text_bound(text.len(), &mut self.bytes)?;
            if !matches!(shape_type(child.value), Some("string" | "enum"))
                || !string_matches(edge, child.value, text)
            {
                return Err(());
            }
            Ok(())
        })
    }

    fn structure(
        &mut self,
        model: &Model<'a>,
        member: &'a serde_json::Value,
        shape: &'a serde_json::Value,
        values: &'a BTreeMap<String, FixedValue>,
        depth: usize,
    ) -> Result<(), ()> {
        collection(member, shape, values.len())?;
        let members = shape
            .get("members")
            .and_then(serde_json::Value::as_object)
            .ok_or(())?;
        if members.iter().any(|(name, member)| {
            member
                .get("traits")
                .and_then(|traits| traits.get("smithy.api#required"))
                .is_some()
                && !values.contains_key(name)
        }) {
            return Err(());
        }
        values.iter().try_for_each(|(name, value)| {
            text_bound(name.len(), &mut self.bytes)?;
            let edge = members.get(name).ok_or(())?;
            let child = target(edge).and_then(|id| model.shape(id)).ok_or(())?;
            self.pending
                .push((edge, child, value, depth.saturating_add(1)));
            Ok(())
        })
    }

    fn map(
        &mut self,
        model: &Model<'a>,
        member: &'a serde_json::Value,
        shape: &'a serde_json::Value,
        values: &'a BTreeMap<String, FixedValue>,
        depth: usize,
    ) -> Result<(), ()> {
        collection(member, shape, values.len())?;
        let (key_edge, key) = child(model, shape, "key")?;
        let (edge, child) = child(model, shape, "value")?;
        values.iter().try_for_each(|(name, value)| {
            text_bound(name.len(), &mut self.bytes)?;
            if !matches!(shape_type(key.value), Some("string" | "enum"))
                || !string_matches(key_edge, key.value, name)
            {
                return Err(());
            }
            self.pending
                .push((edge, child, value, depth.saturating_add(1)));
            Ok(())
        })
    }
}

struct BoundWalk<'a> {
    pending: Vec<(&'a FixedValue, usize)>,
    nodes: usize,
    bytes: usize,
}

pub(crate) fn bounded(value: &FixedValue) -> bool {
    let mut walk = BoundWalk {
        pending: vec![(value, 0usize)],
        nodes: 0,
        bytes: 0,
    };
    let outcome = (0..MAX_NODES).try_for_each(|_| {
        let Some((value, depth)) = walk.pending.pop() else {
            return ControlFlow::Break(true);
        };
        match walk.visit(value, depth) {
            Ok(()) => ControlFlow::Continue(()),
            Err(()) => ControlFlow::Break(false),
        }
    });
    !matches!(outcome, ControlFlow::Break(false)) && walk.pending.is_empty()
}

impl<'a> BoundWalk<'a> {
    fn visit(&mut self, value: &'a FixedValue, depth: usize) -> Result<(), ()> {
        self.nodes = self.nodes.checked_add(1).ok_or(())?;
        if depth > MAX_DEPTH || self.nodes > MAX_NODES {
            return Err(());
        }
        match value {
            FixedValue::String(text) => text_bound(text.len(), &mut self.bytes)?,
            FixedValue::Bytes(value) => text_bound(value.len(), &mut self.bytes)?,
            FixedValue::Float(value) if !value.to_f64().is_finite() => return Err(()),
            FixedValue::Integer(_)
            | FixedValue::Boolean(_)
            | FixedValue::Float(_)
            | FixedValue::Timestamp(_) => (),
            FixedValue::Strings(values) => {
                if values.len() > MAX_COLLECTION || (depth >= MAX_DEPTH && !values.is_empty()) {
                    return Err(());
                }
                self.nodes = self.nodes.checked_add(values.len()).ok_or(())?;
                values
                    .iter()
                    .try_for_each(|text| text_bound(text.len(), &mut self.bytes))?;
            }
            FixedValue::List(values) => {
                if values.len() > MAX_COLLECTION {
                    return Err(());
                }
                values
                    .iter()
                    .rev()
                    .for_each(|value| self.pending.push((value, depth.saturating_add(1))));
            }
            FixedValue::Structure(values) | FixedValue::Map(values) => {
                if values.len() > MAX_COLLECTION {
                    return Err(());
                }
                values.iter().try_for_each(|(key, value)| {
                    text_bound(key.len(), &mut self.bytes)?;
                    self.pending.push((value, depth.saturating_add(1)));
                    Ok(())
                })?;
            }
        }
        if self.nodes.saturating_add(self.pending.len()) > MAX_NODES {
            return Err(());
        }
        Ok(())
    }
}

fn child<'a>(
    model: &Model<'a>,
    shape: &'a serde_json::Value,
    key: &str,
) -> Result<(&'a serde_json::Value, ShapeRef<'a>), ()> {
    let edge = shape.get(key).ok_or(())?;
    Ok((edge, target(edge).and_then(|id| model.shape(id)).ok_or(())?))
}
fn text_bound(length: usize, bytes: &mut usize) -> Result<(), ()> {
    *bytes = bytes.checked_add(length).ok_or(())?;
    if length > MAX_STRING || *bytes > MAX_TEXT_BYTES {
        Err(())
    } else {
        Ok(())
    }
}
fn collection(
    member: &serde_json::Value,
    shape: &serde_json::Value,
    length: usize,
) -> Result<(), ()> {
    if length > MAX_COLLECTION || !length_matches(member, length) || !length_matches(shape, length)
    {
        Err(())
    } else {
        Ok(())
    }
}
fn length_matches(shape: &serde_json::Value, length: usize) -> bool {
    i64::try_from(length).is_ok_and(|length| bounds(shape, "smithy.api#length", length))
}
pub(crate) fn range(shape: &serde_json::Value, number: i64) -> bool {
    bounds(shape, "smithy.api#range", number)
}
fn bounds(shape: &serde_json::Value, trait_name: &str, value: i64) -> bool {
    let range = shape
        .get("traits")
        .and_then(|traits| traits.get(trait_name));
    let bound = |name| {
        range
            .and_then(|range| range.get(name))
            .and_then(serde_json::Value::as_i64)
    };
    bound("min").is_none_or(|min| value >= min) && bound("max").is_none_or(|max| value <= max)
}
fn float_range(shape: &serde_json::Value, value: f64) -> bool {
    let range = shape
        .get("traits")
        .and_then(|traits| traits.get("smithy.api#range"));
    let bound = |name| {
        range
            .and_then(|range| range.get(name))
            .and_then(serde_json::Value::as_f64)
    };
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
    shape
        .get("members")
        .and_then(serde_json::Value::as_object)
        .is_some_and(|members| {
            members.values().any(|member| {
                member
                    .get("traits")
                    .and_then(|traits| traits.get("smithy.api#enumValue"))
                    .and_then(serde_json::Value::as_i64)
                    == Some(value)
            })
        })
}
fn string_matches(member: &serde_json::Value, shape: &serde_json::Value, text: &str) -> bool {
    let length = text.chars().count();
    if !length_matches(member, length) || !length_matches(shape, length) {
        return false;
    }
    if !string_traits(member, text) || !string_traits(shape, text) {
        return false;
    }
    if shape_type(shape) != Some("enum") {
        return true;
    }
    shape
        .get("members")
        .and_then(serde_json::Value::as_object)
        .is_some_and(|members| {
            members.values().any(|member| {
                member
                    .get("traits")
                    .and_then(|traits| traits.get("smithy.api#enumValue"))
                    .and_then(serde_json::Value::as_str)
                    == Some(text)
            })
        })
}
fn string_traits(shape: &serde_json::Value, text: &str) -> bool {
    let traits = shape.get("traits");
    let legacy_enum = traits.and_then(|traits| traits.get("smithy.api#enum"));
    if legacy_enum.is_some_and(|values| {
        !values.as_array().is_some_and(|values| {
            values
                .iter()
                .any(|value| value.get("value").and_then(serde_json::Value::as_str) == Some(text))
        })
    }) {
        return false;
    }
    let Some(pattern) = traits.and_then(|traits| traits.get("smithy.api#pattern")) else {
        return true;
    };
    let Some(pattern) = pattern
        .as_str()
        .filter(|pattern| pattern.len() <= MAX_STRING)
    else {
        return false;
    };
    regex::RegexBuilder::new(pattern)
        .size_limit(MAX_TEXT_BYTES)
        .dfa_size_limit(MAX_TEXT_BYTES)
        .build()
        .is_ok_and(|pattern| pattern.is_match(text))
}
