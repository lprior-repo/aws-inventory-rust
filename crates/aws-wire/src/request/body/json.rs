use crate::request::{EncodeError, common::BoundedBody};

pub(crate) fn string(body: &mut BoundedBody, value: &str) -> Result<(), EncodeError> {
    body.write(b"\"")?;
    let mut start = 0;
    for (index, character) in value.char_indices() {
        let escaped = match character {
            '"' => Some(b"\\\"".as_slice()),
            '\\' => Some(b"\\\\".as_slice()),
            '\u{0008}' => Some(b"\\b".as_slice()),
            '\u{000C}' => Some(b"\\f".as_slice()),
            '\n' => Some(b"\\n".as_slice()),
            '\r' => Some(b"\\r".as_slice()),
            '\t' => Some(b"\\t".as_slice()),
            '\u{0000}'..='\u{001F}' => {
                body.write(
                    value
                        .get(start..index)
                        .ok_or(EncodeError::Limit("JSON string"))?
                        .as_bytes(),
                )?;
                let code = u32::from(character);
                let digits = b"0123456789abcdef";
                let escape = [
                    b'\\',
                    b'u',
                    b'0',
                    b'0',
                    *digits
                        .get(
                            usize::try_from(code >> 4)
                                .map_err(|_| EncodeError::Limit("JSON string"))?,
                        )
                        .ok_or(EncodeError::Limit("JSON string"))?,
                    *digits
                        .get(
                            usize::try_from(code & 0x0f)
                                .map_err(|_| EncodeError::Limit("JSON string"))?,
                        )
                        .ok_or(EncodeError::Limit("JSON string"))?,
                ];
                body.write(&escape)?;
                start = index
                    .checked_add(character.len_utf8())
                    .ok_or(EncodeError::Limit("JSON string"))?;
                None
            }
            _ => None,
        };
        if let Some(escape) = escaped {
            body.write(
                value
                    .get(start..index)
                    .ok_or(EncodeError::Limit("JSON string"))?
                    .as_bytes(),
            )?;
            body.write(escape)?;
            start = index
                .checked_add(character.len_utf8())
                .ok_or(EncodeError::Limit("JSON string"))?;
        }
    }
    body.write(
        value
            .get(start..)
            .ok_or(EncodeError::Limit("JSON string"))?
            .as_bytes(),
    )?;
    body.write(b"\"")
}

pub(crate) fn number(
    body: &mut BoundedBody,
    value: &str,
    field: Option<inventory_core::record::FieldId>,
) -> Result<(), EncodeError> {
    if valid_number(value.as_bytes()) {
        body.write(value.as_bytes())
    } else {
        Err(EncodeError::InvalidValue {
            field,
            reason: "invalid bigDecimal",
        })
    }
}

fn valid_number(bytes: &[u8]) -> bool {
    let mut cursor = usize::from(bytes.first() == Some(&b'-'));
    cursor = match bytes.get(cursor) {
        Some(b'0') => match cursor.checked_add(1) {
            Some(next) => next,
            None => return false,
        },
        Some(b'1'..=b'9') => decimal_digits(bytes, cursor),
        _ => return false,
    };
    if bytes.get(cursor) == Some(&b'.') && !fraction_digits(bytes, &mut cursor) {
        return false;
    }
    if matches!(bytes.get(cursor), Some(b'e' | b'E')) && !exponent_digits(bytes, &mut cursor) {
        return false;
    }
    cursor == bytes.len()
}

fn fraction_digits(bytes: &[u8], cursor: &mut usize) -> bool {
    if !advance(cursor) {
        return false;
    }
    let start = *cursor;
    *cursor = decimal_digits(bytes, *cursor);
    *cursor > start
}

fn exponent_digits(bytes: &[u8], cursor: &mut usize) -> bool {
    if !advance(cursor) {
        return false;
    }
    if matches!(bytes.get(*cursor), Some(b'+' | b'-')) && !advance(cursor) {
        return false;
    }
    let start = *cursor;
    *cursor = decimal_digits(bytes, *cursor);
    *cursor > start
}

fn advance(cursor: &mut usize) -> bool {
    match cursor.checked_add(1) {
        Some(next) => {
            *cursor = next;
            true
        }
        None => false,
    }
}

