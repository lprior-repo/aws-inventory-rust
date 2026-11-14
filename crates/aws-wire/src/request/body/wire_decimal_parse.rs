use super::invalid;
use crate::request::EncodeError;
use inventory_core::record::FieldId;

pub(super) struct DecimalScan {
    pub(super) negative: bool,
    pub(super) exponent: i128,
    pub(super) integer: (usize, usize),
    pub(super) fraction: (usize, usize),
}

pub(super) fn scan_decimal(
    bytes: &[u8],
    field: Option<FieldId>,
) -> Result<DecimalScan, EncodeError> {
    let negative = bytes.first() == Some(&b'-');
    let integer_start = usize::from(negative);
    let integer_stop = integer_end(bytes, integer_start, field)?;
    let mut cursor = integer_stop;
    let fraction = fraction_range(bytes, &mut cursor, field)?;
    let explicit_exponent = parse_exponent(bytes, &mut cursor, field)?;
    if cursor != bytes.len() {
        return Err(invalid(field));
    }
    let exponent = scaled_exponent(explicit_exponent, fraction, field)?;
    Ok(DecimalScan {
        negative,
        exponent,
        integer: (integer_start, integer_stop),
        fraction,
    })
}

fn fraction_range(
    bytes: &[u8],
    cursor: &mut usize,
    field: Option<FieldId>,
) -> Result<(usize, usize), EncodeError> {
    if bytes.get(*cursor) != Some(&b'.') {
        return Ok((*cursor, *cursor));
    }
    let start = cursor.checked_add(1).ok_or_else(|| invalid(field))?;
    *cursor = fraction_end(bytes, start, field)?;
    Ok((start, *cursor))
}

fn scaled_exponent(
    exponent: i128,
    fraction: (usize, usize),
    field: Option<FieldId>,
) -> Result<i128, EncodeError> {
    let length = fraction
        .1
        .checked_sub(fraction.0)
        .ok_or_else(|| invalid(field))?;
    let length = i128::try_from(length).map_err(|_| invalid(field))?;
    exponent.checked_sub(length).ok_or_else(|| invalid(field))
}

pub(super) fn decimal_coefficient(
    bytes: &[u8],
    integer: (usize, usize),
    fraction: (usize, usize),
    field: Option<FieldId>,
    budget: usize,
) -> Result<Vec<u8>, EncodeError> {
    if budget == 0 {
        return Err(EncodeError::Limit("body"));
    }
    let mut coefficient = Vec::new();
    coefficient.try_reserve_exact(1)?;
    coefficient.push(0);
    let integer = bytes
        .get(integer.0..integer.1)
        .ok_or_else(|| invalid(field))?;
    let fraction = bytes
        .get(fraction.0..fraction.1)
        .ok_or_else(|| invalid(field))?;
    check_decimal_growth(integer, fraction, budget)?;
    for digit in integer.iter().chain(fraction) {
        multiply_add_decimal(
            &mut coefficient,
            digit.checked_sub(b'0').ok_or_else(|| invalid(field))?,
            budget,
        )?;
    }
    coefficient.reverse();
    trim_leading_zeroes(&mut coefficient);
    Ok(coefficient)
}

fn check_decimal_growth(integer: &[u8], fraction: &[u8], budget: usize) -> Result<(), EncodeError> {
    let digits = integer
        .len()
        .checked_add(fraction.len())
        .ok_or(EncodeError::Limit("body"))?;
    let significant = integer
        .iter()
        .chain(fraction)
        .position(|digit| *digit != b'0')
        .map(|index| digits.saturating_sub(index))
        .unwrap_or(1);
    let digit_limit = budget
        .checked_mul(3)
        .and_then(|limit| limit.checked_add(1))
        .unwrap_or(usize::MAX);
    if significant > digit_limit {
        return Err(EncodeError::Limit("body"));
    }
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
    if end == start {
        Err(invalid(field))
    } else {
        Ok(end)
    }
}

fn digit_end(bytes: &[u8], start: usize) -> Result<usize, EncodeError> {
    let mut cursor = start;
    while bytes.get(cursor).is_some_and(u8::is_ascii_digit) {
        cursor = cursor
            .checked_add(1)
            .ok_or(EncodeError::Limit("bigDecimal"))?;
    }
    Ok(cursor)
}

fn parse_exponent(
    bytes: &[u8],
    cursor: &mut usize,
    field: Option<FieldId>,
) -> Result<i128, EncodeError> {
    if !matches!(bytes.get(*cursor), Some(b'e' | b'E')) {
        return Ok(0);
    }
    *cursor = cursor.checked_add(1).ok_or_else(|| invalid(field))?;
    let negative = exponent_sign(bytes, cursor, field)?;
    let start = *cursor;
    *cursor = digit_end(bytes, *cursor)?;
    let exponent = exponent_magnitude(
        bytes.get(start..*cursor).ok_or_else(|| invalid(field))?,
        field,
    )?;
    if negative {
        exponent.checked_neg().ok_or_else(|| invalid(field))
    } else {
        Ok(exponent)
    }
}

fn exponent_sign(
    bytes: &[u8],
    cursor: &mut usize,
    field: Option<FieldId>,
) -> Result<bool, EncodeError> {
    let negative = bytes.get(*cursor) == Some(&b'-');
    if negative || bytes.get(*cursor) == Some(&b'+') {
        *cursor = cursor.checked_add(1).ok_or_else(|| invalid(field))?;
    }
    Ok(negative)
}

fn exponent_magnitude(bytes: &[u8], field: Option<FieldId>) -> Result<i128, EncodeError> {
    if bytes.is_empty() {
        return Err(invalid(field));
    }
    bytes.iter().try_fold(0_i128, |value, digit| {
        value
            .checked_mul(10)
            .and_then(|value| value.checked_add(i128::from(digit.checked_sub(b'0')?)))
            .ok_or_else(|| invalid(field))
    })
}

fn multiply_add_decimal(bytes: &mut Vec<u8>, digit: u8, budget: usize) -> Result<(), EncodeError> {
    if bytes.len() > budget {
        return Err(EncodeError::Limit("body"));
    }
    let mut carry = u16::from(digit);
    for byte in bytes.iter_mut() {
        let value = u16::from(*byte)
            .checked_mul(10)
            .and_then(|value| value.checked_add(carry))
            .ok_or(EncodeError::Limit("bigDecimal"))?;
        *byte = u8::try_from(value % 256).map_err(|_| EncodeError::Limit("bigDecimal"))?;
        carry = value / 256;
    }
    while carry != 0 {
        if bytes.len() >= budget {
            return Err(EncodeError::Limit("body"));
        }
        bytes.try_reserve(1)?;
        bytes.push(u8::try_from(carry % 256).map_err(|_| EncodeError::Limit("bigDecimal"))?);
        carry /= 256;
    }
    Ok(())
}

fn trim_leading_zeroes(bytes: &mut Vec<u8>) {
    let leading = bytes
        .iter()
        .position(|byte| *byte != 0)
        .unwrap_or(bytes.len().saturating_sub(1));
    if leading != 0 {
        bytes.drain(..leading);
    }
}
