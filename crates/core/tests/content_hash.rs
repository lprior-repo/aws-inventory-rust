use inventory_core::record::*;

#[path = "content_hash/boundaries.rs"]
mod boundaries;
#[path = "content_hash/changes.rs"]
mod changes;
#[path = "content_hash/exclusions.rs"]
mod exclusions;
#[path = "content_hash/map_keys.rs"]
mod map_keys;
#[path = "content_hash/properties.rs"]
mod properties;
#[path = "content_hash/support.rs"]
mod support;

use support::{TestResult, expected, fields, hash};

const WORKED: [u8; 14] = [
    0xA4, 1, 0x20, 2, 0x62, b'h', b'i', 3, 0x82, 0xF5, 0xF6, 0x61, b'k', 0,
];

#[test]
fn worked_record_matches_hand_written_hash_input() -> TestResult {
    let record = Record::from_bytes(WORKED.to_vec())?;
    let input = [
        0x01, 0xBF, 0x01, 0x20, 0x02, 0x62, 0x68, 0x69, 0x03, 0x9F, 0xF5, 0xF6, 0xFF, 0x61, 0x6B,
        0x00, 0xFF,
    ];

    let actual = record.content_hash(&Exclusions::none())?;

    assert_eq!(FORMAT_VERSION, 1);
    assert_eq!(actual, expected(&input));
    Ok(())
}

#[test]
fn empty_record_includes_version_and_map_delimiters() -> TestResult {
    let value = fields([])?;

    assert_eq!(hash(&value)?, expected(&[0x01, 0xBF, 0xFF]));
    Ok(())
}

#[test]
fn scalar_forms_match_hand_written_hash_input() -> TestResult {
    let value = fields([
        (1, Value::Float(Float::new(1.5))),
        (2, Value::Bytes(vec![0xAA, 0x00])),
        (3, Value::Time(Time::new(-1, 2)?)),
        (4, Value::Bool(false)),
    ])?;
    let input = [
        0x01, 0xBF, 0x01, 0xFB, 0x3F, 0xF8, 0, 0, 0, 0, 0, 0, 0x02, 0x42, 0xAA, 0x00, 0x03, 0xC1,
        0x82, 0x20, 0x02, 0x04, 0xF4, 0xFF,
    ];

    assert_eq!(hash(&value)?, expected(&input));
    Ok(())
}

#[test]
fn stored_hash_bytes_roundtrip_without_change() -> TestResult {
    let bytes = [0xA5; 32];

    assert_eq!(ContentHash::from_bytes(bytes).as_bytes(), &bytes);
    Ok(())
}
