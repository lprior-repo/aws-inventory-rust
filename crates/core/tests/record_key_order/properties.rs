//! Every generated map is scanned and every adjacent pair is independently swapped.
use super::{TestResult, linear, lookup, rejected};
use inventory_core::record::{
    FieldId, Float, Key, KeyRef, MapRef, Record, RecordError, Time, Value, ValueRef,
};
use minicbor::{Decoder, data::Type};
use proptest::prelude::*;
use proptest::test_runner::{RngSeed, TestCaseError, TestRunner};
use std::ops::Range;

struct MapSpan {
    bytes: Range<usize>,
    entries: Vec<Range<usize>>,
}

fn config() -> ProptestConfig {
    ProptestConfig {
        cases: 1_024,
        max_shrink_iters: 1_024,
        failure_persistence: None,
        rng_seed: RngSeed::Fixed(0x4D33_3163_4B45_5953),
        ..ProptestConfig::default()
    }
}

fn key() -> TestResult<impl Strategy<Value = Key> + Clone> {
    let fields = [1, 23, 24, 255, 256, u32::MAX]
        .into_iter()
        .map(|number| FieldId::new(number).map(Key::Field))
        .collect::<Result<Vec<_>, RecordError>>()?;
    Ok(prop_oneof![
        prop::sample::select(fields),
        "[a-zé]{0,6}".prop_map(Key::Name),
    ])
}

fn scalar() -> TestResult<impl Strategy<Value = Value>> {
    Ok(prop_oneof![
        Just(Value::Null),
        any::<bool>().prop_map(Value::Bool),
        any::<i64>().prop_map(Value::Int),
        any::<f64>().prop_map(|v| Value::Float(Float::new(v))),
        "[a-zé]{0,8}".prop_map(Value::Text),
        prop::collection::vec(any::<u8>(), 0..=8).prop_map(Value::Bytes),
        Just(Value::Time(Time::new(-1, 999_999_999)?)),
    ])
}

fn records() -> TestResult<impl Strategy<Value = Value>> {
    let keys = key()?;
    let nested_keys = keys.clone();
    let tree = scalar()?.prop_recursive(3, 32, 4, move |inner| {
        prop_oneof![
            prop::collection::vec(inner.clone(), 0..=4).prop_map(Value::List),
            prop::collection::btree_map(nested_keys.clone(), inner, 0..=4).prop_map(Value::Map),
        ]
    });
    Ok(prop::collection::btree_map(keys, tree, 0..=5).prop_map(Value::Map))
}

fn map_span(d: &mut Decoder<'_>) -> TestResult<MapSpan> {
    let start = d.position();
    let count = d.map()?.ok_or("generated indefinite map")?;
    let mut probe = d.clone();
    let entries = (0..count)
        .map(|_| -> TestResult<Range<usize>> {
            let start = probe.position();
            probe.skip()?;
            probe.skip()?;
            Ok(start..probe.position())
        })
        .collect::<TestResult<Vec<_>>>()?;
    Ok(MapSpan {
        bytes: start..probe.position(),
        entries,
    })
}

fn spans(bytes: &[u8]) -> TestResult<Vec<MapSpan>> {
    let mut d = Decoder::new(bytes);
    let mut maps = Vec::new();
    // Every nonterminal iteration consumes at least one byte, including map keys.
    (0..bytes.len()).try_for_each(|_| -> TestResult {
        if d.position() == bytes.len() {
            return Ok(());
        }
        scan_item(&mut d, &mut maps)?;
        Ok(())
    })?;
    assert_eq!(d.position(), bytes.len());
    Ok(maps)
}

fn scan_item(d: &mut Decoder<'_>, maps: &mut Vec<MapSpan>) -> TestResult {
    match d.datatype()? {
        Type::Map => maps.push(map_span(d)?),
        Type::Array => {
            d.array()?;
        }
        Type::Tag => {
            d.tag()?;
        }
        _ => {
            d.skip()?;
        }
    }
    Ok(())
}

fn swapped(bytes: &[u8], first: &Range<usize>, second: &Range<usize>) -> TestResult<Vec<u8>> {
    Ok(bytes
        .get(..first.start)
        .ok_or("prefix")?
        .iter()
        .chain(bytes.get(second.clone()).ok_or("second entry")?)
        .chain(bytes.get(first.clone()).ok_or("first entry")?)
        .chain(bytes.get(second.end..).ok_or("suffix")?)
        .copied()
        .collect())
}

fn reject_swaps(bytes: &[u8], map: &MapSpan) -> TestResult {
    map.entries.windows(2).try_for_each(|pair| -> TestResult {
        let mut entries = pair.iter();
        let first = entries.next().ok_or("first entry")?;
        let second = entries.next().ok_or("second entry")?;
        let length = second.end.checked_sub(second.start).ok_or("entry length")?;
        let offset = first.start.checked_add(length).ok_or("key offset")?;
        rejected(
            &swapped(bytes, first, second)?,
            RecordError::KeyOutOfOrder { offset },
        );
        Ok(())
    })
}

fn compare_lookup(map: MapRef<'_>, key: KeyRef<'_>) -> TestResult {
    assert_eq!(lookup(map, key)?, linear(map, key)?, "key {key:?}");
    Ok(())
}

fn compare_map(bytes: &[u8], span: &MapSpan) -> TestResult {
    let map = MapRef::from_bytes(bytes.get(span.bytes.clone()).ok_or("map bytes")?)?;
    map.iter()?
        .try_for_each(|entry| -> TestResult { compare_lookup(map, entry?.0) })?;
    (1..=65).try_for_each(|number| -> TestResult {
        compare_lookup(map, KeyRef::Field(FieldId::new(number)?))
    })?;
    compare_lookup(map, KeyRef::Field(FieldId::new(u32::MAX)?))?;
    ["", "a", "aa", "z", "é", "ê", "zzzzzzz"]
        .into_iter()
        .try_for_each(|name| compare_lookup(map, KeyRef::Name(name)))
}

fn check_record(value: Value) -> TestResult {
    let encoded = Record::encode(&value)?;
    let bytes = encoded.as_bytes();
    let owned = Record::from_bytes(bytes.to_vec())?;
    let borrowed = MapRef::from_bytes(bytes)?;
    assert_eq!(ValueRef::Map(owned.root()).to_value()?, value);
    assert_eq!(ValueRef::Map(borrowed).to_value()?, value);
    spans(bytes)?.iter().try_for_each(|span| -> TestResult {
        reject_swaps(bytes, span)?;
        compare_map(bytes, span)
    })
}

#[test]
fn seeded_encoder_records_validate_reject_all_adjacent_map_swaps_and_match_linear_lookup()
-> TestResult {
    // Given bounded generated trees with mixed keys, nested maps, lists and all scalar kinds.
    let mut runner = TestRunner::new(config());
    // When encoded, then both constructors accept; every map swap fails and lookup equals scan.
    runner.run(&records()?, |value| {
        check_record(value).map_err(|error| TestCaseError::fail(error.to_string()))
    })?;
    Ok(())
}
