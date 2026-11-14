use crate::request::{EncodeError, Protocol, common::BoundedBody};
use inventory_core::record::FieldId;

pub(super) fn write_big_integer(
    protocol: Protocol,
    value: i64,
    body: &mut BoundedBody,
) -> Result<(), EncodeError> {
    if protocol != Protocol::RpcV2Cbor {
        return super::json::integer(body, value);
    }
    let negative = value < 0;
    let argument = if negative {
        value.unsigned_abs().saturating_sub(1)
    } else {
        u64::try_from(value).map_err(|_| EncodeError::Limit("bigInteger"))?
    };
    let bytes = argument.to_be_bytes();
    let start = bytes.iter().position(|byte| *byte != 0).unwrap_or(7);
    super::cbor::tag(body, if negative { 3 } else { 2 })?;
    super::cbor::bytes(body, bytes.get(start..).ok_or(EncodeError::Limit("bigInteger"))?)
}

pub(super) fn write_big_decimal(
    protocol: Protocol,
    value: &str,
    field: Option<FieldId>,
    body: &mut BoundedBody,
) -> Result<(), EncodeError> {
    if protocol != Protocol::RpcV2Cbor {
        return super::json::number(body, value, field);
    }
    super::cbor::tag(body, 4)?;
    super::cbor::array_two(body)?;
    let bytes = value.as_bytes();
    let (negative, exponent, integer, fraction) = scan_decimal(bytes, field)?;
    super::cbor::signed(body, exponent)?;
    let mut coefficient = decimal_coefficient(bytes, integer, fraction, field, body.remaining()?)?;
    write_coefficient(body, negative, &mut coefficient)
}


fn scan_decimal(
    bytes: &[u8],
    field: Option<FieldId>,
) -> Result<(bool, i64, (usize, usize), (usize, usize)), EncodeError> {
    let negative = bytes.first() == Some(&b'-');
    let mut cursor = usize::from(negative);
    let integer_start = cursor;
    let integer_stop = integer_end(bytes, cursor, field)?;
    cursor = integer_stop;
    let mut fraction_start = cursor;
    let mut fraction_stop = cursor;
    if bytes.get(cursor) == Some(&b'.') {
        fraction_start = cursor.checked_add(1).ok_or_else(|| invalid(field))?;
        fraction_stop = fraction_end(bytes, fraction_start, field)?;
        cursor = fraction_stop;
    }
    let explicit_exponent = parse_exponent(bytes, &mut cursor, field)?;
    if cursor != bytes.len() { return Err(invalid(field)); }
    let fraction_length = i64::try_from(fraction_stop.checked_sub(fraction_start).ok_or_else(|| invalid(field))?)
        .map_err(|_| invalid(field))?;
    let exponent = explicit_exponent.checked_sub(fraction_length).ok_or_else(|| invalid(field))?;
    Ok((negative, exponent, (integer_start, integer_stop), (fraction_start, fraction_stop)))
}

fn decimal_coefficient(
    bytes: &[u8],
    integer: (usize, usize),
    fraction: (usize, usize),
    field: Option<FieldId>,
    budget: usize,
) -> Result<Vec<u8>, EncodeError> {
    if budget == 0 { return Err(EncodeError::Limit("body")); }
    let mut coefficient = Vec::new();
    coefficient.try_reserve_exact(1)?;
    coefficient.push(0);
    let integer = bytes.get(integer.0..integer.1).ok_or_else(|| invalid(field))?;
    let fraction = bytes.get(fraction.0..fraction.1).ok_or_else(|| invalid(field))?;
    check_decimal_growth(integer, fraction, budget)?;
    for digit in integer.iter().chain(fraction) {
        multiply_add_decimal(&mut coefficient, digit.checked_sub(b'0').ok_or_else(|| invalid(field))?, budget)?;
    }
    coefficient.reverse();
    trim_leading_zeroes(&mut coefficient);
    Ok(coefficient)
}

fn check_decimal_growth(integer: &[u8], fraction: &[u8], budget: usize) -> Result<(), EncodeError> {
    let digits = integer.len().checked_add(fraction.len()).ok_or(EncodeError::Limit("body"))?;
    let significant = integer.iter().chain(fraction).position(|digit| *digit != b'0')
        .map(|index| digits.saturating_sub(index)).unwrap_or(1);
    let digit_limit = budget.checked_mul(3).and_then(|limit| limit.checked_add(1)).unwrap_or(usize::MAX);
    if significant > digit_limit { return Err(EncodeError::Limit("body")); }
    Ok(())
}
fn integer_end(bytes: &[u8], start: usize, field: Option<FieldId>) -> Result<usize, EncodeError> {
    match bytes.get(start) {
        Some(b'0') => Ok(start.checked_add(1).ok_or_else(|| invalid(field))?),
        Some(b'1'..=b'9') => digit_end(bytes, start),
        _ => Err(invalid(field)),
    }
}

