//! Inclusive depth and byte-size limits through public APIs.

mod cbor_support;

use cbor_support::{TestResult, envelope, owned};
use inventory_core::record::{MAX_DEPTH, MAX_RECORD_BYTES, Record, RecordError, Value};
use minicbor::Encoder;

#[derive(Clone, Copy)]
enum Nesting {
    Lists,
    Maps,
}

fn nested_value(kind: Nesting, containers: usize) -> Value {
    let inner = (1..containers).fold(Value::Null, |value, _| match kind {
        Nesting::Lists => Value::List(vec![value]),
        Nesting::Maps => envelope(value),
    });
    envelope(inner)
}

fn nested_bytes(kind: Nesting, containers: usize) -> TestResult<Vec<u8>> {
    let mut encoder = Encoder::new(Vec::new());
    encoder.map(1)?.str("value")?;
    (1..containers).try_for_each(|_| -> TestResult {
        match kind {
            Nesting::Lists => {
                encoder.array(1)?;
            }
            Nesting::Maps => {
                encoder.map(1)?.str("value")?;
            }
        }
        Ok(())
    })?;
    encoder.null()?;
    Ok(encoder.into_writer())
}

#[test]
fn thirty_two_containers_including_the_root_map_encode_and_read_exactly() -> TestResult {
    // Given both list and map chains with exactly 32 containers including root.
    assert_eq!(MAX_DEPTH, 32);
    [Nesting::Lists, Nesting::Maps]
        .into_iter()
        .try_for_each(|kind| -> TestResult {
            let value = nested_value(kind, 32);
            // When encoded and separately validated from independent library bytes.
            let encoded = Record::encode(&value)?;
            let decoded = Record::from_bytes(nested_bytes(kind, 32)?)?;
            // Then both complete trees and bytes agree, without an off-by-one limit.
            assert_eq!(owned(&encoded)?, value);
            assert_eq!(owned(&decoded)?, value);
            assert_eq!(encoded.as_bytes(), decoded.as_bytes());
            Ok(())
        })
}

#[test]
fn thirty_three_containers_are_rejected_by_encode_and_owned_conversion() -> TestResult {
    // Given list and map chains one container past the inclusive limit.
    [Nesting::Lists, Nesting::Maps]
        .into_iter()
        .try_for_each(|kind| -> TestResult {
            let value = nested_value(kind, 33);
            let bytes = nested_bytes(kind, 33)?;
            // When encoding a tree or reading a well-formed raw record.
            let encoded = Record::encode(&value);
            let decoded = Record::from_bytes(bytes)?;
            // Then structural validation accepts CBOR, but both depth-aware paths refuse.
            assert_eq!(encoded, Err(RecordError::DepthLimit));
            assert_eq!(owned(&decoded), Err(RecordError::DepthLimit));
            Ok(())
        })
}

fn exact_payload_length() -> TestResult<usize> {
    // This probe and the final payload both use CBOR's four-byte length argument.
    let probe_length = 65_536usize;
    let probe = Record::encode(&envelope(Value::Bytes(vec![0; probe_length])))?;
    let overhead = probe
        .as_bytes()
        .len()
        .checked_sub(probe_length)
        .ok_or("encoded length cannot be smaller than the payload")?;
    MAX_RECORD_BYTES
        .checked_sub(overhead)
        .ok_or_else(|| "header exceeds record limit".into())
}

fn byte_record(payload_length: usize) -> TestResult<Vec<u8>> {
    let mut encoder = Encoder::new(Vec::new());
    encoder
        .map(1)?
        .str("value")?
        .bytes(&vec![0xA5; payload_length])?;
    Ok(encoder.into_writer())
}

#[test]
fn exactly_sixteen_mebibytes_are_accepted_by_encoding_and_byte_validation() -> TestResult {
    // Given a payload computed from actual encoded header overhead.
    assert_eq!(MAX_RECORD_BYTES, 16_777_216);
    let payload_length = exact_payload_length()?;
    let value = envelope(Value::Bytes(vec![0xA5; payload_length]));
    // When both public constructors process the exact-size record.
    let encoded = Record::encode(&value)?;
    let decoded = Record::from_bytes(byte_record(payload_length)?)?;
    // Then the boundary is inclusive and all payload bytes are preserved.
    assert_eq!(encoded.as_bytes().len(), MAX_RECORD_BYTES);
    assert_eq!(decoded.as_bytes().len(), MAX_RECORD_BYTES);
    assert_eq!(encoded.as_bytes(), decoded.as_bytes());
    assert_eq!(owned(&decoded)?, value);
    Ok(())
}

#[test]
fn one_byte_over_sixteen_mebibytes_is_rejected_by_both_constructors() -> TestResult {
    // Given the exact-size payload plus one byte, retaining the same CBOR header width.
    let payload_length = exact_payload_length()?
        .checked_add(1)
        .ok_or("payload overflow")?;
    let value = envelope(Value::Bytes(vec![0xA5; payload_length]));
    let bytes = byte_record(payload_length)?;
    // When both constructors see the oversize record.
    let encoded = Record::encode(&value);
    let decoded = Record::from_bytes(bytes.clone());
    // Then both name the size error, and the fixture is exactly one byte over.
    assert_eq!(
        bytes.len(),
        MAX_RECORD_BYTES.checked_add(1).ok_or("limit overflow")?
    );
    assert_eq!(encoded, Err(RecordError::SizeLimit));
    assert_eq!(decoded, Err(RecordError::SizeLimit));
    Ok(())
}
