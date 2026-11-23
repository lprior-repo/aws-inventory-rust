use super::*;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn epoch_preserves_nanoseconds_across_sign_exponents_and_second_boundaries() -> TestResult {
    for (wire, seconds, nanos) in [
        ("-0.1", -1, 900_000_000),
        ("1.2345678905", 1, 234_567_891),
        ("-1.2345678905", -2, 765_432_109),
        ("9.46845296123e8", 946_845_296, 123_000_000),
        ("1e-1000000", 0, 0),
        ("-1e-1000000", 0, 0),
        ("9223372036854775807.999999999", i64::MAX, 999_999_999),
        ("-9223372036854775808", i64::MIN, 0),
    ] {
        assert_eq!(parse(wire, None)?, Time::new(seconds, nanos)?, "{wire}");
    }
    Ok(())
}

#[test]
fn epoch_rejects_instants_outside_record_seconds_and_malformed_numbers() {
    for wire in [
        "9223372036854775808",
        "-9223372036854775808.000000001",
        "1e1000000",
        "1e",
        "1.2.3",
        "NaN",
        "Infinity",
    ] {
        assert!(parse(wire, None).is_err(), "{wire}");
    }
}

#[test]
fn binary_float_timestamp_uses_decimal_nanoseconds_without_precision_artifacts() -> TestResult {
    assert_eq!(
        from_float(946_845_296.123)?,
        Time::new(946_845_296, 123_000_000)?
    );
    assert_eq!(from_float(-0.1)?, Time::new(-1, 900_000_000)?);
    for wire in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(from_float(wire).is_err());
    }
    Ok(())
}

#[test]
fn given_numeric_timestamp_for_datetime_shape_when_decoded_then_invalid_emulator_value_is_rejected()
-> TestResult {
    let wire = "1795446406.337384223";
    assert!(parse(wire, Some(TimestampFormat::DateTime)).is_err());
    assert_eq!(
        parse(wire, Some(TimestampFormat::EpochSeconds))?,
        Time::new(1_795_446_406, 337_384_223)?
    );
    Ok(())
}
