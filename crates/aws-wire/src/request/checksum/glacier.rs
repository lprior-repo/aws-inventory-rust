use crate::request::{EncodeError, RequestContext};
use http::{HeaderMap, HeaderValue};

mod tree;

const FULL_HASH: &str = "x-amz-content-sha256";
const TREE_HASH: &str = "x-amz-sha256-tree-hash";

pub(super) fn apply(
    context: RequestContext<'_>,
    headers: &mut HeaderMap,
    body: &[u8],
) -> Result<(), EncodeError> {
    if !headers.contains_key("x-amz-glacier-version") {
        headers.insert(
            "x-amz-glacier-version",
            HeaderValue::from_str(context.service.version()?)?,
        );
    }
    if matches!(
        context.operation.name()?,
        "UploadArchive" | "UploadMultipartPart"
    ) {
        upload(headers, body)?;
    }
    Ok(())
}

fn upload(headers: &mut HeaderMap, body: &[u8]) -> Result<(), EncodeError> {
    let full = !headers.contains_key(FULL_HASH);
    let tree = !headers.contains_key(TREE_HASH);
    if full && tree && body.len() <= tree::CHUNK_BYTES {
        let value = hexadecimal(&tree::digest(body))?;
        headers.insert(FULL_HASH, value.clone());
        headers.insert(TREE_HASH, value);
        return Ok(());
    }
    if full {
        headers.insert(FULL_HASH, hexadecimal(&tree::digest(body))?);
    }
    if tree {
        headers.insert(TREE_HASH, hexadecimal(&tree::hash(body)?)?);
    }
    Ok(())
}

fn hexadecimal(digest: &[u8; 32]) -> Result<HeaderValue, EncodeError> {
    let mut output = [0u8; 64];
    let text = base16ct::lower::encode_str(digest, &mut output)
        .map_err(|_| EncodeError::Limit("checksum header"))?;
    Ok(HeaderValue::from_str(text)?)
}

#[cfg(test)]
mod tests {
    use super::{FULL_HASH, TREE_HASH, upload};
    use http::{HeaderMap, HeaderValue};

    #[test]
    fn uploads_hash_payload_and_preserve_explicit_tree_checksums()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut headers = HeaderMap::new();
        upload(&mut headers, b"hello world")?;
        let expected = HeaderValue::from_static(
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9",
        );
        assert_eq!(headers.get(FULL_HASH), Some(&expected));
        assert_eq!(headers.get(TREE_HASH), Some(&expected));
        upload(&mut headers, b"changed")?;
        assert_eq!(headers.get(FULL_HASH), Some(&expected));
        assert_eq!(headers.get(TREE_HASH), Some(&expected));
        Ok(())
    }
}
