//! Field, key and item access on the borrowed readers.

use super::super::{Float, Time, ValueRef};
use super::{Record, TestResult, Value, fid, fields, map, text};

/// One field of every kind, so a lookup of the last field skips every kind.
fn one_of_each() -> Result<Value, &'static str> {
    let time = Time::new(5, 6).ok_or("valid time rejected")?;
    Ok(fields([
        (1, Value::Null),
        (2, Value::Bool(true)),
        (3, Value::Int(-300)),
        (4, Value::Float(Float::from(1.5))),
        (5, text("skipped text")),
        (6, Value::Bytes(vec![0xFF, 0xFE])),
        (7, Value::Time(time)),
        (8, Value::List(vec![Value::Int(1), text("x")])),
        (9, map([("a", Value::Int(1)), ("b", Value::Null)])),
        (10, fields([(1, Value::Int(2))])),
        (11, Value::Int(42)),
    ]))
}

#[test]
fn a_field_is_found_wherever_it_sits() -> TestResult {
    let record = Record::encode(&one_of_each()?)?;
    let root = record.root();
    assert_eq!(root.get(fid(1)?), Some(ValueRef::Null));
    assert_eq!(root.get(fid(3)?), Some(ValueRef::Int(-300)));
    assert_eq!(root.get(fid(5)?), Some(ValueRef::Text("skipped text")));
    assert_eq!(root.get(fid(6)?), Some(ValueRef::Bytes(&[0xFF, 0xFE])));
    assert_eq!(root.get(fid(11)?), Some(ValueRef::Int(42)));
    Ok(())
}

#[test]
fn an_absent_field_is_none_before_between_and_after() -> TestResult {
    let record = Record::encode(&fields([(2, Value::Int(1)), (4, Value::Int(2))]))?;
    let root = record.root();
    for absent in [1, 3, 5, u32::MAX] {
        assert_eq!(root.get(fid(absent)?), None);
    }
    assert_eq!(Record::encode(&fields([]))?.root().get(fid(1)?), None);
    Ok(())
}

#[test]
fn nested_containers_are_read_in_place() -> TestResult {
    let record = Record::encode(&one_of_each()?)?;
    let root = record.root();
    let Some(ValueRef::Struct(inner)) = root.get(fid(10)?) else {
        return Err("field 10 is not a struct".into());
    };
    assert_eq!(inner.get(fid(1)?), Some(ValueRef::Int(2)));
    let Some(ValueRef::List(list)) = root.get(fid(8)?) else {
        return Err("field 8 is not a list".into());
    };
    assert_eq!(list.len(), 2);
    assert!(!list.is_empty());
    let items: Vec<ValueRef<'_>> = list.iter().collect();
    assert_eq!(items, [ValueRef::Int(1), ValueRef::Text("x")]);
    Ok(())
}

#[test]
fn a_map_key_is_found_or_absent_by_byte_order() -> TestResult {
    let entries = map([
        ("a", Value::Int(1)),
        ("ab", text("skipped")),
        ("b", Value::Int(3)),
    ]);
    let record = Record::encode(&fields([(1, entries)]))?;
    let Some(ValueRef::Map(found)) = record.root().get(fid(1)?) else {
        return Err("field 1 is not a map".into());
    };
    assert_eq!(found.get("a"), Some(ValueRef::Int(1)));
    assert_eq!(found.get("ab"), Some(ValueRef::Text("skipped")));
    assert_eq!(found.get("b"), Some(ValueRef::Int(3)));
    for absent in ["", "aa", "abc", "c"] {
        assert_eq!(found.get(absent), None);
    }
    let keys: Vec<&str> = found.iter().map(|(key, _)| key).collect();
    assert_eq!(keys, ["a", "ab", "b"]);
    Ok(())
}

#[test]
fn an_empty_list_has_no_items() -> TestResult {
    let record = Record::encode(&fields([(1, Value::List(vec![]))]))?;
    let Some(ValueRef::List(list)) = record.root().get(fid(1)?) else {
        return Err("field 1 is not a list".into());
    };
    assert!(list.is_empty());
    assert_eq!(list.iter().count(), 0);
    Ok(())
}

#[test]
fn fields_iterate_in_id_order_with_their_values() -> TestResult {
    let record = Record::encode(&fields([(9, Value::Int(2)), (3, Value::Int(1))]))?;
    let seen: Vec<(u32, ValueRef<'_>)> = record
        .root()
        .iter()
        .map(|(id, value)| (id.get(), value))
        .collect();
    assert_eq!(seen, [(3, ValueRef::Int(1)), (9, ValueRef::Int(2))]);
    Ok(())
}
