use super::BoxError;
use super::{assert_semantically_equal, decode};

#[test]
fn accepts_order_and_container_length_variations_and_float_widths() -> Result<(), BoxError> {
    assert_semantically_equal(
        &[0xa2, 0x61, b'a', 1, 0x61, b'b', 0xf9, 0x3e, 0x00],
        &[
            0xbf, 0x61, b'b', 0xfa, 0x3f, 0xc0, 0, 0, 0x61, b'a', 0x18, 1, 0xff,
        ],
        "equal CBOR",
    )
}

#[test]
fn rejects_changed_missing_extra_duplicate_tags_and_trailing_data() {
    let expected = [0xa1, 0x61, b'a', 1];
    for invalid in [
        &[0xa1, 0x61, b'a', 2][..],
        &[0x80][..],
        &[0xa2, 0x61, b'a', 1, 0x61, b'b', 2][..],
        &[0xa2, 0x61, b'a', 1, 0x61, b'a', 1][..],
        &[0xa1, 0x61, b'a', 0xc0, 1][..],
        &[0xa1, 0x61, b'a', 1, 0][..],
        &[0xbf, 0x61, b'a'][..],
    ] {
        assert!(assert_semantically_equal(invalid, &expected, "reject").is_err());
    }
}

#[test]
fn integers_remain_exact_and_nan_matches_nan() -> Result<(), BoxError> {
    assert_ne!(
        decode(&[0x1b, 0x00, 0x20, 0, 0, 0, 0, 0, 0], "int")?,
        decode(&[0x1b, 0x00, 0x20, 0, 0, 0, 0, 0, 1], "int")?,
    );
    assert_semantically_equal(
        &[0xf9, 0x7e, 0x00],
        &[0xfb, 0x7f, 0xf8, 0, 0, 0, 0, 0, 0],
        "nan",
    )
}

#[test]
fn timestamps_accept_only_exact_integer_float_equivalence() -> Result<(), BoxError> {
    assert_semantically_equal(&[0xc1, 1], &[0xc1, 0xf9, 0x3c, 0], "one second")?;
    assert_semantically_equal(&[0xc1, 0x20], &[0xc1, 0xf9, 0xbc, 0], "negative second")?;
    assert_semantically_equal(
        &[0xc1, 0x1b, 0x80, 0, 0, 0, 0, 0, 0, 0],
        &[0xc1, 0xfb, 0x43, 0xe0, 0, 0, 0, 0, 0, 0],
        "exact 2^63",
    )?;
    let one = decode(&[0xc1, 1], "one second")?;
    for different in [
        &[0xc1, 0xf9, 0x3e, 0][..],
        &[0xc1, 0xf9, 0x7c, 0][..],
        &[0xc0, 1][..],
        &[1][..],
    ] {
        assert_ne!(one, decode(different, "different timestamp")?);
    }
    Ok(())
}

#[test]
fn timestamp_float_rounding_does_not_erase_adjacent_large_integers() -> Result<(), BoxError> {
    assert_ne!(
        decode(&[0xc1, 0x1b, 0, 0x20, 0, 0, 0, 0, 0, 1], "2^53 + 1")?,
        decode(&[0xc1, 0xfb, 0x43, 0x40, 0, 0, 0, 0, 0, 0], "2^53 float")?,
    );
    Ok(())
}
