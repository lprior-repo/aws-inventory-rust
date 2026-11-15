use super::ContextError;
use crate::record::{FieldId, Key, MapRef, Record, Value, ValueRef, scalar_item};
use std::collections::BTreeMap;

pub(super) fn value<'a>(record: &'a Record, path: &[FieldId]) -> Result<Option<ValueRef<'a>>, ContextError> {
    let root = match record.root().get_name("$value")? {
        Some(value) if path.is_empty() => return Ok(Some(value)),
        Some(ValueRef::Map(root)) => root,
        Some(_) => return Ok(None),
        None => record.root(),
    };
    if path.is_empty() {
        return scalar_item(root).map_err(ContextError::from);
    }
    value_at(root, path)
}

fn value_at<'a>(root: MapRef<'a>, path: &[FieldId]) -> Result<Option<ValueRef<'a>>, ContextError> {
    path.iter().enumerate().try_fold(Some(ValueRef::Map(root)), |parent, (position, field)| {
        let Some(ValueRef::Map(map)) = parent else { return Ok(None) };
        let child = map.get_field(*field)?;
        if position.saturating_add(1) < path.len() && !matches!(child, Some(ValueRef::Map(_))) {
            return Ok(None);
        }
        Ok(child)
    })
}

pub(super) fn project(record: &Record, paths: &[Box<[FieldId]>]) -> Result<Record, ContextError> {
    if paths.len() > 256 {
        return Err(ContextError::Bounds);
    }
    let fields = paths.iter().try_fold(BTreeMap::new(), |mut fields, path| {
        let Some(selected) = value(record, path)? else { return Ok(fields) };
        if matches!(selected, ValueRef::Map(_) | ValueRef::List(_)) {
            return Err(ContextError::Invalid);
        }
        if path.is_empty() {
            fields.insert(Key::Name("$".to_owned()), selected.to_value()?);
        } else {
            insert(&mut fields, path, selected.to_value()?)?;
        }
        Ok::<_, ContextError>(fields)
    })?;
    Record::encode(&Value::Map(fields)).map_err(ContextError::from)
}

fn insert(fields: &mut BTreeMap<Key, Value>, path: &[FieldId], value: Value) -> Result<(), ContextError> {
    if path.is_empty() || path.len() > 8 {
        return Err(ContextError::Bounds);
    }
    let Some((leaf, ancestors)) = path.split_last() else { return Err(ContextError::Invalid) };
    let parent = ancestors.iter().try_fold(fields, |map, field| {
        match map.entry(Key::Field(*field)).or_insert_with(|| Value::Map(BTreeMap::new())) {
            Value::Map(child) => Ok(child),
            _ => Err(ContextError::Invalid),
        }
    })?;
    match parent.entry(Key::Field(*leaf)) {
        std::collections::btree_map::Entry::Vacant(entry) => { entry.insert(value); Ok(()) }
        std::collections::btree_map::Entry::Occupied(entry) if entry.get() == &value => Ok(()),
        _ => Err(ContextError::Invalid),
    }
}
