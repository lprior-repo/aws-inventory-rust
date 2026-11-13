//! Every way bytes can fail to be a record, with the offset reported.
//!
//! Records are written out by hand; the comment above each gives its layout.

use super::super::MAX_RECORD_BYTES;
use super::RecordError::{
    BodyNotConsumed, InvalidFieldId, InvalidNanoseconds, InvalidUtf8, LengthOutOfBounds,
    NonCanonicalNan, NonMinimalVarint, OrderViolation, TooLarge, TopLevelNotStruct, TrailingBytes,
    UnexpectedEnd, UnknownTag, VarintOverflow,
};
use super::{Record, TestResult, Value, fields, map, rejection, text};

#[test]
fn the_top_level_must_be_a_complete_struct() -> TestResult {
    assert_eq!(rejection(&[])?, UnexpectedEnd { offset: 0 });
    assert_eq!(rejection(&[0])?, TopLevelNotStruct);
    assert_eq!(rejection(&[8, 1, 0])?, TopLevelNotStruct);
    // struct, body length 1, count 0, then one extra byte
    assert_eq!(rejection(&[10, 1, 0, 0])?, TrailingBytes { offset: 3 });
    Ok(())
}

#[test]
fn lengths_and_counts_must_match_the_bytes_present() -> TestResult {
    // body length 5 with one byte left
    assert_eq!(rejection(&[10, 5, 0])?, LengthOutOfBounds { offset: 1 });
    // body of two bytes: count 0, then an unread byte
    assert_eq!(rejection(&[10, 2, 0, 0])?, BodyNotConsumed { offset: 3 });
    // body of one byte: count 5 and no entries
    assert_eq!(rejection(&[10, 1, 5])?, UnexpectedEnd { offset: 3 });
    // field 1 is text of length 4 with one byte left in the body
    assert_eq!(
        rejection(&[10, 5, 1, 1, 5, 4, 0x61])?,
        LengthOutOfBounds { offset: 5 }
    );
    // field 1 is a float with three of its eight bytes; the input ends at byte 8
    let short_float = [10, 6, 1, 1, 4, 0, 0, 0];
    assert_eq!(rejection(&short_float)?, UnexpectedEnd { offset: 8 });
    // field 1 is an int whose varint continues past the end of the body
    assert_eq!(
        rejection(&[10, 4, 1, 1, 3, 0x80])?,
        UnexpectedEnd { offset: 6 }
    );
    Ok(())
}

#[test]
fn tags_outside_the_format_are_rejected() -> TestResult {
    // count 1, field 1, tag 99
    assert_eq!(
        rejection(&[10, 3, 1, 1, 99])?,
        UnknownTag { offset: 4, tag: 99 }
    );
    assert_eq!(
        rejection(&[10, 3, 1, 1, 11])?,
        UnknownTag { offset: 4, tag: 11 }
    );
    Ok(())
}

#[test]
fn varints_must_be_minimal_and_fit_in_64_bits() -> TestResult {
    // count written as 0x80 0x00
    assert_eq!(
        rejection(&[10, 2, 0x80, 0x00])?,
        NonMinimalVarint { offset: 2 }
    );
    // body length written as 0x81 0x00
    assert_eq!(
        rejection(&[10, 0x81, 0x00, 0])?,
        NonMinimalVarint { offset: 1 }
    );
    // count with a tenth byte above one
    let mut overflow = vec![10, 10];
    overflow.extend([0xFF; 9]);
    overflow.push(0x02);
    assert_eq!(rejection(&overflow)?, VarintOverflow { offset: 2 });
    Ok(())
}

#[test]
fn field_ids_must_be_non_zero_ascending_and_32_bit() -> TestResult {
    // count 1, field 0
    assert_eq!(rejection(&[10, 3, 1, 0, 0])?, InvalidFieldId { offset: 3 });
    // count 1, field 2^32
    let too_wide = [10, 7, 1, 0x80, 0x80, 0x80, 0x80, 0x10, 0];
    assert_eq!(rejection(&too_wide)?, InvalidFieldId { offset: 3 });
    // count 2, field 2 null, field 1 null
    assert_eq!(
        rejection(&[10, 5, 2, 2, 0, 1, 0])?,
        OrderViolation { offset: 5 }
    );
    // count 2, field 1 null, field 1 null
    assert_eq!(
        rejection(&[10, 5, 2, 1, 0, 1, 0])?,
        OrderViolation { offset: 5 }
    );
    Ok(())
}

