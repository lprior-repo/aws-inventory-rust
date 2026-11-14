//! Mixed-key lookup and bounded container views.

mod cbor_support;

use std::collections::BTreeMap;

use cbor_support::{TestResult, envelope, owned};
use inventory_core::record::{FieldId, Key, ListRef, Record, Value, ValueRef};

fn mixed_value() -> TestResult<Value> {
    let nested = Value::List(vec![envelope(Value::List(vec![Value::Int(99)]))]);
    Ok(Value::Map(BTreeMap::from([
        (Key::Field(FieldId::new(2)?), nested.clone()),
        (Key::Field(FieldId::new(4)?), Value::Int(40)),
        (Key::Field(FieldId::new(6)?), Value::Int(60)),
        (Key::Name("b".into()), nested),
        (Key::Name("d".into()), Value::Text("middle".into())),
        (Key::Name("f".into()), Value::Bool(false)),
    ])))
}

#[test]
fn mixed_map_finds_first_middle_last_fields_and_skips_nested_values() -> TestResult {
    // Given mixed field and name keys with a nested first value.
    let value = mixed_value()?;
    let record = Record::encode(&value)?;
    let expected = match &value {
        Value::Map(entries) => entries,
        _ => return Err("fixture must be a map".into()),
    };
    // When querying each field position and gaps before, between, and after.
    [1, 2, 3, 4, 6, 7, u32::MAX]
        .into_iter()
        .try_for_each(|number| -> TestResult {
            let id = FieldId::new(number)?;
            let actual = record
                .root()
                .get_field(id)?
                .map(|v| v.to_value())
                .transpose()?;
            // Then the result is the exact corresponding value or absence.
            assert_eq!(
                actual.as_ref(),
                expected.get(&Key::Field(id)),
                "field {number}"
            );
            Ok(())
        })?;
    assert_eq!(owned(&record)?, value);
    Ok(())
}

#[test]
fn mixed_map_finds_first_middle_last_names_and_skips_fields_and_containers() -> TestResult {
    // Given the same mixed map, including nested field and name values.
    let value = mixed_value()?;
    let record = Record::encode(&value)?;
    let expected = match &value {
        Value::Map(entries) => entries,
        _ => return Err("fixture must be a map".into()),
    };
    // When querying positions and gaps before, between, and after.
    ["", "a", "b", "c", "d", "f", "z"]
        .into_iter()
        .try_for_each(|name| -> TestResult {
            let actual = record
                .root()
                .get_name(name)?
                .map(|v| v.to_value())
                .transpose()?;
            // Then no field key or skipped nested item changes the answer.
            assert_eq!(
                actual.as_ref(),
                expected.get(&Key::Name(name.into())),
                "name {name}"
            );
            Ok(())
        })?;
    Ok(())
}

fn named_list<'a>(record: &'a Record, name: &str) -> TestResult<ListRef<'a>> {
    match record.root().get_name(name)? {
        Some(ValueRef::List(list)) => Ok(list),
        other => Err(format!("expected list for {name}, got {other:?}").into()),
    }
}

#[test]
fn list_get_never_exposes_following_map_fields_outside_empty_or_nonempty_lists() -> TestResult {
    // Given lists immediately followed by other valid map entries.
    let value = Value::Map(BTreeMap::from([
        (Key::Name("empty".into()), Value::List(vec![])),
        (Key::Name("items".into()), Value::List(vec![Value::Int(7)])),
        (Key::Name("sentinel".into()), Value::Int(123)),
    ]));
    let record = Record::encode(&value)?;
    // When indexes include the exact length, larger positions, and usize::MAX.
    let empty = named_list(&record, "empty")?;
    let items = named_list(&record, "items")?;
    // Then valid reads are exact and every out-of-range read is None.
    assert_eq!(
        items.get(0)?.map(|v| v.to_value()).transpose()?,
        Some(Value::Int(7))
    );
    [0, 1, 2, usize::MAX]
        .into_iter()
        .try_for_each(|index| -> TestResult {
            assert_eq!(empty.get(index)?.map(|v| v.to_value()).transpose()?, None);
            Ok(())
        })?;
    [1, 2, 3, usize::MAX]
        .into_iter()
        .try_for_each(|index| -> TestResult {
            assert_eq!(items.get(index)?.map(|v| v.to_value()).transpose()?, None);
            Ok(())
        })
}

