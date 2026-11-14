mod gzip;

use crate::request::{EncodeError, RequestContext, common::check_headers};
use http::{HeaderMap, HeaderValue, header::{CONTENT_ENCODING, CONTENT_LENGTH}};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompressionThreshold(usize);

impl CompressionThreshold {
    pub fn new(bytes: usize) -> Result<Self, EncodeError> {
        if bytes > 10_485_760 {
            return Err(EncodeError::Limit("compression threshold"));
        }
        Ok(Self(bytes))
    }

    pub const fn bytes(self) -> usize {
        self.0
    }
}

impl Default for CompressionThreshold {
    fn default() -> Self {
        Self(10_240)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompressionPolicy {
    Disabled,
    Enabled(CompressionThreshold),
}

impl Default for CompressionPolicy {
    fn default() -> Self {
        Self::Enabled(CompressionThreshold::default())
    }
}

pub(super) fn apply(
    context: RequestContext<'_>,
    headers: &mut HeaderMap,
    body: Vec<u8>,
) -> Result<Vec<u8>, EncodeError> {
    if body.len() > context.limits.body_bytes() {
        return Err(EncodeError::Limit("body"));
    }
    let CompressionPolicy::Enabled(threshold) = context.compression else {
        return Ok(body);
    };
    let Some(algorithm) = context.operation.request_compression()? else {
        return Ok(body);
    };
    if body.len() < threshold.bytes() {
        return Ok(body);
    }
    validate_projected_headers(
        headers,
        context.limits.header_bytes(),
        1,
    )?;
    let compressed = match algorithm {
        inventory_core::catalog::CompressionAlgorithm::Gzip => {
            gzip::encode(&body, context.limits.body_bytes())?
        }
    };
    validate_projected_headers(headers, context.limits.header_bytes(), compressed.len())?;
    headers.append(CONTENT_ENCODING, HeaderValue::from_static("gzip"));
    if headers.contains_key(CONTENT_LENGTH) {
        let value = HeaderValue::from_str(&compressed.len().to_string())?;
        headers.insert(CONTENT_LENGTH, value);
    }
    Ok(compressed)
}

fn validate_projected_headers(
    headers: &HeaderMap,
    limit: usize,
    content_length: usize,
) -> Result<(), EncodeError> {
    let current = check_headers(headers, limit)?;
    let encoding_cost = CONTENT_ENCODING.as_str().len().checked_add(4)
        .and_then(|size| size.checked_add(b"gzip".len()))
        .ok_or(EncodeError::Limit("headers"))?;
    let (length_count, removed_length_cost) = headers.get_all(CONTENT_LENGTH).iter()
        .try_fold((0usize, 0usize), |(count, total), value| {
            let cost = CONTENT_LENGTH.as_str().len().checked_add(value.as_bytes().len())
                .and_then(|cost| cost.checked_add(4)).ok_or(EncodeError::Limit("headers"))?;
            Ok::<(usize, usize), EncodeError>((
                count.checked_add(1).ok_or(EncodeError::Limit("headers"))?,
                total.checked_add(cost).ok_or(EncodeError::Limit("headers"))?,
            ))
        })?;
    let added_length_cost = if length_count == 0 {
        0
    } else {
        CONTENT_LENGTH.as_str().len().checked_add(4)
            .and_then(|size| size.checked_add(decimal_digits(content_length)))
            .ok_or(EncodeError::Limit("headers"))?
    };
    let projected = current.checked_add(encoding_cost)
        .and_then(|size| size.checked_sub(removed_length_cost))
        .and_then(|size| size.checked_add(added_length_cost))
        .ok_or(EncodeError::Limit("headers"))?;
    if projected > limit {
        return Err(EncodeError::Limit("headers"));
    }
    Ok(())
}

fn decimal_digits(value: usize) -> usize {
    let mut digits = 1usize;
    let mut remaining = value;
    while remaining >= 10 {
        remaining /= 10;
        digits = digits.saturating_add(1);
    }
    digits
}

#[cfg(test)]
mod tests;
