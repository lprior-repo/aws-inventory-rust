use aws_wire::sign::{
    self, Body, Credentials, Error, PathMode, PayloadHeader, Region, Request, Scope, SigningName,
};
use http::{HeaderMap, HeaderValue, Method};
use std::{
    error::Error as StdError,
    time::{Duration, SystemTime},
};

type TestResult<T = ()> = Result<T, Box<dyn StdError>>;
const URI: &str = "https://example.amazonaws.com/";
const EMPTY: Body<'static> = Body::Bytes(b"");

fn execute(request: Request<'_>, time: SystemTime) -> Result<HeaderMap, Error> {
    let credentials = Credentials::new("AKIDEXAMPLE", "secret", None)?;
    let scope = Scope {
        region: Region::new("us-east-1")?,
        name: SigningName::new("service")?,
    };
    sign::sign(&credentials, &scope, time, request, PathMode::Standard)
}

fn timed(
    uri: &str,
    headers: &HeaderMap,
    body: Body<'_>,
    time: SystemTime,
) -> Result<HeaderMap, Error> {
    let request = Request {
        method: &Method::GET,
        uri,
        headers,
        body,
        payload_header: PayloadHeader::Omit,
    };
    execute(request, time)
}

fn attempt(uri: &str, headers: &HeaderMap, body: Body<'_>) -> Result<HeaderMap, Error> {
    timed(uri, headers, body, SystemTime::UNIX_EPOCH)
}

#[test]
fn malformed_absolute_uris_return_uri_errors() {
    let cases = [
        "/relative",
        "",
        "https:///",
        "https://",
        "ftp://example.com/",
        "https://user@example.com/",
        "https://example.com/#fragment",
        "https://example.com/a b",
    ];

    cases.into_iter().for_each(|uri| {
        assert_eq!(
            attempt(uri, &HeaderMap::new(), EMPTY),
            Err(Error::Uri),
            "{uri}"
        );
    });
}

#[test]
fn non_utf8_headers_return_header_errors_without_panicking() -> TestResult {
    let mut headers = HeaderMap::new();
    headers.insert("content-type", HeaderValue::from_bytes(&[255])?);

    assert_eq!(attempt(URI, &headers, EMPTY), Err(Error::Headers));
    Ok(())
}

#[test]
fn conflicting_host_headers_are_rejected() -> TestResult {
    ["", "different.example.com", "example.amazonaws.com:443"]
        .into_iter()
        .try_for_each(|host| {
            let mut headers = HeaderMap::new();
            headers.insert("host", HeaderValue::from_str(host)?);

            assert_eq!(attempt(URI, &headers, EMPTY), Err(Error::Headers));
            Ok(())
        })
}

#[test]
fn duplicate_host_headers_are_rejected() {
    let mut headers = HeaderMap::new();
    headers.append("host", HeaderValue::from_static("example.amazonaws.com"));
    headers.append("host", HeaderValue::from_static("example.amazonaws.com"));

    assert_eq!(attempt(URI, &headers, EMPTY), Err(Error::Headers));
}

#[test]
fn signing_owned_headers_are_rejected_in_unsigned_inputs() {
    [
        "authorization",
        "x-amz-date",
        "x-amz-content-sha256",
        "x-amz-security-token",
    ]
    .into_iter()
    .for_each(|name| {
        let mut headers = HeaderMap::new();
        headers.insert(name, HeaderValue::from_static("stale"));

        assert_eq!(attempt(URI, &headers, EMPTY), Err(Error::Headers));
    });
}

#[test]
fn malformed_precomputed_hashes_return_digest_errors() {
    let cases = [
        "".to_owned(),
        "a".repeat(63),
        "a".repeat(65),
        "A".repeat(64),
        "g".repeat(64),
        "é".repeat(32),
    ];

    cases.iter().for_each(|digest| {
        let result = attempt(URI, &HeaderMap::new(), Body::Sha256(digest));
        assert_eq!(result, Err(Error::Digest));
    });
}

#[test]
fn out_of_range_times_return_time_errors() -> TestResult {
    let epoch = SystemTime::UNIX_EPOCH;
    let early = epoch
        .checked_sub(Duration::from_secs(1))
        .ok_or("early time")?;
    let late = epoch
        .checked_add(Duration::from_secs(253_402_300_800))
        .ok_or("late time")?;

    [early, late].into_iter().for_each(|time| {
        assert_eq!(timed(URI, &HeaderMap::new(), EMPTY, time), Err(Error::Time));
    });
    Ok(())
}

#[test]
fn oversized_uri_and_body_return_bound_errors() {
    let uri = format!("https://example.com/{}", "a".repeat(16_365));
    let body = vec![0; 16_777_217];

    assert_eq!(attempt(&uri, &HeaderMap::new(), EMPTY), Err(Error::Bounds));
    let result = attempt(URI, &HeaderMap::new(), Body::Bytes(&body));
    assert_eq!(result, Err(Error::Bounds));
}

#[test]
fn oversized_header_count_returns_bound_error() {
    let mut headers = HeaderMap::new();
    (0..129).for_each(|_| {
        headers.append("x-test", HeaderValue::from_static("value"));
    });

    assert_eq!(attempt(URI, &headers, EMPTY), Err(Error::Bounds));
}

#[test]
fn oversized_header_values_return_bound_errors() -> TestResult {
    let mut headers = HeaderMap::new();
    headers.insert("x-test", HeaderValue::from_str(&"a".repeat(8193))?);

    assert_eq!(attempt(URI, &headers, EMPTY), Err(Error::Bounds));
    Ok(())
}

#[test]
fn oversized_host_values_return_bound_errors_before_host_comparison() -> TestResult {
    let mut headers = HeaderMap::new();
    headers.insert("host", HeaderValue::from_str(&"a".repeat(8193))?);

    assert_eq!(attempt(URI, &headers, EMPTY), Err(Error::Bounds));
    Ok(())
}

#[test]
fn oversized_methods_return_bound_errors() -> TestResult {
    let method = Method::from_bytes(&[b'A'; 65])?;
    let headers = HeaderMap::new();
    let request = Request {
        method: &method,
        uri: URI,
        headers: &headers,
        body: EMPTY,
        payload_header: PayloadHeader::Omit,
    };

    assert_eq!(execute(request, SystemTime::UNIX_EPOCH), Err(Error::Bounds));
    Ok(())
}

#[test]
fn matching_host_with_explicit_port_and_http_scheme_is_supported() -> TestResult {
    let mut headers = HeaderMap::new();
    headers.insert(
        "host",
        HeaderValue::from_static("EXAMPLE.AMAZONAWS.COM:8080"),
    );

    let result = attempt("http://example.amazonaws.com:8080/", &headers, EMPTY)?;

    let authorization = result
        .get("authorization")
        .ok_or("missing authorization")?
        .to_str()?;
    assert!(authorization.contains("AKIDEXAMPLE/19700101/us-east-1/service/aws4_request,"));
    Ok(())
}
