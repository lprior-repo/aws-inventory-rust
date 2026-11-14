use PathMode::{S3, Standard};
use PayloadHeader::{Include, Omit};
use aws_wire::sign::{
    self, Body, Credentials, PathMode, PayloadHeader, Region, Request, Scope, SigningName,
};
use http::{HeaderMap, Method};
use std::{
    error::Error,
    time::{Duration, SystemTime},
};

type TestResult<T = ()> = Result<T, Box<dyn Error>>;
const SECRET: &str = "wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY";
const ABC_SHA256: &str = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
const ABC_NEWLINE_SHA256: &str = "edeaaff3f1774ad2888673770c6d64097e391bc362d7d6fb34982ddf0efd18cb";
const EMPTY_SHA256: &str = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
const URI: &str = "https://example.amazonaws.com/";
const TIME: u64 = 1_440_938_160;
const EMPTY: Body<'static> = Body::Bytes(b"");

fn at(seconds: u64) -> TestResult<SystemTime> {
    SystemTime::UNIX_EPOCH
        .checked_add(Duration::from_secs(seconds))
        .ok_or_else(|| "time overflow".into())
}

fn sign_request(
    request: Request<'_>,
    token: Option<&str>,
    seconds: u64,
    path: PathMode,
) -> TestResult<HeaderMap> {
    let scope = Scope {
        region: Region::new("us-east-1")?,
        name: SigningName::new("service")?,
    };
    let credentials = Credentials::new("AKIDEXAMPLE", SECRET, token)?;
    Ok(sign::sign(
        &credentials,
        &scope,
        at(seconds)?,
        request,
        path,
    )?)
}

fn signed(
    body: Body<'_>,
    checksum: PayloadHeader,
    token: Option<&str>,
    seconds: u64,
) -> TestResult<HeaderMap> {
    let headers = HeaderMap::new();
    let request = Request {
        method: &Method::GET,
        uri: URI,
        headers: &headers,
        body,
        payload_header: checksum,
    };
    sign_request(request, token, seconds, Standard)
}

fn key(uri: &str, path: PathMode) -> TestResult<HeaderMap> {
    key_with_payload(uri, path, Include)
}

fn key_with_payload(uri: &str, path: PathMode, checksum: PayloadHeader) -> TestResult<HeaderMap> {
    let headers = HeaderMap::new();
    let request = Request {
        method: &Method::GET,
        uri,
        headers: &headers,
        body: EMPTY,
        payload_header: checksum,
    };
    sign_request(request, None, TIME, path)
}

fn text<'a>(headers: &'a HeaderMap, name: &str) -> TestResult<&'a str> {
    Ok(headers.get(name).ok_or("missing header")?.to_str()?)
}

#[test]
fn session_token_is_emitted_and_included_in_the_signature() -> TestResult {
    // Given: identical requests and time, differing only in temporary credentials.
    let plain = signed(EMPTY, Omit, None, TIME)?;
    // When: a session token is supplied.
    let temporary = signed(EMPTY, Omit, Some("session-secret"), TIME)?;
    // Then: it is present, sensitive, signed, and changes authorization.
    let token = temporary
        .get("x-amz-security-token")
        .ok_or("missing token")?;
    assert_eq!(token, "session-secret");
    assert!(token.is_sensitive());
    let authorization = text(&temporary, "authorization")?;
    assert!(authorization.contains("SignedHeaders=host;x-amz-date;x-amz-security-token,"));
    assert_ne!(temporary.get("authorization"), plain.get("authorization"));
    assert_eq!(plain.get("x-amz-security-token"), None);
    Ok(())
}

#[test]
fn s3_preserves_spaces_and_repeated_slashes_in_object_keys() -> TestResult {
    // Given: a once-encoded key with a space and repeated slash.
    let uri = "https://example.amazonaws.com/a%20b//c";
    // When: both path modes sign exactly those bytes.
    let s3 = key(uri, S3)?;
    let standard = key(uri, Standard)?;
    let collapsed = key("https://example.amazonaws.com/a%20b/c", S3)?;
    let encoded_twice = key("https://example.amazonaws.com/a%2520b//c", S3)?;
    // Then: S3 signs neither the normalized nor the double-encoded key.
    assert_ne!(s3.get("authorization"), standard.get("authorization"));
    assert_ne!(s3.get("authorization"), collapsed.get("authorization"));
    assert_ne!(s3.get("authorization"), encoded_twice.get("authorization"));
    assert_eq!(text(&s3, "x-amz-content-sha256")?, EMPTY_SHA256);
    Ok(())
}

