use aws_wire::sign::{
    self, Body, Credentials, PathMode, PayloadHeader, Region, Request, Scope, SigningName,
};
use http::{HeaderMap, HeaderName, HeaderValue, Method};
use std::{
    error::Error,
    fs,
    path::Path,
    time::{Duration, SystemTime},
};

type TestResult<T = ()> = Result<T, Box<dyn Error>>;
const SECRET: &str = "wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY";
const TOKEN: &str = "6e86291e8372ff2a2260956d9b8aae1d763fbf315fa00fa31553b73ebf194267";
const STS_TOKEN: &str = "AQoDYXdzEPT//////////wEXAMPLEtc764bNrC9SAPBSM22wDOk4x4HIZ8j4FZTwdQWLWsKWHGBuFqwAeMicRXmxfpSPfIeoIYRqTflfKD8YUuwthAx7mSEI/qkPpKPi/kMcGdQrmGdeehM4IC1NtBmUpp2wUE8phUZampKsburEDy0KPkyQDYwT7WZ0wq5VSXDvp75YU9HFvlRd8Tx6q6fE8YQcHNVXAkiY9q6d+xo0rKwT38xVqr7ZD0u0iPPkUL64lIZbqBAz+scqKmlzm8FDrypNC9Yjc8fPOLn9FX9KSYvKTr4rvx3iSIlTJabIQwj2ICCR/oLxBA==";
const CASES: &[&str] = &[
    "double-encode-path",
    "double-url-encode",
    "get-header-key-duplicate",
    "get-header-value-order",
    "get-header-value-trim",
    "get-relative-normalized",
    "get-relative-relative-normalized",
    "get-relative-relative-unnormalized",
    "get-relative-unnormalized",
    "get-slash-dot-slash-normalized",
    "get-slash-dot-slash-unnormalized",
    "get-slash-normalized",
    "get-slash-pointless-dot-normalized",
    "get-slash-pointless-dot-unnormalized",
    "get-slash-unnormalized",
    "get-slashes-normalized",
    "get-slashes-unnormalized",
    "get-space-normalized",
    "get-unreserved",
    "get-utf8",
    "get-vanilla",
    "get-vanilla-empty-query-key",
    "get-vanilla-query",
    "get-vanilla-query-order-encoded",
    "get-vanilla-query-order-key-case",
    "get-vanilla-query-unreserved",
    "get-vanilla-utf8-query",
    "get-vanilla-with-session-token",
    "post-header-key-case",
    "post-header-key-sort",
    "post-header-value-case",
    "post-sts-header-before",
    "post-vanilla",
    "post-vanilla-empty-query-value",
    "post-vanilla-query",
    "post-x-www-form-urlencoded",
    "post-x-www-form-urlencoded-parameters",
];

struct Parsed<'a> {
    method: Method,
    uri: String,
    headers: HeaderMap,
    body: &'a [u8],
}

fn parse_request(text: &str) -> TestResult<Parsed<'_>> {
    if text.len() > 8192 {
        return Err("fixture exceeds parser bound".into());
    }
    let Some((head, body)) = text.split_once("\n\n") else {
        return parse_head(text);
    };
    Ok(Parsed {
        body: body.as_bytes(),
        ..parse_head(head)?
    })
}

fn parse_head(head: &str) -> TestResult<Parsed<'static>> {
    if head.lines().count() > 65 {
        return Err("too many fixture headers".into());
    }
    let mut lines = head.lines();
    let first = lines.next().ok_or("missing request line")?;
    let (method, target) = first.split_once(' ').ok_or("missing target")?;
    let target = target
        .strip_suffix(" HTTP/1.1")
        .ok_or("missing HTTP version")?;
    let headers = lines.take(64).try_fold(HeaderMap::new(), parse_header)?;
    let host = headers.get("host").ok_or("missing host")?.to_str()?;
    Ok(Parsed {
        method: Method::from_bytes(method.as_bytes())?,
        uri: format!("https://{host}{target}"),
        headers,
        body: b"",
    })
}

