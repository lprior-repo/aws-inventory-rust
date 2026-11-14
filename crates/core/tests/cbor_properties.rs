//! Seeded, bounded properties over supported public record values.

mod cbor_support;

use std::collections::BTreeMap;

use cbor_support::{TestResult, envelope, owned};
use inventory_core::record::{FieldId, Float, Key, Record, RecordError, Time, Value};
use proptest::prelude::*;
use proptest::test_runner::{RngSeed, TestCaseError};

fn config() -> ProptestConfig {
    ProptestConfig {
        cases: 256,
        max_shrink_iters: 1_024,
        failure_persistence: None,
        rng_seed: RngSeed::Fixed(0x4342_4F52_4D33_3161),
        ..ProptestConfig::default()
    }
}

fn key() -> impl Strategy<Value = Key> {
    prop_oneof![
        (1u32..128).prop_filter_map("nonzero field identifier", |number| {
            match FieldId::new(number) {
                Ok(id) => Some(Key::Field(id)),
                Err(_) => None,
            }
        }),
        "k[a-z]{0,7}".prop_map(Key::Name),
    ]
}

fn scalar() -> impl Strategy<Value = Value> {
    prop_oneof![
        Just(Value::Null),
        any::<bool>().prop_map(Value::Bool),
        any::<i64>().prop_map(Value::Int),
        proptest::num::f64::ANY.prop_map(|number| Value::Float(Float::new(number))),
        "[a-zé]{0,4}".prop_map(Value::Text),
        prop::collection::vec(any::<u8>(), 0..=8).prop_map(Value::Bytes),
        (any::<i64>(), 0u32..1_000_000_000).prop_filter_map("valid time", |(seconds, nanos)| {
            match Time::new(seconds, nanos) {
                Ok(time) => Some(Value::Time(time)),
                Err(_) => None,
            }
        }),
    ]
}

fn tree(children: u8) -> impl Strategy<Value = Value> {
    // Three recursive container layers plus the top-level map give depth <= 4.
    scalar().prop_recursive(3, 32, u32::from(children), move |inner| {
        let maximum = usize::from(children);
        prop_oneof![
            prop::collection::vec(inner.clone(), 0..=maximum).prop_map(Value::List),
            prop::collection::btree_map(key(), inner, 0..=maximum).prop_map(Value::Map),
        ]
    })
}

fn record_value(children: u8) -> impl Strategy<Value = Value> {
    let maximum = usize::from(children);
    prop::collection::btree_map(key(), tree(children), 0..=maximum).prop_map(Value::Map)
}

fn failure(error: impl std::fmt::Display) -> TestCaseError {
    TestCaseError::fail(error.to_string())
}

fn encode_and_validate(value: &Value) -> TestResult<Record> {
    let encoded = Record::encode(value)?;
    Ok(Record::from_bytes(encoded.into_bytes())?)
}

fn decode_and_read(bytes: Vec<u8>) -> Result<Value, RecordError> {
    let record = Record::from_bytes(bytes)?;
    owned(&record)
}

proptest! {
    #![proptest_config(config())]

    #[test]
    fn supported_trees_roundtrip_with_identical_values_and_canonical_bytes(value in record_value(8)) {
        // Given a supported tree bounded to four container levels and eight children.
        // When encoded, structurally validated, and reconstructed.
        let record = encode_and_validate(&value).map_err(failure)?;
        let recovered = owned(&record).map_err(failure)?;
        // Then every value and every byte of canonical re-encoding agree.
        prop_assert_eq!(&recovered, &value);
        let encoded_again = Record::encode(&recovered).map_err(failure)?;
        prop_assert_eq!(encoded_again.as_bytes(), record.as_bytes());
    }

    #[test]
    fn generated_lookup_returns_every_exact_value_and_absence(value in record_value(8)) {
        // Given a generated map containing either kind of key and nested values.
        let entries = match &value {
            Value::Map(entries) => entries,
            _ => return Err(TestCaseError::fail("strategy root is not a map")),
        };
        let record = encode_and_validate(&value).map_err(failure)?;
        // When every present key and two reserved absent keys are queried.
        entries.iter().try_for_each(|(key, expected)| {
            let actual = match key {
                Key::Field(id) => record.root().get_field(*id),
                Key::Name(name) => record.root().get_name(name),
            }.map_err(failure)?.map(|v| v.to_value()).transpose().map_err(failure)?;
            // Then borrowed lookup agrees exactly with the generated owned tree.
            prop_assert_eq!(actual.as_ref(), Some(expected));
            Ok(())
        })?;
        prop_assert_eq!(record.root().get_name("\0").map_err(failure)?
            .map(|v| v.to_value()).transpose().map_err(failure)?, None);
        let absent = FieldId::new(u32::MAX).map_err(failure)?;
        prop_assert_eq!(record.root().get_field(absent).map_err(failure)?
            .map(|v| v.to_value()).transpose().map_err(failure)?, None);
    }

    #[test]
    fn mutating_one_valid_record_byte_never_panics(
        value in record_value(8), position in any::<usize>(), mask in 1u8..=255,
    ) {
        // Given valid encoded bytes, mutate exactly one byte using a nonzero XOR mask.
        let mut bytes = Record::encode(&value).map_err(failure)?.into_bytes();
        let index = position.checked_rem(bytes.len()).ok_or_else(|| failure("empty encoding"))?;
        let byte = bytes.get_mut(index).ok_or_else(|| failure("mutation bounds"))?;
        let before = *byte;
        *byte = before ^ mask;
        prop_assert_ne!(*byte, before);
        // When public decode and owned read run, expected parser errors propagate here.
        let recovered = match decode_and_read(bytes) {
            Ok(value) => value,
            Err(_) => return Ok(()),
        };
        // Then any successful result is itself a valid, exactly round-trippable tree.
        let canonical = encode_and_validate(&recovered).map_err(failure)?;
        prop_assert_eq!(owned(&canonical).map_err(failure)?, recovered);
    }

    #[test]
    fn every_strict_prefix_of_a_generated_record_is_rejected(value in record_value(2)) {
        // Given a smaller generated record: depth <= 4, children <= 2, <= 1024 bytes.
        let bytes = Record::encode(&value).map_err(failure)?.into_bytes();
        prop_assert!(bytes.len() <= 1_024);
        // When every strict prefix, including empty, is structurally validated.
        (0..bytes.len()).try_for_each(|length| {
            let prefix = bytes.get(..length).ok_or_else(|| failure("prefix bounds"))?;
            let result = Record::from_bytes(prefix.to_vec());
            // Then no incomplete map is accepted, irrespective of its value kinds.
            prop_assert!(matches!(result, Err(RecordError::Missing { .. })),
                "accepted or misclassified prefix of length {}: {:?}", length, result);
            Ok(())
        })?;
    }
}

#[test]
fn empty_nested_map_remains_a_present_value_after_roundtrip() -> TestResult {
    // Given the generator's empty-container corner case in an enclosing map.
    let value = envelope(Value::Map(BTreeMap::new()));
    // When encoded and reconstructed through the same public path.
    let record = encode_and_validate(&value)?;
    // Then an empty nested map is a value, not a missing field.
    assert_eq!(owned(&record)?, value);
    Ok(())
}
