use super::{StoreError, merge};
use inventory_core::record::{Exclusions, FieldId, Key, MapKeyExclusion, Record, Value, ValueRef};
use std::collections::{BTreeMap, BTreeSet};

pub(in crate::inventory) fn encode(exclusions: &Exclusions) -> Result<Record, StoreError> {
    let paths = exclusions
        .paths()
        .iter()
        .map(|path| {
            Value::List(
                path.iter()
                    .map(|field| Value::Int(i64::from(field.value())))
                    .collect(),
            )
        })
        .collect();
    let keys = exclusions
        .map_keys()
        .iter()
        .map(|entry| {
            Value::List(vec![
                Value::List(
                    entry
                        .path()
                        .iter()
                        .map(|field| Value::Int(i64::from(field.value())))
                        .collect(),
                ),
                Value::Text(entry.key().to_owned()),
            ])
        })
        .collect();
    Ok(Record::encode(&Value::Map(BTreeMap::from([
        (Key::Field(FieldId::new(1)?), Value::List(paths)),
        (Key::Field(FieldId::new(2)?), Value::List(keys)),
    ])))?)
}

pub(in crate::inventory) fn append(
    record: &Record,
    paths: &mut Vec<Vec<FieldId>>,
    keys: &mut BTreeSet<MapKeyExclusion>,
) -> Result<(), StoreError> {
    for path in merge::paths(record)? {
        paths.push(
            path.into_iter()
                .map(|key| match key {
                    Key::Field(field) => Ok(field),
                    Key::Name(_) => Err(StoreError::Invalid(
                        "numeric facet exclusion contains a named key",
                    )),
                })
                .collect::<Result<Vec<_>, StoreError>>()?,
        );
    }
    let Some(value) = record.root().get_field(FieldId::new(2)?)? else {
        return Ok(());
    };
    let ValueRef::List(entries) = value else {
        return Err(StoreError::Invalid("facet map exclusions are not a list"));
    };
    if entries.count()? > 64 {
        return Err(StoreError::Invalid("facet map exclusions exceed bounds"));
    }
    for entry in entries.iter()? {
        let ValueRef::List(entry) = entry? else {
            return Err(StoreError::Invalid("facet map exclusion is not a pair"));
        };
        if entry.count()? != 2 {
            return Err(StoreError::Invalid("facet map exclusion is not a pair"));
        }
        let Some(ValueRef::List(path)) = entry.get(0)? else {
            return Err(StoreError::Invalid("facet map exclusion path is absent"));
        };
        let path = path
            .iter()?
            .map(|value| match value? {
                ValueRef::Int(value) => Ok(FieldId::new(u32::try_from(value).map_err(|_| {
                    StoreError::Invalid("facet map exclusion field exceeds bounds")
                })?)?),
                _ => Err(StoreError::Invalid("facet map exclusion field is invalid")),
            })
            .collect::<Result<Vec<_>, StoreError>>()?;
        let Some(ValueRef::Text(key)) = entry.get(1)? else {
            return Err(StoreError::Invalid("facet map exclusion key is absent"));
        };
        keys.insert(MapKeyExclusion::new(path, key)?);
    }
    Ok(())
}
