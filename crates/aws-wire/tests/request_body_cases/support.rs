#[path = "cbor_semantics.rs"]
mod cbor_semantics;
#[path = "json_semantics.rs"]
mod json_semantics;

use super::Suite;
use super::request_body_values::decode_base64;
use aws_wire::request::Protocol;
use catalog_tool::smithy::{Registry, build_catalog};
use flate2::read::GzDecoder;
use http::Uri;
use serde_json::{Map as JsonMap, Value as Json};
use std::io::Read;
use std::{
    collections::BTreeSet,
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};
static NEXT_TEMP: AtomicUsize = AtomicUsize::new(0);
pub(super) fn compile_fixture_service(fixture: &Json, service_id: &str) -> Vec<u8> {
    let all_shapes = fixture["shapes"].as_object().expect("fixture shapes");
    let service_name = service_id.rsplit('#').next().expect("service name");
    let service_namespace = service_id.split('#').next().expect("service namespace");
    let mut selected_shapes = JsonMap::new();
    for (id, shape) in all_shapes {
        let is_service = shape["type"] == "service";
        if !is_service || id == service_id {
            selected_shapes.insert(id.clone(), shape.clone());
        }
    }
    let names: BTreeSet<String> = selected_shapes
        .values()
        .filter_map(|shape| shape.get("members").and_then(Json::as_object))
        .flat_map(|members| members.keys().cloned())
        .collect();
    let mut registry_text = names.into_iter().collect::<Vec<_>>().join("\n");
    if !registry_text.is_empty() {
        registry_text.push('\n');
    }
    let registry = Registry::parse(&registry_text).expect("fixture field registry");
    let mut model = JsonMap::new();
    model.insert("shapes".to_owned(), Json::Object(selected_shapes));
    let root = fixture_temp_root(service_name);
    let model_dir = root.join("models").join(service_name.replace('.', "-"));
    fs::create_dir_all(&model_dir).expect("create fixture model directory");
    fs::write(
        model_dir.join("model.json"),
        serde_json::to_vec(&Json::Object(model)).expect("encode model"),
    )
    .expect("write fixture model");
    let bytes = build_catalog(&root, &registry, "protocol-fixture").unwrap_or_else(|error| {
        panic!("compile fixture service {service_namespace}#{service_name}: {error}")
    });
    fs::remove_dir_all(root).expect("remove fixture model directory");
    bytes
}

fn fixture_temp_root(service_name: &str) -> PathBuf {
    let sequence = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "aws-wire-body-fixture-{}-{sequence}-{}",
        std::process::id(),
        service_name.replace('.', "-")
    ))
}

pub(super) fn fixture_endpoint(case: &Json) -> String {
    let host = case.get("host").and_then(Json::as_str);
    let host = match host {
        Some(host) => host,
        None => "example.com",
    };
    if let Some((scheme, authority_and_base_path)) = host.split_once("://") {
        format!("{scheme}://{authority_and_base_path}")
    } else {
        format!("https://{host}")
    }
}