fn large_map(count: u32) -> TestResult<Value> {
    let entries = (1..=count)
        .map(|number| {
            Ok((
                Key::Field(FieldId::new(number)?),
                Value::Int(i64::from(number)),
            ))
        })
        .collect::<Result<BTreeMap<_, _>, inventory_core::record::RecordError>>()?;
    Ok(Value::Map(entries))
}

#[test]
fn nested_arrays_with_multibyte_headers_retain_all_items_and_the_following_entry() -> TestResult {
    // Given nested arrays straddling both multibyte-length thresholds.
    [24usize, 25, 256]
        .into_iter()
        .try_for_each(|count| -> TestResult {
            let items = (0..count)
                .map(|n| i64::try_from(n).map(Value::Int))
                .collect::<Result<Vec<_>, _>>()?;
            let value = envelope(Value::List(vec![
                Value::List(items.clone()),
                Value::Int(-7),
            ]));
            let record = Record::encode(&value)?;
            // When reading through two borrowed container layers.
            let outer = named_list(&record, "value")?;
            let inner = match outer.get(0)? {
                Some(ValueRef::List(list)) => list,
                other => return Err(format!("expected inner list, got {other:?}").into()),
            };
            let actual = inner
                .iter()?
                .map(|item| item.and_then(|v| v.to_value()))
                .collect::<Result<Vec<_>, _>>()?;
            // Then headers are not mistaken for payload, and iteration advances correctly.
            assert_eq!(actual, items);
            assert_eq!(inner.get(count)?.map(|v| v.to_value()).transpose()?, None);
            assert_eq!(
                outer.get(1)?.map(|v| v.to_value()).transpose()?,
                Some(Value::Int(-7))
            );
            assert_eq!(owned(&record)?, value);
            Ok(())
        })
}

#[test]
fn nested_maps_with_multibyte_headers_preserve_lookup_iteration_and_following_values() -> TestResult
{
    // Given maps with multibyte headers followed by another list item.
    [24, 25, 256]
        .into_iter()
        .try_for_each(|count| -> TestResult {
            let nested = large_map(count)?;
            let value = envelope(Value::List(vec![nested.clone(), Value::Bool(true)]));
            let record = Record::encode(&value)?;
            // When traversing the nested map through borrowed views.
            let outer = named_list(&record, "value")?;
            let map = match outer.get(0)? {
                Some(ValueRef::Map(map)) => map,
                other => return Err(format!("expected nested map, got {other:?}").into()),
            };
            let entries = map
                .iter()?
                .map(|entry| {
                    entry.and_then(|(key, value)| {
                        value.to_value().map(|value| (key.to_key(), value))
                    })
                })
                .collect::<Result<BTreeMap<_, _>, _>>()?;
            // Then the full map and its boundary lookups are exact.
            assert_eq!(Value::Map(entries), nested);
            assert_eq!(
                map.get_field(FieldId::new(count)?)?
                    .map(|v| v.to_value())
                    .transpose()?,
                Some(Value::Int(i64::from(count)))
            );
            assert_eq!(
                outer.get(1)?.map(|v| v.to_value()).transpose()?,
                Some(Value::Bool(true))
            );
            assert_eq!(owned(&record)?, value);
            Ok(())
        })
}

#[test]
fn map_and_list_iterators_continue_correctly_after_container_valued_entries() -> TestResult {
    // Given alternating container and scalar entries in both kinds of container.
    let list = Value::List(vec![
        envelope(Value::Null),
        Value::Int(9),
        Value::List(vec![Value::Int(10)]),
        Value::Text("tail".into()),
    ]);
    let value = Value::Map(BTreeMap::from([
        (Key::Name("a".into()), list.clone()),
        (Key::Name("b".into()), envelope(Value::Bool(false))),
        (Key::Name("c".into()), Value::Int(11)),
    ]));
    let record = Record::encode(&value)?;
    // When iterating the map and the inner list independently.
    let entries = record
        .root()
        .iter()?
        .map(|entry| {
            entry.and_then(|(key, value)| value.to_value().map(|value| (key.to_key(), value)))
        })
        .collect::<Result<BTreeMap<_, _>, _>>()?;
    let items = named_list(&record, "a")?
        .iter()?
        .map(|item| item.and_then(|v| v.to_value()))
        .collect::<Result<Vec<_>, _>>()?;
    // Then neither iterator repeats a child nor loses the scalar after it.
    assert_eq!(Value::Map(entries), value);
    assert_eq!(Value::List(items), list);
    Ok(())
}
