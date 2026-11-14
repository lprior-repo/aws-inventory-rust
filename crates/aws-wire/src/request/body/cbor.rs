use crate::request::{EncodeError, common::BoundedBody};

pub(super) fn unsigned(body: &mut BoundedBody, value: u64) -> Result<(), EncodeError> {
    head(body, 0, value)
}

pub(super) fn signed(body: &mut BoundedBody, value: i64) -> Result<(), EncodeError> {
    if value >= 0 {
        unsigned(
            body,
            u64::try_from(value).map_err(|_| EncodeError::Limit("CBOR integer"))?,
        )
    } else {
        head(body, 1, value.unsigned_abs().saturating_sub(1))
    }
}

pub(super) fn text(body: &mut BoundedBody, value: &str) -> Result<(), EncodeError> {
    head(
        body,
        3,
        u64::try_from(value.len()).map_err(|_| EncodeError::Limit("CBOR text"))?,
    )?;
    body.write(value.as_bytes())
}

pub(super) fn bytes(body: &mut BoundedBody, value: &[u8]) -> Result<(), EncodeError> {
    head(
        body,
        2,
        u64::try_from(value.len()).map_err(|_| EncodeError::Limit("CBOR bytes"))?,
    )?;
    body.write(value)
}

pub(super) fn float32(body: &mut BoundedBody, value: f32) -> Result<(), EncodeError> {
    body.write(&[0xfa])?;
    body.write(&value.to_bits().to_be_bytes())
}

pub(super) fn float64(body: &mut BoundedBody, value: f64) -> Result<(), EncodeError> {
    body.write(&[0xfb])?;
    body.write(&value.to_bits().to_be_bytes())
}

pub(super) fn tag(body: &mut BoundedBody, value: u64) -> Result<(), EncodeError> {
    head(body, 6, value)
}

pub(super) fn negative(body: &mut BoundedBody, argument: u64) -> Result<(), EncodeError> {
    head(body, 1, argument)
}

pub(super) fn array_two(body: &mut BoundedBody) -> Result<(), EncodeError> {
    head(body, 4, 2)
}
pub(super) fn array_indefinite(body: &mut BoundedBody) -> Result<(), EncodeError> {
    body.write(&[0x9f])
}

pub(super) fn map_indefinite(body: &mut BoundedBody) -> Result<(), EncodeError> {
    body.write(&[0xbf])
}

pub(super) fn boolean(body: &mut BoundedBody, value: bool) -> Result<(), EncodeError> {
    body.write(&[if value { 0xf5 } else { 0xf4 }])
}

pub(super) fn null(body: &mut BoundedBody) -> Result<(), EncodeError> {
    body.write(&[0xf6])
}

fn head(body: &mut BoundedBody, major: u8, value: u64) -> Result<(), EncodeError> {
    let prefix = major << 5;
    match value {
        0..=23 => body.write(&[
            prefix | u8::try_from(value).map_err(|_| EncodeError::Limit("CBOR integer"))?
        ]),
        24..=0xff => body.write(&[
            prefix | 24,
            u8::try_from(value).map_err(|_| EncodeError::Limit("CBOR integer"))?,
        ]),
        0x100..=0xffff => {
            body.write(&[prefix | 25])?;
            body.write(
                &u16::try_from(value)
                    .map_err(|_| EncodeError::Limit("CBOR integer"))?
                    .to_be_bytes(),
            )
        }
        0x1_0000..=0xffff_ffff => {
            body.write(&[prefix | 26])?;
            body.write(
                &u32::try_from(value)
                    .map_err(|_| EncodeError::Limit("CBOR integer"))?
                    .to_be_bytes(),
            )
        }
        _ => {
            body.write(&[prefix | 27])?;
            body.write(&value.to_be_bytes())
        }
    }
}
