//! Seeded proptest runners keep every test's public fallible signature explicit.
use super::support::{TestResult, fields, hash};
use inventory_core::record::*;
use proptest::prelude::*;
use proptest::test_runner::{RngSeed, TestCaseError, TestRunner};

fn config() -> ProptestConfig {
    ProptestConfig {
        cases: 1_024,
        max_shrink_iters: 1_024,
        failure_persistence: None,
        rng_seed: RngSeed::Fixed(0x4841_5348_4D33_3162),
        ..ProptestConfig::default()
    }
}

fn scalar() -> impl Strategy<Value = Value> {
    prop_oneof![
        Just(Value::Null),
        any::<bool>().prop_map(Value::Bool),
        any::<i64>().prop_map(Value::Int),
        proptest::num::f64::ANY.prop_map(|number| Value::Float(Float::new(number))),
        "[a-zé]{0,8}".prop_map(Value::Text),
        prop::collection::vec(any::<u8>(), 0..=8).prop_map(Value::Bytes),
    ]
}

fn tree() -> impl Strategy<Value = Value> {
    // At most three generated container levels and four children per container.
    scalar().prop_recursive(3, 32, 4, |inner| {
        prop_oneof![
            prop::collection::vec(inner.clone(), 0..=4).prop_map(Value::List),
            prop::collection::btree_map("k[a-z]{0,4}", inner, 0..=4).prop_map(
                |entries| Value::Map(
                    entries
                        .into_iter()
                        .map(|(name, value)| (Key::Name(name), value))
                        .collect()
                )
            ),
        ]
    })
}

fn failure(error: impl std::fmt::Display) -> TestCaseError {
    TestCaseError::fail(error.to_string())
}

fn widened_root_key(record: &Record) -> TestResult<Record> {
    let value_bytes = record.as_bytes().get(2..).ok_or("missing root value")?;
    let bytes = [0xA1, 0x1B, 0, 0, 0, 0, 0, 0, 0, 1]
        .into_iter()
        .chain(value_bytes.iter().copied())
        .collect();
    Ok(Record::from_bytes(bytes)?)
}

fn equal_value_case(value: Value) -> TestResult {
    let original = fields([(1, value)])?;
    let first = Record::encode(&original)?;
    let second = widened_root_key(&first)?;
    assert_ne!(first.as_bytes(), second.as_bytes());
    assert_eq!(ValueRef::Map(first.root()).to_value()?, original);
    assert_eq!(ValueRef::Map(second.root()).to_value()?, original);
    assert_eq!(
        first.content_hash(&Exclusions::none())?,
        second.content_hash(&Exclusions::none())?
    );
    Ok(())
}

fn changed_leaf_case((background, leaf): (Value, bool)) -> TestResult {
    let original = fields([(1, background.clone()), (2, Value::Bool(leaf))])?;
    let changed = fields([(1, background), (2, Value::Bool(!leaf))])?;
    assert_ne!(hash(&original)?, hash(&changed)?);
    Ok(())
}

#[test]
fn generated_equal_values_have_equal_hashes() -> TestResult {
    // Given seeded bounded scalar/list/map trees with canonical float values.
    let mut runner = TestRunner::new(config());
    // When key 1 uses shortest versus eight-byte forms, then equal values hash alike.
    runner.run(&tree(), |value| equal_value_case(value).map_err(failure))?;
    Ok(())
}

#[test]
fn generated_records_change_hash_when_one_scalar_leaf_changes() -> TestResult {
    // Given arbitrary bounded background trees and a boolean scalar leaf.
    let mut runner = TestRunner::new(config());
    // When only that leaf is flipped, then no-exclusion hashes differ.
    runner.run(&(tree(), any::<bool>()), |value| {
        changed_leaf_case(value).map_err(failure)
    })?;
    Ok(())
}

fn different_float(value: Float) -> Float {
    match value.bits() {
        0 => Float::new(1.0),
        _ => Float::new(0.0),
    }
}

fn changed_scalar(value: &Value) -> TestResult<Value> {
    Ok(match value {
        Value::Null => Value::Bool(false),
        Value::Bool(value) => Value::Bool(!value),
        Value::Int(value) => Value::Int(value.wrapping_add(1)),
        Value::Float(value) => Value::Float(different_float(*value)),
        Value::Text(value) => Value::Text(format!("{value}x")),
        Value::Bytes(value) => Value::Bytes(value.iter().copied().chain([0]).collect()),
        Value::Time(_) | Value::List(_) | Value::Map(_) => {
            return Err("scalar strategy returned a non-generated kind".into());
        }
    })
}

fn nested_leaf_case((background, leaf, in_map): (Value, Value, bool)) -> TestResult {
    let changed = changed_scalar(&leaf)?;
    assert_ne!(leaf, changed);
    let (before, after) = if in_map {
        (fields([(7, leaf)])?, fields([(7, changed)])?)
    } else {
        (Value::List(vec![leaf]), Value::List(vec![changed]))
    };
    let original = fields([(1, background.clone()), (2, before)])?;
    let mutated = fields([(1, background), (2, after)])?;
    assert_ne!(hash(&original)?, hash(&mutated)?);
    Ok(())
}

#[test]
fn generated_nested_scalar_changes_affect_list_and_map_hashes() -> TestResult {
    // Given seeded bounded background trees and every generated scalar kind, including text.
    let mut runner = TestRunner::new(config());
    let strategy = (tree(), scalar(), any::<bool>());
    // When one leaf inside a list or map changes, then its record's digest changes.
    runner.run(&strategy, |case| nested_leaf_case(case).map_err(failure))?;
    Ok(())
}
