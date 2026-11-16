use super::super::StoreError;
use inventory_core::record::{FieldId, Key, Record, Value, ValueRef};
use std::collections::{BTreeMap, BTreeSet};

pub(super) type Path = Vec<Key>;
pub(super) type Fields = BTreeMap<Path, Value>;
const MAX_FIELDS: usize = 65_536;

pub(super) fn fields(record: &Record) -> Result<Fields, StoreError> {
    let pending = vec![(Vec::new(), ValueRef::Map(record.root()))];
    // Each step consumes one node; the terminal step exits without scanning the cap.
    let result = (0..MAX_FIELDS).try_fold((pending, BTreeMap::new()), |(mut pending, mut result), _| {
        let Some((path, value)) = pending.pop() else {
            return std::ops::ControlFlow::Break(Ok(result));
        };
        let step = visit(&mut pending, &mut result, path, value);
        match step {
            Ok(()) => std::ops::ControlFlow::Continue((pending, result)),
            Err(error) => std::ops::ControlFlow::Break(Err(error)),
        }
    });
    let mut result = match result {
        std::ops::ControlFlow::Break(result) => result?,
        std::ops::ControlFlow::Continue((pending, result)) if pending.is_empty() => result,
        std::ops::ControlFlow::Continue(_) => return Err(StoreError::Invalid("facet field count exceeds bounds")),
    };
    result.remove(&Vec::new());
    Ok(result)
}

fn visit<'a>(
    pending: &mut Vec<(Path, ValueRef<'a>)>, result: &mut Fields, path: Path, value: ValueRef<'a>,
) -> Result<(), StoreError> {
    let ValueRef::Map(map) = value else {
        result.insert(path, value.to_value()?);
        return Ok(());
    };
    result.insert(path.clone(), Value::Map(BTreeMap::new()));
    map.iter()?.try_for_each(|entry| -> Result<(), StoreError> {
        let (key, child) = entry?;
        if pending.len().saturating_add(result.len()) >= MAX_FIELDS {
            return Err(StoreError::Invalid("facet field count exceeds bounds"));
        }
        let mut child_path = path.clone();
        child_path.push(key.to_key());
        pending.push((child_path, child));
        Ok(())
    })
}

pub(super) fn owned(previous: Option<&Record>, current: &Fields) -> Result<Record, StoreError> {
    let mut paths = match previous {
        Some(record) => paths(record)?,
        None => BTreeSet::new(),
    };
    paths.extend(current.keys().cloned());
    if paths.len() > MAX_FIELDS { return Err(StoreError::Invalid("facet ownership exceeds bounds")); }
    // Containers organize descendants; only terminal paths own values. Otherwise
    // omitting one source's map would erase another source's sibling leaves.
    let next = paths.iter().skip(1).map(Some).chain(std::iter::once(None));
    encode_paths(paths.iter().zip(next).filter_map(|(path, next)| {
        (!next.is_some_and(|next| next.starts_with(path))).then_some(path.as_slice())
    }))
}

pub(super) fn encode_paths<'a>(paths: impl IntoIterator<Item = &'a [Key]>) -> Result<Record, StoreError> {
    let mut set = BTreeSet::new();
    for path in paths {
        if path.len() > inventory_core::record::MAX_DEPTH {
            return Err(StoreError::Invalid("facet path depth exceeds bounds"));
        }
        set.insert(path.to_vec());
    }
    if set.len() > MAX_FIELDS {
        return Err(StoreError::Invalid("facet ownership exceeds bounds"));
    }
    let values = set.into_iter().map(|path| Value::List(path.iter().map(|key| match key {
        Key::Field(field) => Value::Int(i64::from(field.value())),
        Key::Name(name) => Value::Text(name.clone()),
    }).collect())).collect::<Vec<_>>();
    Ok(Record::encode(&Value::Map(BTreeMap::from([(Key::Field(FieldId::new(1)?), Value::List(values))])))?)
}

pub(super) fn paths(record: &Record) -> Result<BTreeSet<Path>, StoreError> {
    decode_paths(record, false)
}

pub(super) fn dependency_paths(record: &Record) -> Result<BTreeSet<Path>, StoreError> {
    decode_paths(record, true)
}

fn decode_paths(record: &Record, allow_root: bool) -> Result<BTreeSet<Path>, StoreError> {
    let Some(ValueRef::List(list)) = record.root().get_field(FieldId::new(1)?)? else {
        return Err(StoreError::Invalid("facet path list is absent"));
    };
    list.iter()?.map(|entry| {
        let ValueRef::List(path) = entry? else { return Err(StoreError::Invalid("facet path is not a list")); };
        let path = path.iter()?.map(|key| match key? {
            ValueRef::Int(value) => Ok(Key::Field(FieldId::new(u32::try_from(value)
                .map_err(|_| StoreError::Invalid("facet field identifier exceeds bounds"))?)?)),
            ValueRef::Text(value) => Ok(Key::Name(value.to_owned())),
            _ => Err(StoreError::Invalid("facet path component is invalid")),
        }).collect::<Result<Vec<_>, StoreError>>()?;
        if (!allow_root && path.is_empty()) || path.len() > inventory_core::record::MAX_DEPTH {
            return Err(StoreError::Invalid("facet path depth exceeds bounds"));
        }
        Ok(path)
    }).collect()
}

pub(super) fn overlay(
    merged: &mut Fields,
    owners: &mut BTreeMap<Path, usize>,
    owned: &Record,
    current: Fields,
    facet: usize,
) -> Result<(), StoreError> {
    paths(owned)?.iter().filter(|path| !current.contains_key(*path)).for_each(|path| {
        merged.retain(|candidate, _| !candidate.starts_with(path));
        owners.retain(|candidate, _| !candidate.starts_with(path));
    });
    current.into_iter().for_each(|(path, value)| {
        if !matches!(value, Value::Map(_)) {
            merged.retain(|candidate, _| !candidate.starts_with(&path));
            owners.retain(|candidate, _| !candidate.starts_with(&path));
        }
        merged.insert(path.clone(), value);
        owners.insert(path, facet);
    });
    Ok(())
}

pub(super) fn record(fields: Fields) -> Result<Record, StoreError> {
    let mut entries = fields.into_iter().collect::<Vec<_>>();
    entries.sort_unstable_by(|(left, _), (right, _)| right.len().cmp(&left.len()).then(left.cmp(right)));
    let mut nodes = BTreeMap::<Path, Value>::new();
    entries.into_iter().try_for_each(|(mut path, value)| -> Result<(), StoreError> {
        let value = match nodes.remove(&path) { Some(children) => children, None => value };
        let key = path.pop().ok_or(StoreError::Invalid("facet has an empty field path"))?;
        match nodes.entry(path).or_insert_with(|| Value::Map(BTreeMap::new())) {
            Value::Map(parent) => { parent.insert(key, value); }
            _ => return Err(StoreError::Invalid("facet has conflicting container fields")),
        }
        Ok(())
    })?;
    let root = match nodes.remove(&Vec::new()) { Some(value) => value, None => Value::Map(BTreeMap::new()) };
    Ok(Record::encode(&root)?)
}
