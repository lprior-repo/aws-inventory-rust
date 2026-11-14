use super::{CompressionThreshold, decimal_digits, gzip, validate_projected_headers};
use crate::request::{EncodeError, common::check_headers};
use flate2::read::GzDecoder;
use http::{
    HeaderMap, HeaderValue,
    header::{CONTENT_ENCODING, CONTENT_LENGTH},
};
use std::io::Read;

#[test]
fn thresholds_accept_inclusive_protocol_bounds() {
    assert!(matches!(
        CompressionThreshold::new(0),
        Ok(threshold) if threshold.bytes() == 0
    ));
    assert_eq!(CompressionThreshold::default().bytes(), 10_240);
    assert!(matches!(
        CompressionThreshold::new(10_485_760),
        Ok(threshold) if threshold.bytes() == 10_485_760
    ));
    assert!(matches!(
        CompressionThreshold::new(10_485_761),
        Err(EncodeError::Limit("compression threshold"))
    ));
    assert_eq!(decimal_digits(0), 1);
    assert_eq!(decimal_digits(10_485_760), 8);
}

#[test]
fn gzip_is_deterministic_and_round_trips_within_the_output_cap() {
    let input = b"bounded query request body".repeat(256);
    let first = gzip::encode(&input, 16_384).expect("gzip encoding succeeds");
    let second = gzip::encode(&input, 16_384).expect("repeat gzip encoding succeeds");
    assert_eq!(first, second);
    let mut decoded = Vec::new();
    GzDecoder::new(first.as_slice())
        .read_to_end(&mut decoded)
        .expect("gzip decodes");
    assert_eq!(decoded, input);
}

#[test]
fn gzip_rejects_input_and_compressed_output_over_body_cap() {
    assert!(matches!(
        gzip::encode(b"too long", 7),
        Err(EncodeError::Limit("body"))
    ));
    let incompressible = (0..=255).collect::<Vec<_>>();
    assert!(matches!(
        gzip::encode(&incompressible, incompressible.len()),
        Err(EncodeError::Limit("body")),
    ));
}

#[test]
fn projected_header_budget_uses_exact_compressed_length() {
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_LENGTH, HeaderValue::from_static("9"));
    assert!(validate_projected_headers(&headers, 128, 7).is_ok());
    assert!(matches!(
        validate_projected_headers(&headers, 32, 7),
        Err(EncodeError::Limit("headers"))
    ));

    let compressed = gzip::encode(b"exact header budget", 128).expect("compression fits");
    let compressed_length = compressed.len();
    assert!(decimal_digits(compressed_length) < decimal_digits(16_777_216));
    let mut exact_headers = headers.clone();
    exact_headers.append(CONTENT_ENCODING, HeaderValue::from_static("gzip"));
    exact_headers.insert(
        CONTENT_LENGTH,
        compressed_length
            .to_string()
            .parse()
            .expect("valid length header"),
    );
    let exact_limit =
        check_headers(&exact_headers, usize::MAX).expect("projected header count fits");
    assert!(validate_projected_headers(&headers, exact_limit, compressed_length).is_ok());
    assert!(matches!(
        validate_projected_headers(
            &headers,
            exact_limit.checked_sub(1).expect("positive header budget"),
            compressed_length
        ),
        Err(EncodeError::Limit("headers")),
    ));
    assert!(matches!(
        validate_projected_headers(&headers, exact_limit, 16_777_216),
        Err(EncodeError::Limit("headers")),
    ));

    headers.append(CONTENT_ENCODING, HeaderValue::from_static("br"));
    assert!(validate_projected_headers(&headers, 128, compressed_length).is_ok());
}
