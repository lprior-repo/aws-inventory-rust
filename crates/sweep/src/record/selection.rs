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
    Owned(Vec<Record>),
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
            Source::Owned(records) => records.len(),
        };
        if count > MAX_PAGE_ITEMS {
            return Err(SweepError::Operation(
                "page exceeds the atomic observation bound",
            ));
        }
        Ok(count)
    }
}

fn records<'a>(
    source: Source<'a>,
) -> Result<impl Iterator<Item = Result<Record, SweepError>> + 'a + use<'a>, SweepError> {
    let (root, single, list, map, owned) = match source {
        Source::Empty => (None, None, None, None, None),
        Source::Root(record) => (Some(record), None, None, None, None),
        Source::Single(value) => (None, Some(value), None, None, None),
        Source::List(list) => (None, None, Some(list), None, None),
        Source::Map(map, kind) => (None, None, None, Some((map, kind)), None),
        Source::Owned(records) => (None, None, None, None, Some(records)),
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
    let owned = owned.into_iter().flatten().map(Ok);
    Ok(roots.chain(singles).chain(lists).chain(maps).chain(owned))
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
        ItemPlan::Nested {
            parents,
            items,
            owner,
        } => return Ok(Source::Owned(nested_records(record, parents, *items, *owner)?)),
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
fn nested_records(
    record: &Record,
    parents_path: &[inventory_core::record::FieldId],
    items_field: inventory_core::record::FieldId,
    owner_field: inventory_core::record::FieldId,
) -> Result<Vec<Record>, SweepError> {
    let parents = match path_value(record.root(), parents_path)? {
        None | Some(ValueRef::Null) => return Ok(Vec::new()),
        Some(ValueRef::List(parents)) => parents,
        Some(_) => {
            return Err(SweepError::Operation(
                "nested item parent path is not a list",
            ));
        }
    };
    parents.iter()?.try_fold(Vec::new(), |rows, parent| {
        nested_parent(rows, parent?, items_field, owner_field)
    })
}

fn nested_parent(
    rows: Vec<Record>,
    parent: ValueRef<'_>,
    items_field: inventory_core::record::FieldId,
    owner_field: inventory_core::record::FieldId,
) -> Result<Vec<Record>, SweepError> {
    let ValueRef::Map(parent) = parent else {
        return Err(SweepError::Operation("nested item parent is not a structure"));
    };
    let owner = match parent.get_field(owner_field)? {
        Some(ValueRef::Text(owner)) => owner,
        _ => return Err(SweepError::Operation("nested item owner is missing or invalid")),
    };
    inventory_core::inventory::AccountId::parse(owner)
        .map_err(|_| SweepError::Operation("nested item owner is invalid"))?;
    let items = match parent.get_field(items_field)? {
        None | Some(ValueRef::Null) => return Ok(rows),
        Some(ValueRef::List(items)) => items,
        Some(_) => return Err(SweepError::Operation("nested item member is not a list")),
    };
    items.iter()?.try_fold(rows, |mut rows, item| {
        if rows.len() >= MAX_PAGE_ITEMS {
            return Err(SweepError::Operation(
                "page exceeds the atomic observation bound",
            ));
        }
        rows.try_reserve(1)?;
        rows.push(nested_item(item?, owner_field, owner)?);
        Ok(rows)
    })
}

fn nested_item(
    item: ValueRef<'_>,
    owner_field: inventory_core::record::FieldId,
    owner: &str,
) -> Result<Record, SweepError> {
    let ValueRef::Map(item) = item else {
        return Err(SweepError::Operation("nested item is not a structure"));
    };
    let mut fields = item
        .iter()?
        .map(|entry| {
            let (key, value) = entry?;
            Ok((key.to_key(), value.to_value()?))
        })
        .collect::<Result<BTreeMap<_, _>, SweepError>>()?;
    let owner_key = Key::Field(owner_field);
    match fields.get(&owner_key) {
        Some(Value::Text(value)) if value == owner => {}
        Some(_) => return Err(SweepError::Operation("nested item owner conflicts with parent")),
        None => {
            fields.insert(owner_key, Value::Text(owner.to_owned()));
        }
    }
    Ok(Record::encode(&Value::Map(fields))?)
}

fn selected_source(
    kind: Option<SelectionKind>,
    value: ValueRef<'_>,
) -> Result<Source<'_>, SweepError> {
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
        SelectionKind::MapKeys | SelectionKind::MapValues | SelectionKind::MapEntries => {
            Ok(Source::Map(map, kind))
        }
        _ => Err(SweepError::Operation(
            "response value does not match its compiled item selector",
        )),
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
