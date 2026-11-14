use super::*;

pub(super) fn decode_request_body(request: &http::Request<bytes::Bytes>) -> FixtureResult<Vec<u8>> {
    let encodings = request.headers().get_all(http::header::CONTENT_ENCODING);
    let is_gzip = encodings
        .iter()
        .any(|value| value.to_str().ok() == Some("gzip"));
    if !is_gzip {
        return Ok(request.body().to_vec());
    }
    let mut decoded = Vec::new();
    GzDecoder::new(request.body().as_ref())
        .take((RequestLimits::default().body_bytes() as u64) + 1)
        .read_to_end(&mut decoded)?;
    if decoded.len() > RequestLimits::default().body_bytes() {
        return Err(fixture_error("decompressed request exceeds body limit"));
    }
    Ok(decoded)
}

pub(super) fn assert_compression_boundaries(
    context: RequestContext<'_>,
    plain_context: RequestContext<'_>,
    plain_request: &http::Request<bytes::Bytes>,
) -> FixtureResult<()> {
    let body = plain_request.body().to_vec();
    let disabled_body = body.clone();
    let disabled_pointer = disabled_body.as_ptr();
    let mut disabled_headers = plain_request.headers().clone();
    let disabled_context = RequestContext {
        compression: CompressionPolicy::Disabled,
        ..context
    };
    let unchanged =
        crate::request::compression::apply(disabled_context, &mut disabled_headers, disabled_body)?;
    assert_eq!(unchanged.as_ptr(), disabled_pointer);
    assert_eq!(disabled_headers, *plain_request.headers());

    let above_threshold = body
        .len()
        .checked_add(1)
        .ok_or_else(|| fixture_error("fixture body length overflow"))?;
    let above_context = RequestContext {
        compression: CompressionPolicy::Enabled(crate::request::CompressionThreshold::new(
            above_threshold,
        )?),
        ..plain_context
    };
    let mut above_headers = plain_request.headers().clone();
    assert_eq!(
        crate::request::compression::apply(above_context, &mut above_headers, body.clone())?,
        body
    );
    assert_eq!(above_headers, *plain_request.headers());

    let exact_context = RequestContext {
        compression: CompressionPolicy::Enabled(crate::request::CompressionThreshold::new(
            body.len(),
        )?),
        ..plain_context
    };
    let mut exact_headers = plain_request.headers().clone();
    let exact_body =
        crate::request::compression::apply(exact_context, &mut exact_headers, body.clone())?;
    assert_eq!(decode_header_body(&exact_headers, &exact_body)?, body);
    let expected_length = exact_body.len().to_string();
    assert_eq!(
        exact_headers
            .get(http::header::CONTENT_LENGTH)
            .and_then(|value| value.to_str().ok()),
        Some(expected_length.as_str()),
    );
    let existing_header_bytes = crate::request::common::check_headers(
        plain_request.headers(),
        RequestLimits::default().header_bytes(),
    )?;
    let header_limit = existing_header_bytes
        .checked_add(1)
        .ok_or_else(|| fixture_error("fixture header size overflow"))?;
    let limits = RequestLimits::new(
        RequestLimits::default().body_bytes(),
        header_limit,
        RequestLimits::default().uri_bytes(),
    )?;
    let constrained_context = RequestContext {
        limits,
        ..exact_context
    };
    let mut constrained_headers = plain_request.headers().clone();
    assert!(matches!(
        crate::request::compression::apply(
            constrained_context,
            &mut constrained_headers,
            body.clone()
        ),
        Err(crate::request::EncodeError::Limit("headers")),
    ));
    assert_eq!(constrained_headers, *plain_request.headers());

    let mut appended_headers = plain_request.headers().clone();
    appended_headers.insert(
        http::header::CONTENT_ENCODING,
        http::HeaderValue::from_static("br"),
    );
    let appended_body =
        crate::request::compression::apply(exact_context, &mut appended_headers, body.clone())?;
    let encodings = appended_headers
        .get_all(http::header::CONTENT_ENCODING)
        .iter()
        .map(|value| value.to_str())
        .collect::<Result<Vec<_>, _>>()?;
    assert_eq!(encodings, ["br", "gzip"]);
    assert_eq!(decode_header_body(&appended_headers, &appended_body)?, body);
    Ok(())
}

fn decode_header_body(headers: &http::HeaderMap, body: &[u8]) -> FixtureResult<Vec<u8>> {
    let mut decoded = Vec::new();
    if headers
        .get_all(http::header::CONTENT_ENCODING)
        .iter()
        .any(|value| value.to_str().ok() == Some("gzip"))
    {
        GzDecoder::new(body)
            .take((RequestLimits::default().body_bytes() as u64) + 1)
            .read_to_end(&mut decoded)?;
        if decoded.len() > RequestLimits::default().body_bytes() {
            return Err(fixture_error(
                "decompressed boundary body exceeds body limit",
            ));
        }
        return Ok(decoded);
    }
    Ok(body.to_vec())
}