pub(super) fn check_request(
    suite: &Suite,
    operation_name: &str,
    case: &Json,
    request: http::Request<bytes::Bytes>,
) {
    let label = format!("{} {} {}", suite.name, operation_name, case["id"]);
    let method = case["method"].as_str().expect("fixture method");
    assert_eq!(request.method().as_str(), method, "{label} method");
    assert_eq!(
        request
            .headers()
            .get("host")
            .and_then(|value| value.to_str().ok()),
        request
            .uri()
            .authority()
            .map(|authority| authority.as_str()),
        "{label} host"
    );
    if let Some(expected) = case.get("resolvedHost").and_then(Json::as_str) {
        assert_eq!(
            request.uri().host(),
            Some(expected),
            "{label} resolved host"
        );
    }
    let expected_path = case["uri"].as_str().unwrap_or("/");
    let expected_uri: Uri = expected_path.parse().expect("fixture request path");
    assert_eq!(request.uri().path(), expected_uri.path(), "{label} path");
    if let Some(headers) = case.get("headers").and_then(Json::as_object) {
        for (name, value) in headers {
            let expected = value.as_str().expect("fixture header text");
            let actual = request
                .headers()
                .get(name)
                .unwrap_or_else(|| panic!("{label}: missing header {name}"));
            assert_eq!(
                actual.as_bytes(),
                expected.as_bytes(),
                "{label} header {name}"
            );
        }
    }
    for name in case
        .get("forbidHeaders")
        .and_then(Json::as_array)
        .into_iter()
        .flatten()
    {
        let name = name.as_str().expect("forbidden header text");
        assert!(
            !request.headers().contains_key(name),
            "{label} forbidden header {name}"
        );
    }
    for name in case
        .get("requireHeaders")
        .and_then(Json::as_array)
        .into_iter()
        .flatten()
    {
        let name = name.as_str().expect("required header text");
        assert!(
            request.headers().contains_key(name),
            "{label} required header {name}"
        );
    }
    let decompressed = decompressed_body(&request, &label);
    let actual_body = decompressed.as_deref().unwrap_or(request.body().as_ref());
    if let Some(body_value) = case.get("body").and_then(Json::as_str) {
        if body_value.is_empty() {
            assert!(actual_body.is_empty(), "{label} expected empty body");
        } else if case.get("bodyMediaType").and_then(Json::as_str) == Some("application/cbor") {
            cbor_semantics::assert_semantically_equal(
                actual_body,
                &decode_base64(body_value),
                &label,
            );
        } else {
            let actual: Json = serde_json::from_slice(actual_body)
                .unwrap_or_else(|error| panic!("{label}: invalid JSON body: {error}"));
            let expected: Json = serde_json::from_str(body_value)
                .unwrap_or_else(|error| panic!("{label}: invalid expected JSON body: {error}"));
            assert!(
                json_semantics::equivalent(&actual, &expected),
                "{label} JSON body: {actual} != {expected}"
            );
        }
    } else if decompressed.is_some() {
        assert_compressed_body_matches_params(suite.protocol, actual_body, &case["params"], &label);
    }
}

fn decompressed_body(request: &http::Request<bytes::Bytes>, label: &str) -> Option<Vec<u8>> {
    let encoding = request
        .headers()
        .get(http::header::CONTENT_ENCODING)?
        .to_str()
        .ok()?;
    if !encoding
        .split(',')
        .any(|item| item.trim().eq_ignore_ascii_case("gzip"))
    {
        return None;
    }
    let mut decoder = GzDecoder::new(request.body().as_ref());
    let mut decoded = Vec::new();
    decoder
        .read_to_end(&mut decoded)
        .unwrap_or_else(|error| panic!("{label}: gzip body decode failed: {error}"));
    Some(decoded)
}

fn assert_compressed_body_matches_params(
    protocol: Protocol,
    body: &[u8],
    params: &Json,
    label: &str,
) {
    match protocol {
        Protocol::AwsJson10 | Protocol::AwsJson11 => {
            let actual: Json = serde_json::from_slice(body)
                .unwrap_or_else(|error| panic!("{label}: invalid decompressed JSON: {error}"));
            assert!(
                json_semantics::equivalent(&actual, params),
                "{label} decompressed JSON input parity"
            );
        }
        Protocol::RpcV2Cbor => assert_rpc_compressed_body(body, params, label),
        _ => panic!("{label}: compressed case uses a non-body protocol"),
    }
}

fn assert_rpc_compressed_body(body: &[u8], params: &Json, label: &str) {
    let expected = params["data"]
        .as_str()
        .unwrap_or_else(|| panic!("{label}: compressed RPCv2 fixture data is not a string"));
    cbor_semantics::assert_rpc_data(body, expected, label);
}
