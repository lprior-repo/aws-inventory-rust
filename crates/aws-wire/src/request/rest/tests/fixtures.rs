mod input;
mod xml_infoset;

use super::{CompressionPolicy, Protocol, RequestContext, RequestLimits};
use catalog_tool::smithy::{build_catalog, Registry};
use http::{header::CONTENT_TYPE, Uri};
use inventory_core::catalog::{Catalog, Operation, Service};
use serde_json::{Map as JsonMap, Value as Json};
use std::{collections::BTreeSet, fs, path::PathBuf, sync::atomic::{AtomicUsize, Ordering}};

static NEXT_TEMP: AtomicUsize = AtomicUsize::new(0);

struct Suite { name: &'static str, protocol_id: &'static str, protocol: Protocol, fixture: &'static str, clients: usize, servers: &'static [&'static str], events: usize }

#[test]
fn every_published_rest_request_client_fixture_runs() {
    let suites = [
        Suite { name: "restJson1", protocol_id: "aws.protocols#restJson1", protocol: Protocol::RestJson1,
            fixture: include_str!("../../../../tests/fixtures/protocol-tests/restJson1.json"), clients: 142,
            servers: &["AcceptHeaderStarRequestTest", "AcceptHeaderStarStarRequestTest", "RestJsonMustSupportParametersInContentType", "RestJsonEmptyInputAndEmptyOutputWithJson", "RestJsonHttpEmptyPrefixHeadersRequestServer", "RestJsonHttpPayloadTraitsWithBlobAcceptsAllContentTypes", "RestJsonHttpPayloadTraitsWithBlobAcceptsNoContentType", "RestJsonHttpPayloadTraitsWithBlobAcceptsAllAccepts", "RestJsonNoInputAllowsAccept", "RestJsonNoInputAndOutputAllowsAccept", "RestJsonServersAcceptStaticQueryParamAsEmptyString", "RestJsonServerPopulatesDefaultsWhenMissingInRequestBody", "RestJsonServerPopulatesNestedDefaultsWhenMissingInRequestBody", "RestJsonServersQueryParamsStringListMap", "RestJsonServersPutAllQueryParamsInMap", "RestJsonServersDontSerializeNullStructureValues", "RestJsonUnitInputAllowsAccept"], events: 102 },
        Suite { name: "restXml", protocol_id: "aws.protocols#restXml", protocol: Protocol::RestXml,
            fixture: include_str!("../../../../tests/fixtures/protocol-tests/restXml.json"), clients: 109,
            servers: &["RestXmlMustSupportParametersInContentType", "HttpEmptyPrefixHeadersRequestServer", "RestXmlServersQueryParamsStringListMap", "RestXmlServersPutAllQueryParamsInMap"], events: 0 },
    ];
    for suite in suites { run_suite(suite); }
}

fn run_suite(suite: Suite) {
    let fixture: Json = serde_json::from_str(suite.fixture).expect("published fixture JSON");
    let shapes = fixture["shapes"].as_object().expect("fixture shapes");
    let service_ids = shapes.iter().filter_map(|(id, shape)| (shape["type"] == "service").then(|| id.clone())).collect::<Vec<_>>();
    let mut clients = BTreeSet::new();
    let mut servers = BTreeSet::new();
    let mut events = 0usize;
    for service_id in service_ids {
        let service_shape = &shapes[&service_id];
        if service_shape["traits"].get(suite.protocol_id).is_none() { continue; }
        let bytes = compile_service(&fixture, &service_id);
        let catalog = Catalog::borrow(&bytes).expect("compiled fixture catalog");
        let service_name = service_id.rsplit('#').next().expect("service name").replace('.', "-");
        let service = catalog.service(&service_name).expect("service lookup").expect("compiled service");
        for operation_ref in service_shape["operations"].as_array().expect("service operation list") {
            let operation_id = operation_ref["target"].as_str().expect("operation target");
            let operation_name = operation_id.rsplit('#').next().expect("operation name");
            let operation = find_operation(service, operation_name);
            let operation_shape = &shapes[operation_id];
            let cases = operation_shape["traits"]["smithy.test#httpRequestTests"].as_array();
            if let Some(cases) = cases {
                for case in cases {
                    if case["appliesTo"] == "server" {
                        servers.insert(case["id"].as_str().expect("server case id").to_owned());
                        continue;
                    }
                    assert!(case["appliesTo"].as_str().map_or(true, |value| value == "client" || value == "both"));
                    let id = format!("{operation_id}::{}", case["id"].as_str().expect("client case id"));
                    assert!(clients.insert(id.clone()), "duplicate fixture ledger entry {id}");
                    run_case(suite, service, operation, operation_name, case);
                }
            }
            events += operation_shape["traits"]["smithy.test#eventStreamTests"].as_array().map_or(0, Vec::len);
        }
    }
    assert_eq!(clients.len(), suite.clients, "{} client/both fixture ledger", suite.name);
    assert_eq!(servers, suite.servers.iter().map(|id| (*id).to_owned()).collect());
    assert_eq!(events, suite.events, "{} event-stream cases outside buffered request scope", suite.name);
}

fn run_case(suite: Suite, service: Service<'_>, operation: Operation<'_>, operation_name: &str, case: &Json) {
    let id = case["id"].as_str().expect("fixture case id");
    let input = input::record(service, operation, case.get("params").unwrap_or(&Json::Null));
    let endpoint = fixture_endpoint(case);
    let context = RequestContext { service, operation, protocol: suite.protocol, input: input.root(), endpoint: &endpoint,
        limits: RequestLimits::default(), idempotency_token: None, compression: CompressionPolicy::default() };
    let request = super::super::encode(context).unwrap_or_else(|error| panic!("{} {operation_name} {id}: {error}", suite.name));
    let label = format!("{} {operation_name} {id}", suite.name);
    assert_eq!(request.method().as_str(), case["method"].as_str().expect("fixture method"), "{label} method");
    let expected_uri = case["uri"].as_str().unwrap_or("/").parse::<Uri>().expect("fixture URI");
    assert_eq!(request.uri().path_and_query(), expected_uri.path_and_query(), "{label} path/query");
    if let Some(resolved) = case.get("resolvedHost").and_then(Json::as_str) {
        assert_eq!(request.uri().authority().map(http::uri::Authority::host), Some(resolved), "{label} resolved host");
    }
    check_headers(case, &request, suite.protocol, &label);
    check_body(case, &request, suite.protocol, &label);
}

fn check_headers(case: &Json, request: &http::Request<bytes::Bytes>, protocol: Protocol, label: &str) {
    let expected = case.get("headers").and_then(Json::as_object);
    if let Some(expected) = expected {
        for (name, value) in expected {
            let actual = request.headers().get(name).unwrap_or_else(|| panic!("{label}: missing header {name}"));
            assert_eq!(actual.as_bytes(), value.as_str().expect("header fixture text").as_bytes(), "{label} header {name}");
        }
    }
    let allowed = expected.into_iter().flat_map(|map| map.keys().map(|name| name.to_ascii_lowercase()))
        .chain(["host".to_owned()]).collect::<BTreeSet<_>>();
    for name in request.headers().keys() {
        assert!(allowed.contains(name.as_str()) || name == CONTENT_TYPE, "{label}: unexpected header {name}");
    }
    for forbidden in ["authorization", "x-amz-date", "x-amz-content-sha256", "x-amz-security-token"] {
        assert!(!request.headers().contains_key(forbidden), "{label}: forbidden signing header {forbidden}");
    }
    if request.body().is_empty() {
        assert!(!request.headers().contains_key(CONTENT_TYPE), "{label}: empty body has Content-Type");
    } else if let Some(media_type) = case.get("bodyMediaType").and_then(Json::as_str) {
        assert_eq!(request.headers().get(CONTENT_TYPE).and_then(|value| value.to_str().ok()), Some(media_type), "{label} Content-Type");
    } else if case.get("body").and_then(Json::as_str).is_some_and(|body| !body.is_empty()) {
        let default = if protocol == Protocol::RestJson1 { "application/json" } else { "application/xml" };
        assert_eq!(request.headers().get(CONTENT_TYPE).and_then(|value| value.to_str().ok()), Some(default), "{label} Content-Type");
    }
    let expected_encoding = expected.and_then(|headers| headers.iter().find(|(name, _)| name.eq_ignore_ascii_case("content-encoding")).map(|(_, value)| value.as_str().expect("encoding header")));
    assert_eq!(request.headers().get("content-encoding").and_then(|value| value.to_str().ok()), expected_encoding, "{label} Content-Encoding");
}

fn check_body(case: &Json, request: &http::Request<bytes::Bytes>, protocol: Protocol, label: &str) {
    let wire_body = request.body().as_ref();
    let compressed = request.headers().get("content-encoding").and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.split(',').any(|encoding| encoding.trim().eq_ignore_ascii_case("gzip")));
    let uncompressed = if compressed { decompress_gzip(wire_body, label) } else { wire_body.to_vec() };
    let Some(expected) = case.get("body").and_then(Json::as_str) else {
        if let Some(payload) = case.get("params").and_then(|params| params.get("data")).and_then(Json::as_str) {
            assert_eq!(uncompressed, payload.as_bytes(), "{label} decompressed payload params");
        } else {
            assert!(uncompressed.is_empty(), "{label}: fixture omits body and payload");
        }
        return;
    };
    if expected.is_empty() { assert!(uncompressed.is_empty(), "{label}: expected empty body"); return; }
    if protocol == Protocol::RestJson1 {
        let actual: Json = serde_json::from_slice(&uncompressed).unwrap_or_else(|error| panic!("{label}: invalid JSON body: {error}"));
        let expected: Json = serde_json::from_str(expected).unwrap_or_else(|error| panic!("{label}: invalid fixture JSON body: {error}"));
        assert_eq!(actual, expected, "{label} JSON body");
    } else {
        assert!(xml_infoset::equal(&uncompressed, expected.as_bytes()), "{label} XML body");
    }
}

fn decompress_gzip(body: &[u8], label: &str) -> Vec<u8> {
    use std::io::Read;
    let mut output = Vec::new();
    flate2::read::GzDecoder::new(body).take(16 * 1024 * 1024 + 1).read_to_end(&mut output)
        .unwrap_or_else(|error| panic!("{label}: invalid gzip request body: {error}"));
    assert!(output.len() <= 16 * 1024 * 1024, "{label}: decompressed fixture body exceeds the catalog body bound");
    output
}


fn compile_service(fixture: &Json, service_id: &str) -> Vec<u8> {
    let shapes = fixture["shapes"].as_object().expect("fixture shapes");
    let mut selected = JsonMap::new();
    for (id, shape) in shapes {
        if shape["type"] != "service" || id == service_id { selected.insert(id.clone(), shape.clone()); }
    }
    let names = selected.values().filter_map(|shape| shape.get("members").and_then(Json::as_object))
        .flat_map(|members| members.keys().cloned()).collect::<BTreeSet<_>>();
    let registry = Registry::parse(&names.into_iter().collect::<Vec<_>>().join("\n")).expect("fixture field registry");
    let name = service_id.rsplit('#').next().expect("service name");
    let root = temp_root(name);
    let model_dir = root.join("models").join(name.replace('.', "-"));
    fs::create_dir_all(&model_dir).expect("fixture model directory");
    let model = JsonMap::from([("shapes".to_owned(), Json::Object(selected))]);
    fs::write(model_dir.join("model.json"), serde_json::to_vec(&model).expect("encode fixture model")).expect("write fixture model");
    let result = build_catalog(&root, &registry, "rest-protocol-fixture").unwrap_or_else(|error| panic!("compile {service_id}: {error}"));
    fs::remove_dir_all(root).expect("remove fixture model directory");
    result
}

fn find_operation(service: Service<'_>, name: &str) -> Operation<'_> {
    service.operations().expect("service operations").map(|item| item.expect("valid operation"))
        .find(|operation| operation.name().expect("operation name") == name).expect("fixture operation in catalog")
}

fn fixture_endpoint(case: &Json) -> String {
    let host = case.get("host").and_then(Json::as_str).unwrap_or("example.com");
    let authority = host.split_once("://").map_or(host, |(_, value)| value).split('/').next().unwrap_or(host);
    format!("https://{authority}")
}

fn temp_root(service: &str) -> PathBuf {
    let sequence = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("aws-wire-rest-fixture-{}-{sequence}-{}", std::process::id(), service.replace('.', "-")))
}
