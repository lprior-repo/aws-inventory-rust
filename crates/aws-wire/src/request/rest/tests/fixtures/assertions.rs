use super::xml_infoset;
use crate::request::Protocol;
use inventory_core::catalog::{HttpBinding, Operation, Service, Target};
use serde_json::Value as Json;

#[path = "../../../../../tests/request_body_cases/json_semantics.rs"]
mod json_semantics;

pub(super) fn check_headers(case: &Json, request: &http::Request<bytes::Bytes>, label: &str) {
    let expected = case.get("headers").and_then(Json::as_object);
    check_header_values(expected, request, label);
    check_header_presence(case, request, label);
    check_signing_headers(request, label);
}

fn check_header_values(
    expected: Option<&serde_json::Map<String, Json>>,
    request: &http::Request<bytes::Bytes>,
    label: &str,
) {
    for (name, value) in expected.into_iter().flatten() {
        let actual =
            header_text(request, name).unwrap_or_else(|| panic!("{label}: missing header {name}"));
        assert_eq!(
            actual,
            value.as_str().expect("header fixture text"),
            "{label} header {name}"
        );
    }
}

fn header_text(request: &http::Request<bytes::Bytes>, name: &str) -> Option<String> {
    let values = request
        .headers()
        .get_all(name)
        .iter()
        .map(|value| String::from_utf8_lossy(value.as_bytes()).into_owned())
        .collect::<Vec<_>>();
    (!values.is_empty()).then(|| values.join(", "))
}

fn check_header_presence(case: &Json, request: &http::Request<bytes::Bytes>, label: &str) {
    for (field, present) in [("requireHeaders", true), ("forbidHeaders", false)] {
        for name in case
            .get(field)
            .and_then(Json::as_array)
            .into_iter()
            .flatten()
        {
            let name = name.as_str().expect("fixture header name");
            assert_eq!(
                request.headers().contains_key(name),
                present,
                "{label}: {field} {name}"
            );
        }
    }
}

fn check_signing_headers(request: &http::Request<bytes::Bytes>, label: &str) {
    for forbidden in ["authorization", "x-amz-date", "x-amz-security-token"] {
        assert!(
            !request.headers().contains_key(forbidden),
            "{label}: forbidden signing header {forbidden}"
        );
    }
}

pub(super) fn check_body(
    case: &Json,
    request: &http::Request<bytes::Bytes>,
    plain: &[u8],
    protocol: Protocol,
    label: &str,
    service: Service<'_>,
    operation: Operation<'_>,
) {
    let (body, compressed) = decoded_body(request, label);
    let Some(expected) = case.get("body").and_then(Json::as_str) else {
        check_omitted_body(&body, compressed, plain, label);
        return;
    };
    compare_body(case, &body, expected, protocol, label, service, operation);
}

fn decoded_body(request: &http::Request<bytes::Bytes>, label: &str) -> (Vec<u8>, bool) {
    let body = request.body().as_ref();
    let gzip = header_text(request, "content-encoding").is_some_and(|value| {
        value
            .split(',')
            .any(|encoding| encoding.trim().eq_ignore_ascii_case("gzip"))
    });
    if gzip {
        (decompress_gzip(body, label), true)
    } else {
        (body.to_vec(), false)
    }
}

fn check_omitted_body(body: &[u8], compressed: bool, plain: &[u8], label: &str) {
    if compressed {
        assert!(!plain.is_empty(), "{label}: compressed an empty body");
        assert_eq!(body, plain, "{label} decompressed body");
    } else {
        assert!(body.is_empty(), "{label}: fixture omits body and payload");
    }
}

fn compare_body(
    case: &Json,
    body: &[u8],
    expected: &str,
    protocol: Protocol,
    label: &str,
    service: Service<'_>,
    operation: Operation<'_>,
) {
    if raw_payload(service, operation) {
        assert_eq!(body, expected.as_bytes(), "{label} raw payload");
        return;
    }
    let media_type = case
        .get("bodyMediaType")
        .and_then(Json::as_str)
        .or_else(|| {
            case.get("headers")?
                .as_object()?
                .iter()
                .find_map(|(name, value)| {
                    name.eq_ignore_ascii_case("content-type")
                        .then(|| value.as_str())
                        .flatten()
                })
        })
        .unwrap_or(match protocol {
            Protocol::RestJson1 => "application/json",
            Protocol::RestXml => "application/xml",
            _ => "application/octet-stream",
        });
    let media_type = media_type.split(';').next().unwrap_or_default().trim();
    if expected.is_empty() {
        assert!(body.is_empty(), "{label}: expected empty body");
    } else if media_type == "application/json" || media_type.ends_with("+json") {
        compare_json_body(body, expected, label);
    } else if matches!(media_type, "application/xml" | "text/xml") || media_type.ends_with("+xml") {
        compare_xml_body(body, expected, label, service, operation);
    } else {
        assert_eq!(body, expected.as_bytes(), "{label} raw payload");
    }
}

fn compare_json_body(body: &[u8], expected: &str, label: &str) {
    let actual: Json = serde_json::from_slice(body)
        .unwrap_or_else(|error| panic!("{label}: invalid JSON body: {error}"));
    let expected: Json = serde_json::from_str(expected)
        .unwrap_or_else(|error| panic!("{label}: invalid fixture JSON body: {error}"));
    assert!(
        json_semantics::equivalent(&actual, &expected),
        "{label} JSON body: actual={actual}, expected={expected}"
    );
}

fn raw_payload(service: Service<'_>, operation: Operation<'_>) -> bool {
    let Target::Shape(index) = operation.input().expect("fixture input") else {
        return false;
    };
    let input = service
        .shape(index)
        .expect("input shape lookup")
        .expect("input shape");
    for member in input.members().expect("input members") {
        let member = member.expect("input member");
        if member
            .wire()
            .expect("member wire")
            .binding()
            .expect("binding")
            != HttpBinding::Payload
        {
            continue;
        }
        let kind = match member.target {
            Target::Shape(index) => service
                .shape(index)
                .expect("payload lookup")
                .expect("payload shape")
                .kind()
                .expect("payload kind"),
            Target::Prelude(kind) => kind,
            _ => panic!("unknown fixture payload target"),
        };
        return matches!(kind, "blob" | "string" | "enum");
    }
    false
}

fn compare_xml_body(
    body: &[u8],
    expected: &str,
    label: &str,
    service: Service<'_>,
    operation: Operation<'_>,
) {
    assert!(
        xml_infoset::equal(body, expected.as_bytes(), service, operation),
        "{label} XML body: actual={}, expected={expected}",
        String::from_utf8_lossy(body)
    );
}

fn decompress_gzip(body: &[u8], label: &str) -> Vec<u8> {
    use std::io::Read;
    let mut output = Vec::new();
    flate2::read::GzDecoder::new(body)
        .take(16 * 1024 * 1024 + 1)
        .read_to_end(&mut output)
        .unwrap_or_else(|error| panic!("{label}: invalid gzip request body: {error}"));
    assert!(
        output.len() <= 16 * 1024 * 1024,
        "{label}: decompressed fixture body exceeds the catalog body bound"
    );
    output
}