#[test]
fn resigning_one_second_later_changes_date_and_authorization() -> TestResult {
    // Given: an unsigned request retained by the executor.
    let first = signed(EMPTY, Omit, None, TIME)?;
    // When: an expired-signature retry uses the next supplied second.
    let next = signed(EMPTY, Omit, None, 1_440_938_161)?;
    // Then: no cached timestamp or signature survives.
    assert_eq!(text(&first, "x-amz-date")?, "20150830T123600Z");
    assert_eq!(text(&next, "x-amz-date")?, "20150830T123601Z");
    assert_ne!(first.get("authorization"), next.get("authorization"));
    assert_eq!(first.len(), 2);
    Ok(())
}

fn compare_digests(body: &[u8], sha256: &str, checksum: PayloadHeader) -> TestResult {
    // Given/When: bytes and their independently known digest use the public API.
    let bytes = signed(Body::Bytes(body), checksum, None, TIME)?;
    let digest = signed(Body::Sha256(sha256), checksum, None, TIME)?;
    // Then: every emitted header agrees, including checksum selection.
    assert_eq!(bytes, digest);
    match checksum {
        Include => assert_eq!(text(&bytes, "x-amz-content-sha256")?, sha256),
        Omit => assert_eq!(bytes.get("x-amz-content-sha256"), None),
        _ => return Err("unexpected checksum mode".into()),
    }
    assert!(
        bytes
            .get("authorization")
            .ok_or("missing authorization")?
            .is_sensitive()
    );
    Ok(())
}

#[test]
fn precomputed_digest_matches_bytes_with_and_without_checksum_header() -> TestResult {
    [Omit, Include]
        .into_iter()
        .try_for_each(|checksum| compare_digests(b"abc", ABC_SHA256, checksum))
}

#[test]
fn credential_debug_redacts_both_secret_and_session_token() -> TestResult {
    // Given: temporary credentials with recognizable secrets.
    let credentials = Credentials::new("AKIDEXAMPLE", "never-print", Some("hidden-token"))?;
    // When: caller diagnostics format the credentials.
    let debug = format!("{credentials:?}");
    // Then: only the fixed redacted representation is visible.
    assert_eq!(debug, "Credentials([REDACTED])");
    Ok(())
}

fn borrowed(headers: &HeaderMap, body: &[u8]) -> TestResult<HeaderMap> {
    let request = Request {
        method: &Method::POST,
        uri: URI,
        headers,
        body: Body::Bytes(body),
        payload_header: PayloadHeader::default(),
    };
    sign_request(request, None, TIME, PathMode::default())
}

#[test]
fn fixed_inputs_are_deterministic_and_do_not_mutate_request_parts() -> TestResult {
    // Given: caller-owned headers and binary payload.
    let mut headers = HeaderMap::new();
    headers.insert(
        "content-type",
        http::HeaderValue::from_static("application/octet-stream"),
    );
    let body = [0, 255, 1, 128];
    // When: the same borrowed request is signed twice.
    let first = borrowed(&headers, &body)?;
    let second = borrowed(&headers, &body)?;
    // Then: output agrees; inputs remain unsigned and byte-identical.
    assert_eq!(first, second);
    assert_eq!(headers.len(), 1);
    assert_eq!(body, [0, 255, 1, 128]);
    Ok(())
}

#[test]
fn supported_time_endpoints_have_exact_dates() -> TestResult {
    // Given: inclusive boundaries guarding the AWS library's date conversion.
    [
        (0, "19700101T000000Z"),
        (253_402_300_799, "99991231T235959Z"),
    ]
    .into_iter()
    .try_for_each(|(seconds, expected)| {
        // When: signing at a boundary. Then: the exact date is retained.
        let result = signed(EMPTY, Omit, None, seconds)?;
        assert_eq!(text(&result, "x-amz-date")?, expected);
        Ok(())
    })
}

#[test]
fn newline_body_matches_its_digest_without_trimming_in_both_checksum_modes() -> TestResult {
    // Given: the published SHA-256 of all four bytes, including the final newline.
    // When/Then: complete header maps and included checksum values match exactly.
    [Omit, Include]
        .into_iter()
        .try_for_each(|checksum| compare_digests(b"abc\n", ABC_NEWLINE_SHA256, checksum))
}

#[test]
fn default_signing_modes_match_standard_paths_and_omit_payload_headers() -> TestResult {
    // Given: a path whose space and repeated slash distinguish Standard from S3.
    let uri = "https://example.amazonaws.com/a%20b//c";
    // When: defaults and both explicit modes sign identical unsigned inputs.
    let defaults = key_with_payload(uri, PathMode::default(), PayloadHeader::default())?;
    let standard = key_with_payload(uri, Standard, Omit)?;
    let s3 = key_with_payload(uri, S3, Omit)?;
    // Then: the complete default map equals Standard/Omit, not S3.
    assert_eq!(defaults, standard);
    assert_ne!(defaults, s3);
    assert_eq!(defaults.get("x-amz-content-sha256"), None);
    Ok(())
}
