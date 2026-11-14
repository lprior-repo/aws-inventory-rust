use super::{EncodeError, RequestContext};
use base64::Engine;
use http::{HeaderMap, HeaderValue};
use md5::{Digest, Md5};

mod glacier;

pub(super) fn apply(
    context: RequestContext<'_>,
    headers: &mut HeaderMap,
    body: &[u8],
) -> Result<(), EncodeError> {
    if context.service.endpoint_prefix()? == Some("glacier") {
        return glacier::apply(context, headers, body);
    }
    if context.operation.checksum_required()? {
        required_md5(headers, body)?;
    }
    Ok(())
}

fn required_md5(headers: &mut HeaderMap, body: &[u8]) -> Result<(), EncodeError> {
    if headers.contains_key("content-md5") {
        return Ok(());
    }
    let mut encoded = [0u8; 24];
    base64::engine::general_purpose::STANDARD
        .encode_slice(Md5::digest(body), &mut encoded)
        .map_err(|_| EncodeError::Limit("checksum header"))?;
    headers.insert("content-md5", HeaderValue::from_bytes(&encoded)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::required_md5;
    use http::{HeaderMap, HeaderValue};

    #[test]
    fn md5_covers_the_transmitted_bytes_including_empty_bodies()
    -> Result<(), Box<dyn std::error::Error>> {
        for (body, expected) in [
            (b"".as_slice(), "1B2M2Y8AsgTpgAmY7PhCfg=="),
            (b"hello world".as_slice(), "XrY7u+Ae7tCTyyK7j1rNww=="),
        ] {
            let mut headers = HeaderMap::new();
            required_md5(&mut headers, body)?;
            assert_eq!(
                headers.get("content-md5"),
                Some(&HeaderValue::from_static(expected))
            );
        }
        Ok(())
    }

    #[test]
    fn explicit_checksum_is_not_overwritten() -> Result<(), Box<dyn std::error::Error>> {
        let mut headers = HeaderMap::new();
        let expected = HeaderValue::from_static("1B2M2Y8AsgTpgAmY7PhCfg==");
        headers.insert("content-md5", expected.clone());
        required_md5(&mut headers, b"different body")?;
        assert_eq!(headers.get("content-md5"), Some(&expected));
        Ok(())
    }
}
