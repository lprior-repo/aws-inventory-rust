use super::{MAX_PAGE_ITEMS, SweepError, item_record, path_value};
use inventory_core::{
    catalog::runtime::{ItemPlan, SelectionKind},
    record::{Key, KeyRef, ListRef, MapRef, Record, Value, ValueRef},
};
use std::collections::BTreeMap;

enum Source<'a> {
    Empty,
    Root(&'a Record),
    Single(ValueRef<'a>),
    List(ListRef<'a>),
    Map(MapRef<'a>, SelectionKind),
}

pub(super) fn items<'a>(
    record: &'a Record,
    plan: &ItemPlan,
) -> Result<
    (
        usize,
        impl Iterator<Item = Result<Record, SweepError>> + 'a + use<'a>,
    ),
    SweepError,
> {
    let source = source(record, plan)?;
    let count = source.count()?;
    Ok((count, records(source)?))
}

impl Source<'_> {
    fn count(&self) -> Result<usize, SweepError> {
        let count = match self {
            Source::Empty => 0,
            Source::Root(_) | Source::Single(_) => 1,
            Source::List(list) => list.count()?,
            Source::Map(map, _) => map.count()?,
        };
        if count > MAX_PAGE_ITEMS {
            return Err(SweepError::Operation("page exceeds the atomic observation bound"));
        }
        Ok(count)
    }
}

fn records<'a>(
    source: Source<'a>,
) -> Result<impl Iterator<Item = Result<Record, SweepError>> + 'a + use<'a>, SweepError> {
    let (root, single, list, map) = match source {
        Source::Empty => (None, None, None, None),
        Source::Root(record) => (Some(record), None, None, None),
        Source::Single(value) => (None, Some(value), None, None),
        Source::List(list) => (None, None, Some(list), None),
        Source::Map(map, kind) => (None, None, None, Some((map, kind))),
    };
    let roots = root.into_iter().map(|record| Ok(record.clone()));
    let singles = single.into_iter().map(item_record);
    let lists = list
        .map(|list| list.iter())
        .transpose()?
        .into_iter()
        .flatten()
        .map(|value| item_record(value?));
    let maps = map_records(map)?;
    Ok(roots.chain(singles).chain(lists).chain(maps))
}

fn map_records<'a>(
    map: Option<(MapRef<'a>, SelectionKind)>,
) -> Result<impl Iterator<Item = Result<Record, SweepError>> + 'a + use<'a>, SweepError> {
    Ok(map
        .map(|(map, kind)| {
            map.iter().map(|entries| {
                entries.map(move |entry| {
                    let (key, value) = entry?;
                    map_record(kind, key, value)
                })
            })
        })
        .transpose()?
        .into_iter()
        .flatten())
}

fn source<'a>(record: &'a Record, plan: &ItemPlan) -> Result<Source<'a>, SweepError> {
    let (path, kind) = match plan {
        ItemPlan::Root => return Ok(Source::Root(record)),
        ItemPlan::Path(path) => (path.as_ref(), None),
        ItemPlan::Select { path, kind } => (path.as_ref(), Some(*kind)),
    };
    let value = if kind.is_some() && path.is_empty() {
        Some(ValueRef::Map(record.root()))
    } else {
        path_value(record.root(), path)?
    };
    let Some(value) = value else {
        return Ok(Source::Empty);
    };
    if matches!(value, ValueRef::Null) {
        return Ok(Source::Empty);
    }
    selected_source(kind, value)
}

fn selected_source(kind: Option<SelectionKind>, value: ValueRef<'_>) -> Result<Source<'_>, SweepError> {
    match (kind, value) {
        (None | Some(SelectionKind::Structure), value @ ValueRef::Map(_)) => {
            Ok(Source::Single(value))
        }
        (None | Some(SelectionKind::List), ValueRef::List(list)) => Ok(Source::List(list)),
        (Some(SelectionKind::Scalar), value)
            if !matches!(value, ValueRef::Map(_) | ValueRef::List(_)) =>
        {
            Ok(Source::Single(value))
        }
        (Some(kind), ValueRef::Map(map)) => map_source(map, kind),
        (None, _) => Err(SweepError::Operation(
            "compiled enumeration item path is neither a list nor a structure",
        )),
        _ => Err(SweepError::Operation(
            "response value does not match its compiled item selector",
        )),
    }
}

fn map_source(map: MapRef<'_>, kind: SelectionKind) -> Result<Source<'_>, SweepError> {
    match kind {
        SelectionKind::MapKeys | SelectionKind::MapValues | SelectionKind::MapEntries => Ok(Source::Map(map, kind)),
        _ => Err(SweepError::Operation("response value does not match its compiled item selector")),
    }
}

fn map_record(
    kind: SelectionKind,
    key: KeyRef<'_>,
    value: ValueRef<'_>,
) -> Result<Record, SweepError> {
    let KeyRef::Name(key) = key else {
        return Err(SweepError::Operation("modeled map has a non-text key"));
    };
    match kind {
        SelectionKind::MapKeys => Ok(inventory_core::record::scalar_record(ValueRef::Text(key))?),
        SelectionKind::MapValues => item_record(value),
        SelectionKind::MapEntries => Ok(Record::encode(&Value::Map(BTreeMap::from([
            (Key::Name("$key".to_owned()), Value::Text(key.to_owned())),
            (Key::Name("$value".to_owned()), value.to_value()?),
        ])))?),
        _ => Err(SweepError::Operation("invalid map selector")),
    }
}
