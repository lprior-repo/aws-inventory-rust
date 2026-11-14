use super::{Record, RecordError, TestResult, Time, Value, envelope};
use crate::record::ValueRef;
use minicbor::data::Type;

fn timestamp_chain(maps: usize) -> TestResult<(Value, Vec<u8>)> {
    let time = Time::new(-1, 999_999_999)?;
    let value = (0..maps).fold(Value::Time(time), |value, _| envelope(value));
    let mut encoder = minicbor::Encoder::new(Vec::new());
    (0..maps).try_for_each(|_| encoder.map(1)?.str("value").map(|_| ()))?;
    encoder
        .tag(minicbor::data::Tag::new(1))?
        .array(2)?
        .i64(-1)?
        .u32(999_999_999)?;
    Ok((value, encoder.into_writer()))
}

#[test]
fn timestamp_array_is_the_thirty_second_container_at_the_inclusive_boundary() -> TestResult {
    // Given 31 maps followed by the timestamp's physical array.
    let (value, bytes) = timestamp_chain(31)?;
    // When encoding and rebuilding independently written bytes, then both accept depth 32.
    assert_eq!(Record::encode(&value)?.as_bytes(), bytes);
    assert_eq!(
        ValueRef::Map(Record::from_bytes(bytes)?.root()).to_value()?,
        value
    );
    Ok(())
}

#[test]
fn timestamp_array_is_rejected_when_it_is_the_thirty_third_container() -> TestResult {
    // Given 32 maps followed by the timestamp's array: 33 physical containers.
    let (value, bytes) = timestamp_chain(32)?;
    // When constructing from either tree or bytes, then only generic structural validation accepts.
    assert_eq!(Record::encode(&value), Err(RecordError::DepthLimit));
    assert_eq!(
        ValueRef::Map(Record::from_bytes(bytes)?.root()).to_value(),
        Err(RecordError::DepthLimit)
    );
    Ok(())
}

fn time_error(seconds: &[u8], nanos: &[u8]) -> TestResult<RecordError> {
    let bytes = [0xA1, 1, 0xC1, 0x82]
        .into_iter()
        .chain(seconds.iter().copied())
        .chain(nanos.iter().copied())
        .collect();
    ValueRef::Map(Record::from_bytes(bytes)?.root())
        .to_value()
        .err()
        .ok_or_else(|| "accepted invalid time".into())
}

#[test]
fn timestamp_component_types_are_not_misreported_as_integer_overflow() -> TestResult {
    // Given well-formed noninteger seconds or nanoseconds.
    // When read, then the actual encountered type and position are retained.
    assert_eq!(
        time_error(&[0xF5], &[0])?,
        RecordError::ValueType {
            found: Type::Bool,
            offset: 4
        }
    );
    assert_eq!(
        time_error(&[0], &[0xF5])?,
        RecordError::ValueType {
            found: Type::Bool,
            offset: 5
        }
    );
    assert_eq!(
        time_error(&[0x60], &[0])?,
        RecordError::ValueType {
            found: Type::String,
            offset: 4
        }
    );
    Ok(())
}

#[test]
fn timestamp_integer_range_errors_remain_distinct_from_component_types() -> TestResult {
    // Given a native unsigned integer that cannot fit seconds, and negative nanoseconds.
    let overflow = [0x1B, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
    // When read, then integer bounds and a negative fraction have specific domain errors.
    assert_eq!(
        time_error(&overflow, &[0])?,
        RecordError::UnsignedOutOfRange { offset: 4 }
    );
    assert_eq!(
        time_error(&[0], &[0x20])?,
        RecordError::NegativeNanos {
            nanos: -1,
            offset: 5
        }
    );
    Ok(())
}

#[test]
fn extreme_native_negative_integers_retain_their_exact_nanosecond_cause() -> TestResult {
    // Given the most negative native CBOR integer, beyond the i64 seconds domain.
    // When used in either component, then the domain failure remains exact.
    let negative = [0x3B, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
    assert_eq!(
        time_error(&negative, &[0])?,
        RecordError::SignedOutOfRange { offset: 4 }
    );
    assert_eq!(
        time_error(&[0], &negative)?,
        RecordError::NegativeNanos {
            nanos: -18_446_744_073_709_551_616,
            offset: 5
        }
    );
    Ok(())
}
