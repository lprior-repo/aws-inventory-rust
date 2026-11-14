use crate::request::{EncodeError, common::BoundedBody};

pub(super) fn pair(
    body: &mut BoundedBody,
    first: &mut bool,
    name: &str,
    value: &str,
) -> Result<(), EncodeError> {
    if !*first {
        body.write(b"&")?;
    }
    *first = false;
    component(body, name.as_bytes())?;
    body.write(b"=")?;
    component(body, value.as_bytes())
}
pub(super) fn pair_base64(
    body: &mut BoundedBody,
    first: &mut bool,
    name: &str,
    bytes: &[u8],
    limit: usize,
) -> Result<(), EncodeError> {
    let encoded_len = bytes.len().checked_add(2).ok_or(EncodeError::Limit("body"))?
        .checked_div(3).and_then(|count| count.checked_mul(4))
        .ok_or(EncodeError::Limit("body"))?;
    if encoded_len > limit {
        return Err(EncodeError::Limit("body"));
    }
    if !*first {
        body.write(b"&")?;
    }
    *first = false;
    component(body, name.as_bytes())?;
    body.write(b"=")?;
    let mut input = bytes.iter().copied();
    while let Some(a) = input.next() {
        let b = input.next();
        let c = input.next();
        let values = [
            BASE64[usize::from(a >> 2)],
            BASE64[usize::from(((a & 0x03) << 4) | b.unwrap_or(0) >> 4)],
            match b {
                Some(value) => BASE64[usize::from(((value & 0x0f) << 2) | c.unwrap_or(0) >> 6)],
                None => b'=',
            },
            match c {
                Some(value) => BASE64[usize::from(value & 0x3f)],
                None => b'=',
            },
        ];
        component(body, &values)?;
    }
    Ok(())
}

const BASE64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub(super) fn component(
    body: &mut BoundedBody,
    value: &[u8],
) -> Result<(), EncodeError> {
    value.iter().try_for_each(|byte| write_byte(body, *byte))
}

fn write_byte(body: &mut BoundedBody, byte: u8) -> Result<(), EncodeError> {
    if byte.is_ascii_alphanumeric() || matches!(byte, b'*' | b'-' | b'.' | b'_') {
        return body.write(&[byte]);
    }
    if byte == b' ' {
        return body.write(b"+");
    }
    let encoded = [b'%', hex(byte >> 4), hex(byte & 0x0f)];
    body.write(&encoded)
}

fn hex(value: u8) -> u8 {
    match value {
        0..=9 => b'0' + value,
        _ => b'A' + value - 10,
    }
}
#[cfg(test)]
mod tests {
    use super::{pair, pair_base64};
    use crate::request::{EncodeError, common::BoundedBody};

    #[test]
    fn pairs_use_form_encoding_and_preserve_insertion_order() {
        let mut body = BoundedBody::new(128);
        let mut first = true;
        let result = pair(&mut body, &mut first, "Action", "List +&");
        assert!(result.is_ok());
        let result = pair(&mut body, &mut first, "A~", "é");
        assert!(result.is_ok());
        assert_eq!(
            body.into_vec(),
            b"Action=List+%2B%26&A%7E=%C3%A9"
        );
    }

    #[test]
    fn blob_base64_is_percent_encoded_and_body_bounded() {
        let mut body = BoundedBody::new(64);
        let mut first = true;
        let result = pair_base64(&mut body, &mut first, "Blob", &[0xfb, 0xff], 64);
        assert!(result.is_ok());
        assert_eq!(body.into_vec(), b"Blob=%2B%2F8%3D");

        let mut body = BoundedBody::new(8);
        let mut first = true;
        let result = pair_base64(&mut body, &mut first, "B", &[1, 2, 3], 8);
        assert!(matches!(result, Err(EncodeError::Limit("body"))));
        assert!(body.into_vec().len() <= 8);
    }

    #[test]
    fn encoding_checks_limit_before_exceeding_it() {
        let mut body = BoundedBody::new(8);
        let mut first = true;
        let result = pair(&mut body, &mut first, "a", "longer");
        assert!(matches!(result, Err(crate::request::EncodeError::Limit("body"))));
        assert!(body.into_vec().len() <= 8);
    }
}
