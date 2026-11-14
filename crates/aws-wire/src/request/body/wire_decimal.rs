use crate::request::{EncodeError, Protocol, common::BoundedBody};
use inventory_core::record::FieldId;

#[path = "wire_decimal_parse.rs"]
mod parse;
use self::parse::{DecimalScan, decimal_coefficient, scan_decimal};

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
    super::cbor::bytes(
        body,
        bytes.get(start..).ok_or(EncodeError::Limit("bigInteger"))?,
    )
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
    let DecimalScan {
        negative,
        exponent,
        integer,
        fraction,
    } = scan_decimal(bytes, field)?;
    write_exponent(body, exponent, field)?;
    let mut coefficient = decimal_coefficient(bytes, integer, fraction, field, body.remaining()?)?;
    write_coefficient(body, negative, &mut coefficient)
}

fn write_exponent(
    body: &mut BoundedBody,
    exponent: i128,
    field: Option<FieldId>,
) -> Result<(), EncodeError> {
    if exponent >= 0 {
        let argument = u64::try_from(exponent).map_err(|_| invalid(field))?;
        return super::cbor::unsigned(body, argument);
    }
    let argument = exponent
        .checked_neg()
        .and_then(|value| value.checked_sub(1))
        .and_then(|value| u64::try_from(value).ok())
        .ok_or_else(|| invalid(field))?;
    super::cbor::negative(body, argument)
}

fn write_coefficient(
    body: &mut BoundedBody,
    negative: bool,
    magnitude: &mut [u8],
) -> Result<(), EncodeError> {
    let value = bytes_u64(magnitude);
    if let Some(magnitude) = value {
        if magnitude == 0 {
            return super::cbor::signed(body, 0);
        }
        if negative {
            return super::cbor::negative(
                body,
                magnitude
                    .checked_sub(1)
                    .ok_or(EncodeError::Limit("bigDecimal"))?,
            );
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
    if bytes.len() > 8 {
        return None;
    }
    bytes.iter().try_fold(0_u64, |value, byte| {
        value.checked_mul(256)?.checked_add(u64::from(*byte))
    })
}

fn decrement_be(bytes: &mut [u8]) -> Result<(), EncodeError> {
    for byte in bytes.iter_mut().rev() {
        if *byte != 0 {
            *byte = byte
                .checked_sub(1)
                .ok_or(EncodeError::Limit("bigDecimal"))?;
            return Ok(());
        }
        *byte = u8::MAX;
    }
    Err(EncodeError::Limit("bigDecimal"))
}

fn invalid(field: Option<FieldId>) -> EncodeError {
    EncodeError::InvalidValue {
        field,
        reason: "invalid bigDecimal",
    }
}