fn decimal_digits(bytes: &[u8], cursor: usize) -> usize {
    let Some(tail) = bytes.get(cursor..) else {
        return bytes.len();
    };
    let digits = tail
        .iter()
        .position(|byte| !byte.is_ascii_digit())
        .unwrap_or(tail.len());
    match cursor.checked_add(digits) {
        Some(end) => end,
        None => bytes.len(),
    }
}
pub(crate) fn integer(body: &mut BoundedBody, value: i64) -> Result<(), EncodeError> {
    let mut buffer = [0_u8; 20];
    let mut cursor = buffer.len();
    let negative = value < 0;
    let mut magnitude = value.unsigned_abs();
    loop {
        cursor = cursor
            .checked_sub(1)
            .ok_or(EncodeError::Limit("JSON integer"))?;
        let digit = b'0'
            .checked_add(
                u8::try_from(
                    magnitude
                        .checked_rem(10)
                        .ok_or(EncodeError::Limit("JSON integer"))?,
                )
                .map_err(|_| EncodeError::Limit("JSON integer"))?,
            )
            .ok_or(EncodeError::Limit("JSON integer"))?;
        *buffer
            .get_mut(cursor)
            .ok_or(EncodeError::Limit("JSON integer"))? = digit;
        magnitude = magnitude
            .checked_div(10)
            .ok_or(EncodeError::Limit("JSON integer"))?;
        if magnitude == 0 {
            break;
        }
    }
    if negative {
        cursor = cursor
            .checked_sub(1)
            .ok_or(EncodeError::Limit("JSON integer"))?;
        *buffer
            .get_mut(cursor)
            .ok_or(EncodeError::Limit("JSON integer"))? = b'-';
    }
    body.write(
        buffer
            .get(cursor..)
            .ok_or(EncodeError::Limit("JSON integer"))?,
    )
}

pub(crate) fn float(body: &mut BoundedBody, value: f64) -> Result<(), EncodeError> {
    if value.is_nan() {
        return string(body, "NaN");
    }
    if value == f64::INFINITY {
        return string(body, "Infinity");
    }
    if value == f64::NEG_INFINITY {
        return string(body, "-Infinity");
    }
    let mut buffer = FloatBuffer {
        bytes: [0; 64],
        length: 0,
    };
    std::fmt::write(&mut buffer, format_args!("{value}"))
        .map_err(|_| EncodeError::Limit("JSON float"))?;
    body.write(
        buffer
            .bytes
            .get(..buffer.length)
            .ok_or(EncodeError::Limit("JSON float"))?,
    )
}

pub(crate) fn base64(body: &mut BoundedBody, bytes: &[u8]) -> Result<(), EncodeError> {
    let (chunks, rest) = bytes.as_chunks::<3>();
    for &[first, second, third] in chunks {
        let encoded = [
            base64_digit(first >> 2)?,
            base64_digit((first & 0x03) << 4 | second >> 4)?,
            base64_digit((second & 0x0f) << 2 | third >> 6)?,
            base64_digit(third & 0x3f)?,
        ];
        body.write(&encoded)?;
    }
    if let Some(first) = rest.first().copied() {
        let second = rest.get(1).copied();
        let encoded = [
            base64_digit(first >> 2)?,
            base64_digit((first & 0x03) << 4 | second.unwrap_or(0) >> 4)?,
            second.map_or(Ok(b'='), |value| base64_digit((value & 0x0f) << 2))?,
            b'=',
        ];
        body.write(&encoded)?;
    }
    Ok(())
}
fn base64_digit(value: u8) -> Result<u8, EncodeError> {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    TABLE
        .get(usize::from(value))
        .copied()
        .ok_or(EncodeError::Limit("base64"))
}

struct FloatBuffer {
    bytes: [u8; 64],
    length: usize,
}

impl std::fmt::Write for FloatBuffer {
    fn write_str(&mut self, value: &str) -> std::fmt::Result {
        let end = self
            .length
            .checked_add(value.len())
            .ok_or(std::fmt::Error)?;
        let destination = self
            .bytes
            .get_mut(self.length..end)
            .ok_or(std::fmt::Error)?;
        destination.copy_from_slice(value.as_bytes());
        self.length = end;
        Ok(())
    }
}