#[test]
fn map_keys_must_be_strictly_ascending() -> TestResult {
    // field 1 is a map of 7 bytes: count 2, "b" null, "a" null
    let unsorted = [10, 11, 1, 1, 9, 7, 2, 1, 0x62, 0, 1, 0x61, 0];
    assert_eq!(rejection(&unsorted)?, OrderViolation { offset: 10 });
    // the same with "a" twice
    let duplicate = [10, 11, 1, 1, 9, 7, 2, 1, 0x61, 0, 1, 0x61, 0];
    assert_eq!(rejection(&duplicate)?, OrderViolation { offset: 10 });
    Ok(())
}

#[test]
fn text_and_keys_must_be_utf8() -> TestResult {
    // field 1 is text of one byte, 0xFF
    assert_eq!(
        rejection(&[10, 5, 1, 1, 5, 1, 0xFF])?,
        InvalidUtf8 { offset: 6 }
    );
    // field 1 is a map of 4 bytes: count 1, key 0xFF, null
    let bad_key = [10, 8, 1, 1, 9, 4, 1, 1, 0xFF, 0];
    assert_eq!(rejection(&bad_key)?, InvalidUtf8 { offset: 8 });
    // the same byte is fine in a blob
    assert!(Record::from_bytes(vec![10, 5, 1, 1, 6, 1, 0xFF]).is_ok());
    Ok(())
}

#[test]
fn floats_and_times_must_be_canonical() -> TestResult {
    // field 1 is a float with a NaN payload bit set
    let nan = [10, 11, 1, 1, 4, 0x7F, 0xF8, 0, 0, 0, 0, 0, 1];
    assert_eq!(rejection(&nan)?, NonCanonicalNan { offset: 5 });
    // field 1 is a time: 0 seconds, 1_000_000_000 nanoseconds
    let time = [10, 9, 1, 1, 7, 0, 0x80, 0x94, 0xEB, 0xDC, 0x03];
    assert_eq!(rejection(&time)?, InvalidNanoseconds { offset: 6 });
    // one nanosecond less is accepted
    let time = [10, 9, 1, 1, 7, 0, 0xFF, 0x93, 0xEB, 0xDC, 0x03];
    assert!(Record::from_bytes(time.to_vec()).is_ok());
    Ok(())
}

#[test]
fn every_strict_prefix_of_a_record_is_rejected() -> TestResult {
    let inner = map([("key", Value::List(vec![Value::Int(-70_000), text("v")]))]);
    let bytes = Record::encode(&fields([(1, text("name")), (2, inner)]))?.into_bytes();
    for len in 0..bytes.len() {
        let prefix = bytes.get(..len).ok_or("prefix out of range")?;
        assert!(
            Record::from_bytes(prefix.to_vec()).is_err(),
            "prefix of {len} bytes"
        );
    }
    Ok(())
}

#[test]
fn a_record_over_the_size_limit_is_rejected_before_parsing() -> TestResult {
    let len = MAX_RECORD_BYTES.saturating_add(1);
    assert_eq!(rejection(&vec![0; len])?, TooLarge { len });
    Ok(())
}

#[test]
fn a_value_over_the_size_limit_is_rejected_while_encoding() {
    let len = MAX_RECORD_BYTES;
    let blob = fields([(1, Value::Bytes(vec![0; len]))]);
    assert!(matches!(Record::encode(&blob), Err(TooLarge { .. })));
    let key = "k".repeat(len);
    let keyed = fields([(1, map([(key.as_str(), Value::Null)]))]);
    assert!(matches!(Record::encode(&keyed), Err(TooLarge { .. })));
    // eleven bytes each, so a tenth of the limit in values is over the limit in bytes
    let many = fields([(1, Value::List(vec![Value::Int(i64::MIN); len / 10]))]);
    assert!(matches!(Record::encode(&many), Err(TooLarge { .. })));
}
