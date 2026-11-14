use crate::request::{EncodeError, Protocol, common::BoundedBody};
use inventory_core::record::FieldId;
use super::wire_time_format::{write_datetime, write_http_date};
fn integer_seconds_as_f64(value: i64) -> Result<f64, EncodeError> {
    if value == 0 {
        return Ok(0.0);
    }
    let negative = value < 0;
    let magnitude = value.unsigned_abs();
    let bit_length = u64::BITS.checked_sub(magnitude.leading_zeros()).ok_or(EncodeError::Limit("timestamp"))?;
    let (significand, exponent) = seconds_significand(magnitude, bit_length)?;
    let sign = if negative { 1_u64.checked_shl(63).ok_or(EncodeError::Limit("timestamp"))? } else { 0 };
    let exponent_bits = exponent.checked_add(1023).ok_or(EncodeError::Limit("timestamp"))?;
    let exponent_bits = u64::from(exponent_bits).checked_shl(52).ok_or(EncodeError::Limit("timestamp"))?;
    let fraction = significand & 0x000f_ffff_ffff_ffff;
    Ok(f64::from_bits(sign | exponent_bits | fraction))
}

fn seconds_significand(magnitude: u64, bit_length: u32) -> Result<(u64, u32), EncodeError> {
    if bit_length <= 53 {
        let shift = 53_u32.checked_sub(bit_length).ok_or(EncodeError::Limit("timestamp"))?;
        return Ok((
            magnitude.checked_shl(shift).ok_or(EncodeError::Limit("timestamp"))?,
            bit_length.checked_sub(1).ok_or(EncodeError::Limit("timestamp"))?,
        ));
    }
    rounded_seconds_significand(magnitude, bit_length)
}

fn rounded_seconds_significand(magnitude: u64, bit_length: u32) -> Result<(u64, u32), EncodeError> {
    let shift = bit_length.checked_sub(53).ok_or(EncodeError::Limit("timestamp"))?;
    let mut rounded = magnitude.checked_shr(shift).ok_or(EncodeError::Limit("timestamp"))?;
    let remainder_mask = 1_u64.checked_shl(shift).and_then(|value| value.checked_sub(1))
        .ok_or(EncodeError::Limit("timestamp"))?;
    let remainder = magnitude & remainder_mask;
    let halfway = 1_u64.checked_shl(shift.checked_sub(1).ok_or(EncodeError::Limit("timestamp"))?)
        .ok_or(EncodeError::Limit("timestamp"))?;
    if remainder > halfway || (remainder == halfway && rounded & 1 == 1) {
        rounded = rounded.checked_add(1).ok_or(EncodeError::Limit("timestamp"))?;
    }
    normalize_rounded_seconds(rounded, bit_length)
}

fn normalize_rounded_seconds(rounded: u64, bit_length: u32) -> Result<(u64, u32), EncodeError> {
    if rounded == 1_u64.checked_shl(53).ok_or(EncodeError::Limit("timestamp"))? {
        Ok((rounded.checked_shr(1).ok_or(EncodeError::Limit("timestamp"))?, bit_length))
    } else {
        Ok((rounded, bit_length.checked_sub(1).ok_or(EncodeError::Limit("timestamp"))?))
    }
}
pub(super) fn write_time(
    protocol: Protocol,
    value: inventory_core::record::Time,
    format: Option<&str>,
    field: Option<FieldId>,
    body: &mut BoundedBody,
) -> Result<(), EncodeError> {
    if protocol == Protocol::RpcV2Cbor {
        return write_rpc_time(value, body);
    }
    match format.unwrap_or("epoch-seconds") {
        "epoch-seconds" => write_epoch_seconds(value.seconds(), value.nanoseconds(), body),
        "date-time" => write_datetime(value, field, body),
        "http-date" => write_http_date(value, field, body),
        _ => Err(EncodeError::InvalidValue {
            field,
            reason: "unknown timestamp format",
        }),
    }
}

fn write_rpc_time(
    value: inventory_core::record::Time,
    body: &mut BoundedBody,
) -> Result<(), EncodeError> {
    super::cbor::tag(body, 1)?;
    if value.nanoseconds() == 0 {
        return super::cbor::signed(body, value.seconds());
    }
    let seconds = integer_seconds_as_f64(value.seconds())?
        + f64::from(value.nanoseconds()) / 1_000_000_000.0;
    super::cbor::float64(body, seconds)
}

fn write_epoch_seconds(seconds: i64, nanos: u32, body: &mut BoundedBody) -> Result<(), EncodeError> {
    let (negative, whole, fraction) = normalize_epoch(seconds, nanos)?;
    if negative {
        body.write(b"-")?;
    }
    write_unsigned(body, whole)?;
    write_fraction(body, fraction)
}

fn normalize_epoch(seconds: i64, nanos: u32) -> Result<(bool, u64, u32), EncodeError> {
    let negative = seconds < 0;
    let mut whole = seconds.unsigned_abs();
    let mut fraction = nanos;
    if negative && nanos != 0 {
        whole = whole.checked_sub(1).ok_or(EncodeError::Limit("timestamp"))?;
        fraction = 1_000_000_000_u32.checked_sub(nanos).ok_or(EncodeError::Limit("timestamp"))?;
    }
    Ok((negative, whole, fraction))
}

fn write_fraction(body: &mut BoundedBody, value: u32) -> Result<(), EncodeError> {
    if value == 0 {
        return Ok(());
    }
    let digits = fraction_digits(value)?;
    let length = fraction_length(&digits)?;
    body.write(b".")?;
    body.write(digits.get(..length).ok_or(EncodeError::Limit("timestamp"))?)
}

pub(super) fn fraction_digits(value: u32) -> Result<[u8; 9], EncodeError> {
    let mut digits = [b'0'; 9];
    let mut remaining = value;
    for digit in digits.iter_mut().rev() {
        *digit = b'0'.checked_add(u8::try_from(remaining.checked_rem(10).ok_or(EncodeError::Limit("timestamp"))?)
            .map_err(|_| EncodeError::Limit("timestamp"))?)
            .ok_or(EncodeError::Limit("timestamp"))?;
        remaining = remaining.checked_div(10).ok_or(EncodeError::Limit("timestamp"))?;
    }
    Ok(digits)
}

pub(super) fn fraction_length(digits: &[u8; 9]) -> Result<usize, EncodeError> {
    digits.iter().rposition(|digit| *digit != b'0')
        .and_then(|index| index.checked_add(1))
        .ok_or(EncodeError::Limit("timestamp"))
}

fn write_unsigned(body: &mut BoundedBody, value: u64) -> Result<(), EncodeError> {
    let mut digits = [0_u8; 20];
    let mut cursor = digits.len();
    let mut remaining = value;
    loop {
        cursor = cursor.checked_sub(1).ok_or(EncodeError::Limit("timestamp"))?;
        let digit = b'0'.checked_add(u8::try_from(remaining.checked_rem(10).ok_or(EncodeError::Limit("timestamp"))?)
            .map_err(|_| EncodeError::Limit("timestamp"))?)
            .ok_or(EncodeError::Limit("timestamp"))?;
        *digits.get_mut(cursor).ok_or(EncodeError::Limit("timestamp"))? = digit;
        remaining = remaining.checked_div(10).ok_or(EncodeError::Limit("timestamp"))?;
        if remaining == 0 {
            break;
        }
    }
    body.write(digits.get(cursor..).ok_or(EncodeError::Limit("timestamp"))?)
}