fn fraction_end(bytes: &[u8], start: usize, field: Option<FieldId>) -> Result<usize, EncodeError> {
    let end = digit_end(bytes, start)?;
    if end == start { Err(invalid(field)) } else { Ok(end) }
}

fn digit_end(bytes: &[u8], start: usize) -> Result<usize, EncodeError> {
    let mut cursor = start;
    while bytes.get(cursor).is_some_and(u8::is_ascii_digit) {
        cursor = cursor.checked_add(1).ok_or(EncodeError::Limit("bigDecimal"))?;
    }
    Ok(cursor)
}

fn parse_exponent(
    bytes: &[u8],
    cursor: &mut usize,
    field: Option<FieldId>,
) -> Result<i64, EncodeError> {
    if !matches!(bytes.get(*cursor), Some(b'e' | b'E')) {
        return Ok(0);
    }
    *cursor = cursor.checked_add(1).ok_or_else(|| invalid(field))?;
    let negative = match bytes.get(*cursor) {
        Some(b'-') => { *cursor = cursor.checked_add(1).ok_or_else(|| invalid(field))?; true }
        Some(b'+') => { *cursor = cursor.checked_add(1).ok_or_else(|| invalid(field))?; false }
        _ => false,
    };
    let start = *cursor;
    *cursor = digit_end(bytes, *cursor)?;
    if *cursor == start { return Err(invalid(field)); }
    let mut exponent = 0_u64;
    for digit in bytes.get(start..*cursor).ok_or_else(|| invalid(field))? {
        exponent = exponent.checked_mul(10)
            .and_then(|value| value.checked_add(u64::from(digit.checked_sub(b'0')?)))
            .ok_or_else(|| invalid(field))?;
    }
    signed_exponent(exponent, negative, field)
}

fn signed_exponent(exponent: u64, negative: bool, field: Option<FieldId>) -> Result<i64, EncodeError> {
    if negative && exponent == i64::MAX.unsigned_abs().saturating_add(1) { return Ok(i64::MIN); }
    let magnitude = i64::try_from(exponent).map_err(|_| invalid(field))?;
    if negative { magnitude.checked_neg().ok_or_else(|| invalid(field)) } else { Ok(magnitude) }
}

fn multiply_add_decimal(bytes: &mut Vec<u8>, digit: u8, budget: usize) -> Result<(), EncodeError> {
    if bytes.len() > budget { return Err(EncodeError::Limit("body")); }
    let mut carry = u16::from(digit);
    for byte in bytes.iter_mut() {
        let value = u16::from(*byte).checked_mul(10).and_then(|value| value.checked_add(carry))
            .ok_or(EncodeError::Limit("bigDecimal"))?;
        *byte = u8::try_from(value % 256).map_err(|_| EncodeError::Limit("bigDecimal"))?;
        carry = value / 256;
    }
    while carry != 0 {
        if bytes.len() >= budget { return Err(EncodeError::Limit("body")); }
        bytes.try_reserve(1)?;
        bytes.push(u8::try_from(carry % 256).map_err(|_| EncodeError::Limit("bigDecimal"))?);
        carry /= 256;
    }
    Ok(())
}

fn trim_leading_zeroes(bytes: &mut Vec<u8>) {
    let leading = bytes.iter().position(|byte| *byte != 0).unwrap_or(bytes.len().saturating_sub(1));
    if leading != 0 { bytes.drain(..leading); }
}

fn write_coefficient(body: &mut BoundedBody, negative: bool, magnitude: &mut [u8]) -> Result<(), EncodeError> {
    let value = bytes_u64(magnitude);
    if let Some(magnitude) = value {
        if magnitude == 0 { return super::cbor::signed(body, 0); }
        if negative {
            return super::cbor::negative(body, magnitude.checked_sub(1).ok_or(EncodeError::Limit("bigDecimal"))?);
        }
        return super::cbor::unsigned(body, magnitude);
    }
    if negative {
        decrement_be(magnitude)?;
        if let Some(argument) = bytes_u64(magnitude) {
            return super::cbor::negative(body, argument);
        }
    }
    super::cbor::tag(body, if negative { 3 } else { 2 })?;
    super::cbor::bytes(body, magnitude)
}

fn bytes_u64(bytes: &[u8]) -> Option<u64> {
    if bytes.len() > 8 { return None; }
    bytes.iter().try_fold(0_u64, |value, byte| value.checked_mul(256)?.checked_add(u64::from(*byte)))
}

fn decrement_be(bytes: &mut [u8]) -> Result<(), EncodeError> {
    for byte in bytes.iter_mut().rev() {
        if *byte != 0 { *byte = byte.checked_sub(1).ok_or(EncodeError::Limit("bigDecimal"))?; return Ok(()); }
        *byte = u8::MAX;
    }
    Err(EncodeError::Limit("bigDecimal"))
}

fn invalid(field: Option<FieldId>) -> EncodeError {
    EncodeError::InvalidValue { field, reason: "invalid bigDecimal" }
}
