use super::{Record, RecordError, TestResult};
use crate::record::ValueRef;

#[test]
fn misplaced_breaks_never_complete_definite_items_or_unmatched_map_keys() {
    // Given breaks in places where a complete item is required.
    [
        vec![0xA1, 1, 0xFF],
        vec![0xBF, 1, 0xFF],
        vec![0xA1, 0xFF, 0],
        vec![0xA1, 1, 0x81, 0xFF],
        vec![0xA1, 1, 0xC1, 0xFF],
        vec![0xA1, 1, 0xBF, 1, 0xFF],
    ]
    .into_iter()
    .for_each(|bytes| {
        // When structurally validated, then each is malformed rather than accepted.
        assert!(matches!(
            Record::from_bytes(bytes),
            Err(RecordError::Malformed { .. })
        ));
    });
}

#[test]
fn legitimate_breaks_close_only_their_indefinite_containers() -> TestResult {
    // Given nested generic CBOR, including indefinite strings and container-valued keys.
    [
        vec![0xBF, 0xFF],
        vec![0xA1, 1, 0x9F, 0xA0, 0xFF],
        vec![0xBF, 0x9F, 0xFF, 0xBF, 0xFF, 0xFF],
        vec![0xA1, 1, 0x7F, 0x61, b'x', 0xFF],
        vec![0xA1, 1, 0x5F, 0x41, 0xFF, 0xFF],
    ]
    .into_iter()
    .try_for_each(|bytes| -> TestResult {
        // When validated, then every original byte survives, including embedded 0xFF data.
        assert_eq!(Record::from_bytes(bytes.clone())?.as_bytes(), bytes);
        Ok(())
    })
}

#[test]
fn generic_validation_accepts_deep_unsupported_items_without_recursive_descent() -> TestResult {
    // Given 1024 generic array levels and an unsupported float-width leaf.
    let mut encoder = minicbor::Encoder::new(Vec::new());
    encoder.map(1)?.u8(1)?;
    (0..1024).try_for_each(|_| encoder.array(1).map(|_| ()))?;
    encoder.f32(0.0)?;
    let bytes = encoder.into_writer();
    // When validated, then depth and supported-form checks remain deferred to readers.
    let record = Record::from_bytes(bytes.clone())?;
    assert_eq!(record.as_bytes(), bytes);
    assert_eq!(
        ValueRef::Map(record.root()).to_value(),
        Err(RecordError::DepthLimit)
    );
    Ok(())
}

#[test]
fn every_prefix_of_generic_indefinite_cbor_is_missing_an_item_or_terminator() -> TestResult {
    // Given valid indefinite maps, arrays, and tags.
    let bytes = [0xBF, 1, 0x9F, 0xC2, 0x40, 0xFF, 0xFF];
    (0..bytes.len()).try_for_each(|length| -> TestResult {
        // When any strict prefix is validated, then truncation cannot become a valid record.
        let prefix = bytes.get(..length).ok_or("prefix bounds")?;
        assert!(matches!(
            Record::from_bytes(prefix.to_vec()),
            Err(RecordError::Missing { .. })
        ));
        Ok(())
    })
}