fn parse_header(mut headers: HeaderMap, line: &str) -> TestResult<HeaderMap> {
    let (name, value) = line.split_once(':').ok_or("malformed fixture header")?;
    if !name.eq_ignore_ascii_case("x-amz-date") {
        headers.append(
            HeaderName::from_bytes(name.as_bytes())?,
            HeaderValue::from_str(value.trim())?,
        );
    }
    Ok(headers)
}

fn credentials(case: &str) -> TestResult<Credentials> {
    let token = match case {
        "get-vanilla-with-session-token" => Some(TOKEN),
        "post-sts-header-before" => Some(STS_TOKEN),
        _ => None,
    };
    if case == "double-encode-path" {
        return Ok(Credentials::new(
            "ANOTREAL",
            "notrealrnrELgWzOk3IfjzDKtFBhDby",
            None,
        )?);
    }
    Ok(Credentials::new("AKIDEXAMPLE", SECRET, token)?)
}

fn scope_time(case: &str) -> TestResult<(Scope<'static>, SystemTime)> {
    let (region, service, seconds) = match case {
        "double-url-encode" => ("us-east-2", "lambda", 1_620_747_645),
        _ => ("us-east-1", "service", 1_440_938_160),
    };
    Ok((
        Scope {
            region: Region::new(region)?,
            name: SigningName::new(service)?,
        },
        SystemTime::UNIX_EPOCH
            .checked_add(Duration::from_secs(seconds))
            .ok_or("time overflow")?,
    ))
}

fn vector_modes(case: &str) -> (PathMode, PayloadHeader) {
    let path = if case.ends_with("-unnormalized") || case == "get-space-normalized" {
        PathMode::S3
    } else {
        PathMode::Standard
    };
    let payload = if case.starts_with("post-x-www-form-urlencoded") {
        PayloadHeader::Include
    } else {
        PayloadHeader::Omit
    };
    (path, payload)
}

fn sign_vector(case: &str, parsed: &Parsed<'_>) -> TestResult<HeaderMap> {
    let (scope, time) = scope_time(case)?;
    let (path, payload) = vector_modes(case);
    let credentials = credentials(case)?;
    let request = Request {
        method: &parsed.method,
        uri: &parsed.uri,
        headers: &parsed.headers,
        body: Body::Bytes(parsed.body),
        payload_header: payload,
    };
    Ok(sign::sign(&credentials, &scope, time, request, path)?)
}

fn assert_vector(case: &str, root: &Path) -> TestResult {
    // Given: unchanged AWS request and signed-request fixtures, with a fixed clock.
    let input = fs::read_to_string(root.join(case).join("request.txt"))?;
    let expected = fs::read_to_string(root.join(case).join("header-signed-request.txt"))?;
    let parsed = parse_request(input.trim_end_matches('\n'))?;
    // When: the public wire signer signs the parsed request without changing its body.
    let result = sign_vector(case, &parsed)?;
    let actual = result
        .get("authorization")
        .ok_or("missing authorization")?
        .to_str()?;
    // Then: the entire Authorization value, not merely a signature suffix, matches AWS.
    assert_eq!(actual, expected_authorization(&expected)?, "{case}");
    Ok(())
}

fn expected_authorization(text: &str) -> TestResult<&str> {
    text.lines()
        .filter_map(|line| line.split_once(':'))
        .find(|(name, _)| name.eq_ignore_ascii_case("authorization"))
        .map(|(_, value)| value.trim())
        .ok_or_else(|| "missing expected authorization".into())
}

#[test]
fn published_v4_headers_match_for_every_copied_case() -> TestResult {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sigv4");
    let mut actual = fs::read_dir(&root)?
        .map(|entry| Ok(entry?.path()))
        .collect::<TestResult<Vec<_>>>()?;
    actual.retain(|path| path.is_dir());
    assert_eq!(
        actual.len(),
        CASES.len(),
        "every copied case must be exercised"
    );
    CASES.iter().try_for_each(|case| assert_vector(case, &root))
}
