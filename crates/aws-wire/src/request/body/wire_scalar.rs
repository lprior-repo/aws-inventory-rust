use crate::request::{EncodeError, Protocol, common::BoundedBody};
use inventory_core::record::FieldId;
pub(super) fn write_key(body: &mut BoundedBody, protocol: Protocol, key: &str) -> Result<(), EncodeError> {
    if protocol == Protocol::RpcV2Cbor {
        super::cbor::text(body, key)
    } else {
        super::json::string(body, key)
    }
}

pub(super) fn write_null(protocol: Protocol, body: &mut BoundedBody) -> Result<(), EncodeError> {
    if protocol == Protocol::RpcV2Cbor {
        super::cbor::null(body)
    } else {
        body.write(b"null")
    }
}

pub(super) fn write_bool(protocol: Protocol, value: bool, body: &mut BoundedBody) -> Result<(), EncodeError> {
    if protocol == Protocol::RpcV2Cbor {
        super::cbor::boolean(body, value)
    } else {
        body.write(if value { b"true" } else { b"false" })
    }
}

pub(super) fn write_integer(protocol: Protocol, value: i64, body: &mut BoundedBody) -> Result<(), EncodeError> {
    if protocol == Protocol::RpcV2Cbor {
        super::cbor::signed(body, value)
    } else {
        super::json::integer(body, value)
    }
}
pub(super) fn validate_integer(kind: &str, value: i64, field: Option<FieldId>) -> Result<(), EncodeError> {
    let valid = match kind {
        "byte" => i8::try_from(value).is_ok(),
        "short" => i16::try_from(value).is_ok(),
        "integer" | "intEnum" => i32::try_from(value).is_ok(),
        _ => true,
    };
    if valid {
        Ok(())
    } else {
        Err(EncodeError::InvalidValue { field, reason: "integer is out of modeled range" })
    }
}

pub(super) fn write_float(protocol: Protocol, kind: &str, value: f64, body: &mut BoundedBody) -> Result<(), EncodeError> {
    if protocol == Protocol::RpcV2Cbor {
        if kind == "float" {
            super::cbor::float32(body, narrow_f64_to_f32(value)?)
        } else {
            super::cbor::float64(body, value)
        }
    } else {
        super::json::float(body, value)
    }
}

fn narrow_f64_to_f32(value: f64) -> Result<f32, EncodeError> {
    let bits = value.to_bits();
    let sign = u32::try_from((bits >> 32) & 0x8000_0000).map_err(|_| EncodeError::Limit("float"))?;
    let raw_exponent = (bits >> 52) & 0x7ff;
    let fraction = bits & 0x000f_ffff_ffff_ffff;
    if raw_exponent == 0x7ff {
        return special_float(sign, fraction);
    }
    if raw_exponent == 0 {
        return Ok(f32::from_bits(sign));
    }
    let exponent = i32::try_from(raw_exponent).map_err(|_| EncodeError::Limit("float"))?
        .checked_sub(1023).ok_or(EncodeError::Limit("float"))?;
    narrow_normalized_float(sign, (1_u64 << 52) | fraction, exponent)
}

fn special_float(sign: u32, fraction: u64) -> Result<f32, EncodeError> {
    let payload = u32::try_from(fraction >> 29).map_err(|_| EncodeError::Limit("float"))?;
    let payload = if fraction == 0 { 0 } else if payload == 0 { 1 } else { payload };
    Ok(f32::from_bits(sign | 0x7f80_0000 | payload))
}

fn narrow_normalized_float(sign: u32, significand: u64, exponent: i32) -> Result<f32, EncodeError> {
    if exponent > 127 {
        return Ok(f32::from_bits(sign | 0x7f80_0000));
    }
    if exponent >= -126 {
        return narrow_normal_float(sign, significand, exponent);
    }
    narrow_subnormal_float(sign, significand, exponent)
}

fn narrow_normal_float(sign: u32, significand: u64, exponent: i32) -> Result<f32, EncodeError> {
    let mut rounded = round_right_even(significand, 29)?;
    let mut adjusted_exponent = exponent;
    if rounded == (1_u64 << 24) {
        rounded = rounded.checked_shr(1).ok_or(EncodeError::Limit("float"))?;
        adjusted_exponent = adjusted_exponent.checked_add(1).ok_or(EncodeError::Limit("float"))?;
    }
    if adjusted_exponent > 127 {
        return Ok(f32::from_bits(sign | 0x7f80_0000));
    }
    let exponent_bits = u32::try_from(adjusted_exponent.checked_add(127).ok_or(EncodeError::Limit("float"))?)
        .map_err(|_| EncodeError::Limit("float"))?;
    let fraction_bits = u32::try_from(rounded & 0x007f_ffff).map_err(|_| EncodeError::Limit("float"))?;
    Ok(f32::from_bits(sign | exponent_bits.checked_shl(23).ok_or(EncodeError::Limit("float"))? | fraction_bits))
}

fn narrow_subnormal_float(sign: u32, significand: u64, exponent: i32) -> Result<f32, EncodeError> {
    let shift = u32::try_from(exponent.checked_neg().and_then(|value| value.checked_sub(97)).ok_or(EncodeError::Limit("float"))?)
        .map_err(|_| EncodeError::Limit("float"))?;
    if shift > 53 {
        return Ok(f32::from_bits(sign));
    }
    let fraction_bits = u32::try_from(round_right_even(significand, shift)?)
        .map_err(|_| EncodeError::Limit("float"))?;
    Ok(f32::from_bits(sign | fraction_bits))
}

fn round_right_even(value: u64, shift: u32) -> Result<u64, EncodeError> {
    let quotient = value.checked_shr(shift).ok_or(EncodeError::Limit("float"))?;
    let mask = (1_u64.checked_shl(shift).ok_or(EncodeError::Limit("float"))?).checked_sub(1)
        .ok_or(EncodeError::Limit("float"))?;
    let remainder = value & mask;
    let halfway = 1_u64.checked_shl(shift.checked_sub(1).ok_or(EncodeError::Limit("float"))?)
        .ok_or(EncodeError::Limit("float"))?;
    if remainder > halfway || (remainder == halfway && quotient & 1 == 1) {
        quotient.checked_add(1).ok_or(EncodeError::Limit("float"))
    } else {
        Ok(quotient)
    }
}

pub(super) fn json_or_cbor_text(protocol: Protocol, value: &str, body: &mut BoundedBody) -> Result<(), EncodeError> {
    if protocol == Protocol::RpcV2Cbor {
        super::cbor::text(body, value)
    } else {
        super::json::string(body, value)
    }
}
