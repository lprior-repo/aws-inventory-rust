use aws_wire::sign::{
    self, Body, Credentials, PathMode, PayloadHeader, Region, Request, Scope, SigningName,
};
use http::{HeaderMap, HeaderValue, Method};
use std::{error::Error, time::SystemTime};

type TestResult<T = ()> = Result<T, Box<dyn Error>>;
const URI: &str = "https://example.amazonaws.com/";

fn assert_signed(request: Request<'_>) -> TestResult {
    let credentials = Credentials::new("A", "s", None)?;
    let scope = Scope {
        region: Region::new("r")?,
        name: SigningName::new("s")?,
    };
    let result = sign::sign(
        &credentials,
        &scope,
        SystemTime::UNIX_EPOCH,
        request,
        PathMode::Standard,
    )?;
    assert_eq!(result.len(), 2);
    assert_eq!(
        result.get("x-amz-date").ok_or("missing date")?,
        "19700101T000000Z"
    );
    let authorization = result
        .get("authorization")
        .ok_or("missing authorization")?
        .to_str()?;
    assert!(authorization.starts_with("AWS4-HMAC-SHA256 Credential=A/19700101/r/s/aws4_request,"));
    Ok(())
}

fn get<'a>(uri: &'a str, headers: &'a HeaderMap, body: &'a [u8]) -> Request<'a> {
    Request {
        method: &Method::GET,
        uri,
        headers,
        body: Body::Bytes(body),
        payload_header: PayloadHeader::Omit,
    }
}

#[test]
fn maximum_uri_length_is_accepted() -> TestResult {
    // Given: an encoded absolute URI of exactly 16384 bytes.
    let uri = format!("https://example.com/{}", "a".repeat(16_364));
    assert_eq!(uri.len(), 16_384);
    // When/Then: its inclusive boundary can be signed.
    assert_signed(get(&uri, &HeaderMap::new(), b""))
}

#[test]
fn maximum_body_length_is_accepted_without_replacing_the_payload() -> TestResult {
    // Given: caller-owned body bytes at exactly 16 MiB.
    let body = vec![0; 16_777_216];
    // When/Then: the maximum borrowed body is signed successfully.
    assert_signed(get(URI, &HeaderMap::new(), &body))?;
    assert_eq!(body.len(), 16_777_216);
    assert_eq!(body.first(), Some(&0));
    assert_eq!(body.last(), Some(&0));
    Ok(())
}

#[test]
fn maximum_header_count_is_accepted() -> TestResult {
    // Given: 128 header values, including duplicates counted individually.
    let mut headers = HeaderMap::new();
    (0..128).for_each(|_| {
        headers.append("x-test", HeaderValue::from_static("value"));
    });
    assert_eq!(headers.len(), 128);
    // When/Then: the inclusive count bound still permits signing.
    assert_signed(get(URI, &headers, b""))
}

#[test]
fn maximum_header_value_length_is_accepted() -> TestResult {
    // Given: exactly 8192 bytes in a single header value.
    let mut headers = HeaderMap::new();
    headers.insert("x-test", HeaderValue::from_str(&"a".repeat(8192))?);
    // When/Then: the inclusive size bound still permits signing.
    assert_signed(get(URI, &headers, b""))
}

#[test]
fn maximum_method_length_is_accepted() -> TestResult {
    // Given: a valid HTTP extension method of exactly 64 bytes.
    let method = Method::from_bytes(&[b'A'; 64])?;
    let headers = HeaderMap::new();
    let request = Request {
        method: &method,
        uri: URI,
        headers: &headers,
        body: Body::Bytes(b""),
        payload_header: PayloadHeader::Omit,
    };
    // When/Then: its inclusive boundary can be signed.
    assert_signed(request)
}
