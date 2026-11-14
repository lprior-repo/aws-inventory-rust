use crate::request::EncodeError;
use sha2::{Digest, Sha256};

pub(super) const CHUNK_BYTES: usize = 1_048_576;

pub(super) fn digest(body: &[u8]) -> [u8; 32] {
    Sha256::digest(body).into()
}

pub(super) fn hash(body: &[u8]) -> Result<[u8; 32], EncodeError> {
    if body.len() > inventory_core::record::MAX_RECORD_BYTES {
        return Err(EncodeError::Limit("body"));
    }
    if body.len() <= CHUNK_BYTES {
        return Ok(digest(body));
    }
    let mut levels = [None; 5];
    for chunk in body.chunks(CHUNK_BYTES) {
        push(&mut levels, digest(chunk))?;
    }
    let mut root = None;
    for left in levels.into_iter().flatten() {
        root = Some(match root {
            None => left,
            Some(right) => pair(&left, &right),
        });
    }
    root.ok_or(EncodeError::Limit("checksum tree"))
}

fn push(levels: &mut [Option<[u8; 32]>], mut node: [u8; 32]) -> Result<(), EncodeError> {
    for slot in levels {
        if let Some(left) = slot.take() {
            node = pair(&left, &node);
        } else {
            *slot = Some(node);
            return Ok(());
        }
    }
    Err(EncodeError::Limit("checksum tree"))
}

fn pair(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(left);
    hash.update(right);
    hash.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::super::hexadecimal;
    use super::{CHUNK_BYTES, hash};

    #[test]
    fn odd_chunk_tree_matches_the_independent_three_chunk_vector()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut body = Vec::with_capacity(2_097_153);
        body.resize(CHUNK_BYTES, b'a');
        body.resize(2_097_152, b'b');
        body.push(b'c');
        assert_eq!(
            hexadecimal(&hash(&body)?)?.to_str()?,
            "7cd5b47e2b2af0202f4824eaddb1fb39a4c97f30a69f019d093b5c6e3a190782"
        );
        Ok(())
    }

    #[test]
    fn empty_upload_uses_the_standard_sha256_digest() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(
            hexadecimal(&hash(b"")?)?.to_str()?,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        Ok(())
    }
}
