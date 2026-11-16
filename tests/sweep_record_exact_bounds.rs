use inventory_core::{
    record::{Float, Record, Time, Value, ValueRef, scalar_item, scalar_record},
};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

// Each envelope is independently specified CBOR: a one-entry map with the text
// key "$". Expected bytes never come from the scalar encoder under test.
fn scalar_cases() -> TestResult<Vec<(ValueRef<'static>, Vec<u8>)>> {
    Ok(vec![
        (ValueRef::Null, vec![0xa1, 0x61, 0x24, 0xf6]),
        (ValueRef::Bool(false), vec![0xa1, 0x61, 0x24, 0xf4]),
        (ValueRef::Bool(true), vec![0xa1, 0x61, 0x24, 0xf5]),
        (ValueRef::Int(i64::MIN), vec![0xa1, 0x61, 0x24, 0x3b, 0x7f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]),
        (ValueRef::Int(i64::MAX), vec![0xa1, 0x61, 0x24, 0x1b, 0x7f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]),
        (ValueRef::Int(0), vec![0xa1, 0x61, 0x24, 0x00]),
        (ValueRef::Text(""), vec![0xa1, 0x61, 0x24, 0x60]),
        (ValueRef::Text("é"), vec![0xa1, 0x61, 0x24, 0x62, 0xc3, 0xa9]),
        (ValueRef::Bytes(&[]), vec![0xa1, 0x61, 0x24, 0x40]),
        (ValueRef::Bytes(&[0, 255]), vec![0xa1, 0x61, 0x24, 0x42, 0, 255]),
        (ValueRef::Float(Float::new(0.5)), vec![0xa1, 0x61, 0x24, 0xfb, 0x3f, 0xe0, 0, 0, 0, 0, 0, 0]),
        (ValueRef::Time(Time::new(-1, 1)?), vec![0xa1, 0x61, 0x24, 0xc1, 0x82, 0x20, 0x01]),
    ])
}

#[test]
fn scalar_decoder_preserves_each_native_scalar_in_an_independently_encoded_envelope() -> TestResult {
    // Given independently encoded scalar envelopes, including both polarities,
    // signed extrema, null and scalar kinds that are not AWS identity strings.
    for (expected, bytes) in scalar_cases()? {
        let record = Record::from_bytes(bytes)?;
        // When the public selected-item decoder reads the envelope.
        let actual = scalar_item(record.root())?.map(|value| value.to_value()).transpose()?;
        // Then native type and value are exact, without paired-encoder oracles.
        assert_eq!(actual, Some(expected.to_value()?));
    }
    Ok(())
}

#[test]
fn scalar_encoder_emits_exactly_one_dollar_entry_with_independently_expected_bytes() -> TestResult {
    for (value, expected) in scalar_cases()? {
        // Given a native scalar; when wrapped by the public scalar encoder.
        let record = scalar_record(value)?;
        // Then the bytes and entry count match the independent envelope oracle.
        assert_eq!(record.as_bytes(), expected.as_slice());
        assert_eq!(record.root().iter()?.collect::<Result<Vec<_>, _>>()?.len(), 1);
    }
    Ok(())
}

#[test]
fn scalar_decoder_refuses_extra_entries_containers_and_non_dollar_structures() -> TestResult {
    // All are valid records, but none is a selected-scalar envelope. In
    // particular "$" comes first in the two-entry named map, exercising the
    // second-entry guard rather than rejection of the first key alone.
    let cases: &[(&str, &[u8])] = &[
        ("extra named entry", &[0xa2, 0x61, 0x24, 0x61, b'x', 0x61, b'z', 0x01]),
        ("extra ordinary field", &[0xa2, 0x01, 0x01, 0x61, 0x24, 0x61, b'x']),
        ("empty map payload", &[0xa1, 0x61, 0x24, 0xa0]),
        ("populated map payload", &[0xa1, 0x61, 0x24, 0xa1, 0x01, 0x01]),
        ("empty list payload", &[0xa1, 0x61, 0x24, 0x80]),
        ("populated list payload", &[0xa1, 0x61, 0x24, 0x81, 0xf6]),
        ("different named key", &[0xa1, 0x61, b'x', 0x61, b'a']),
        ("empty structure", &[0xa0]),
        ("ordinary field structure", &[0xa1, 0x01, 0x61, b'a']),
    ];
    for (name, bytes) in cases {
        // Given a hostile shape encoded without scalar_record.
        let record = Record::from_bytes(bytes.to_vec())?;
        // When decoded, then it is exactly absent, not a scalar or parse error.
        assert_eq!(scalar_item(record.root())?.map(|value| value.to_value()).transpose()?, None, "{name}");
    }
    Ok(())
}

// DEAD exact-bounds integration helpers deleted — wrappers in record.rs cover extraction; these standalone helpers never used, removed.
