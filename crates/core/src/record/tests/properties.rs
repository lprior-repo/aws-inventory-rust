//! Properties over generated value trees and over arbitrary bytes.

use std::collections::BTreeMap;

use proptest::collection::{btree_map, vec};
use proptest::prelude::*;
use proptest::sample::Index;

use super::super::{Float, Time};
use super::{FieldId, Record, RecordError, Value, decoded};

fn scalar() -> impl Strategy<Value = Value> {
    prop_oneof![
        Just(Value::Null),
        any::<bool>().prop_map(Value::Bool),
        any::<i64>().prop_map(Value::Int),
        any::<f64>().prop_map(|float| Value::Float(Float::from(float))),
        "[a-z\u{e9}]{0,6}".prop_map(Value::Text),
        vec(any::<u8>(), 0..6).prop_map(Value::Bytes),
        (any::<i64>(), 0u32..1_000_000_000)
            .prop_filter_map("nanoseconds in range", |(seconds, nanos)| {
                Time::new(seconds, nanos).map(Value::Time)
            }),
    ]
}

fn struct_of(entries: BTreeMap<u32, Value>) -> Value {
    Value::Struct(
        entries
            .into_iter()
            .filter_map(|(id, value)| Some((FieldId::new(id)?, value)))
            .collect(),
    )
}

fn tree() -> impl Strategy<Value = Value> {
    scalar().prop_recursive(4, 64, 8, |inner| {
        prop_oneof![
            vec(inner.clone(), 0..8).prop_map(Value::List),
            btree_map("[a-z]{0,4}", inner.clone(), 0..8).prop_map(Value::Map),
            btree_map(1u32..50, inner, 0..8).prop_map(struct_of),
        ]
    })
}

fn top() -> impl Strategy<Value = Value> {
    btree_map(1u32..50, tree(), 0..8).prop_map(struct_of)
}

fn fail(error: RecordError) -> TestCaseError {
    TestCaseError::fail(error.to_string())
}

/// Bytes are either rejected or exactly the encoding of what they decode to.
fn rejected_or_canonical(bytes: Vec<u8>) -> Result<(), TestCaseError> {
    if let Ok(record) = Record::from_bytes(bytes.clone()) {
        let again = Record::encode(&decoded(&record)).map_err(fail)?;
        prop_assert_eq!(again.into_bytes(), bytes);
    }
    Ok(())
}

proptest! {
    #[test]
    fn an_encoded_value_validates_and_reads_back(value in top()) {
        let record = Record::encode(&value).map_err(fail)?;
        let reread = Record::from_bytes(record.as_bytes().to_vec()).map_err(fail)?;
        prop_assert_eq!(decoded(&reread), value);
    }

    #[test]
    fn arbitrary_bytes_are_rejected_or_canonical(bytes in vec(any::<u8>(), 0..48)) {
        rejected_or_canonical(bytes)?;
    }

    #[test]
    fn one_changed_byte_is_rejected_or_canonical(
        value in top(),
        index in any::<Index>(),
        byte in any::<u8>(),
    ) {
        let mut bytes = Record::encode(&value).map_err(fail)?.into_bytes();
        let position = index.index(bytes.len());
        if let Some(slot) = bytes.get_mut(position) {
            *slot = byte;
        }
        rejected_or_canonical(bytes)?;
    }

    #[test]
    fn every_strict_prefix_is_rejected(value in top()) {
        let bytes = Record::encode(&value).map_err(fail)?.into_bytes();
        for len in 0..bytes.len() {
            let prefix = bytes.get(..len).unwrap_or(&[]).to_vec();
            prop_assert!(Record::from_bytes(prefix).is_err());
        }
    }

    #[test]
    fn lookup_agrees_with_the_owned_tree(value in top(), absent in 50u32..60) {
        let record = Record::encode(&value).map_err(fail)?;
        let Value::Struct(fields) = value else {
            return Err(TestCaseError::fail("generator produced a non-struct"));
        };
        for (id, expected) in fields {
            let found = record.root().get(id).map(|found| found.to_value());
            prop_assert_eq!(found, Some(expected));
        }
        let missing = FieldId::new(absent).and_then(|id| record.root().get(id));
        prop_assert_eq!(missing, None);
    }
}
