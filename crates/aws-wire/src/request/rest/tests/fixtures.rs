mod assertions;
#[path = "input.rs"]
mod input;
mod model;
mod runner;
#[path = "xml_infoset.rs"]
mod xml_infoset;

use crate::request::{
    AddressingStyle, CompressionPolicy, Protocol, RequestContext, RequestLimits, S3EndpointConfig,
    s3_endpoint,
};
use http::Uri;
use inventory_core::catalog::{Operation, Service};
use serde_json::Value as Json;

#[derive(Clone, Copy)]
pub(super) struct Suite {
    pub(super) name: &'static str,
    pub(super) protocol_id: &'static str,
    pub(super) protocol: Protocol,
    pub(super) fixture: &'static str,
    pub(super) clients: usize,
    pub(super) servers: &'static [&'static str],
    pub(super) events: usize,
}
#[test]
fn every_published_rest_request_client_fixture_runs() {
    let suites = [
        Suite {
            name: "restJson1",
            protocol_id: "aws.protocols#restJson1",
            protocol: Protocol::RestJson1,
            fixture: include_str!("../../../../tests/fixtures/protocol-tests/restJson1.json"),
            clients: 142,
            servers: &[
                "RestJsonMustSupportParametersInContentType",
                "RestJsonEmptyInputAndEmptyOutputWithJson",
                "RestJsonHttpEmptyPrefixHeadersRequestServer",
                "RestJsonHttpPayloadTraitsWithBlobAcceptsAllContentTypes",
                "RestJsonHttpPayloadTraitsWithBlobAcceptsNoContentType",
                "RestJsonHttpPayloadTraitsWithBlobAcceptsAllAccepts",
                "RestJsonNoInputAllowsAccept",
                "RestJsonNoInputAndOutputAllowsAccept",
                "RestJsonServersAcceptStaticQueryParamAsEmptyString",
                "RestJsonServerPopulatesDefaultsWhenMissingInRequestBody",
                "RestJsonServerPopulatesNestedDefaultsWhenMissingInRequestBody",
                "RestJsonServersQueryParamsStringListMap",
                "RestJsonServersPutAllQueryParamsInMap",
                "RestJsonServersDontSerializeNullStructureValues",
                "RestJsonUnitInputAllowsAccept",
            ],
            events: 102,
        },
        Suite {
            name: "restXml",
            protocol_id: "aws.protocols#restXml",
            protocol: Protocol::RestXml,
            fixture: include_str!("../../../../tests/fixtures/protocol-tests/restXml.json"),
            clients: 109,
            servers: &[
                "RestXmlMustSupportParametersInContentType",
                "HttpEmptyPrefixHeadersRequestServer",
                "RestXmlServersQueryParamsStringListMap",
                "RestXmlServersPutAllQueryParamsInMap",
            ],
            events: 0,
        },
    ];
    let failures = suites
        .into_iter()
        .flat_map(runner::run_suite)
        .collect::<Vec<_>>();
    assert!(
        failures.is_empty(),
        "{} failing published cases: {failures:#?}",
        failures.len()
    );
}

fn run_case(
    suite: Suite,
    service: Service<'_>,
    operation: Operation<'_>,
    operation_name: &str,
    case: &Json,
) {
    let request = encode_case(suite, service, operation, operation_name, case, true);
    let plain = encode_case(suite, service, operation, operation_name, case, false);
    let label = format!(
        "{} {operation_name} {}",
        suite.name,
        case["id"].as_str().expect("fixture case id")
    );
    check_method(case, &request, &label);
    check_uri(case, &request, &label);
    assertions::check_headers(case, &request, &label);
    assertions::check_body(
        case,
        &request,
        plain.body(),
        suite.protocol,
        &label,
        service,
        operation,
    );
}

fn encode_case(
    suite: Suite,
    service: Service<'_>,
    operation: Operation<'_>,
    operation_name: &str,
    case: &Json,
    compress: bool,
) -> http::Request<bytes::Bytes> {
    let id = case["id"].as_str().expect("fixture case id");
    let input = input::record(
        service,
        operation,
        case.get("params").unwrap_or(&Json::Null),
    );
    let base_endpoint = fixture_endpoint(case);
    let endpoint = fixture_addressing(service, case, &base_endpoint);
    let endpoint_url = endpoint
        .as_ref()
        .map_or(base_endpoint.as_str(), |value| value.endpoint.as_ref());
    let addressing = endpoint.as_ref().and_then(|value| value.addressing);
    let context = RequestContext {
        service,
        operation,
        protocol: suite.protocol,
        input: input.root(),
        endpoint: endpoint_url,
        limits: RequestLimits::default(),
        idempotency_token: Some("00000000-0000-4000-8000-000000000000"),
        compression: if compress {
            CompressionPolicy::default()
        } else {
            CompressionPolicy::Disabled
        },
        addressing,
    };
    super::super::encode(context)
        .unwrap_or_else(|error| panic!("{} {operation_name} {id}: {error}", suite.name))
}

fn check_method(case: &Json, request: &http::Request<bytes::Bytes>, label: &str) {
    assert_eq!(
        request.method().as_str(),
        case["method"].as_str().expect("fixture method"),
        "{label} method"
    );
}

fn check_uri(case: &Json, request: &http::Request<bytes::Bytes>, label: &str) {
    let expected = case["uri"]
        .as_str()
        .unwrap_or("/")
        .parse::<Uri>()
        .expect("fixture URI");
    assert_eq!(request.uri().path(), expected.path(), "{label} path");

    let mut expected_query = expected
        .query()
        .into_iter()
        .flat_map(|query| query.split('&'))
        .map(str::to_owned)
        .collect::<Vec<_>>();
    expected_query.extend(
        case.get("queryParams")
            .and_then(Json::as_array)
            .into_iter()
            .flatten()
            .map(|parameter| {
                parameter
                    .as_str()
                    .expect("fixture query parameter")
                    .to_owned()
            }),
    );
    let mut actual_query = request
        .uri()
        .query()
        .into_iter()
        .flat_map(|query| query.split('&'))
        .map(str::to_owned)
        .collect::<Vec<_>>();
    expected_query.sort_unstable();
    actual_query.sort_unstable();
    assert_eq!(actual_query, expected_query, "{label} query parameters");
    check_resolved_host(case, request, label);
}

fn check_resolved_host(case: &Json, request: &http::Request<bytes::Bytes>, label: &str) {
    if let Some(resolved) = case.get("resolvedHost").and_then(Json::as_str) {
        assert_eq!(
            request.uri().authority().map(http::uri::Authority::host),
            Some(resolved),
            "{label} resolved host"
        );
    }
}
fn find_operation<'a>(service: Service<'a>, name: &str) -> Operation<'a> {
    service
        .operations()
        .expect("service operations")
        .map(|item| item.expect("valid operation"))
        .find(|operation| operation.name().expect("operation name") == name)
        .expect("fixture operation in catalog")
}

fn fixture_endpoint(case: &Json) -> String {
    let host = case
        .get("host")
        .and_then(Json::as_str)
        .unwrap_or("example.com");
    if host.contains("://") {
        host.to_owned()
    } else {
        format!("https://{host}")
    }
}
fn fixture_addressing<'a>(
    service: Service<'_>,
    case: &'a Json,
    endpoint: &'a str,
) -> Option<crate::request::ResolvedAddressing<'a>> {
    if service.endpoint_prefix().expect("service endpoint prefix") != Some("s3") {
        return None;
    }
    let vendor = case.get("vendorParams")?;
    let client = vendor.get("scopedConfig")?.get("client")?;
    let region = client.get("region")?.as_str()?;
    let bucket = case.get("params")?.get("Bucket")?.as_str();
    Some(
        s3_endpoint(fixture_s3_config(endpoint, region, bucket, client, vendor))
            .expect("valid S3 endpoint fixture configuration"),
    )
}

fn fixture_s3_config<'a>(
    endpoint: &'a str,
    region: &'a str,
    bucket: Option<&'a str>,
    client: &Json,
    vendor: &Json,
) -> S3EndpointConfig<'a> {
    let s3 = client.get("s3").unwrap_or(&Json::Null);
    let operation = vendor
        .get("scopedConfig")
        .and_then(|value| value.get("operation"))
        .and_then(|value| value.get("s3"))
        .unwrap_or(&Json::Null);
    S3EndpointConfig {
        endpoint,
        region,
        bucket,
        style: fixture_style(s3.get("addressing_style")),
        dualstack: fixture_flag(s3.get("use_dualstack_endpoint")),
        accelerate: fixture_flag(s3.get("use_accelerate_endpoint")),
        operation_style: fixture_style(operation.get("addressing_style")),
        explicit_endpoint: false,
    }
}

fn fixture_style(value: Option<&Json>) -> Option<AddressingStyle> {
    let Some(value) = value else {
        return None;
    };
    match value.as_str().expect("fixture S3 addressing style") {
        "auto" => Some(AddressingStyle::Auto),
        "path" => Some(AddressingStyle::Path),
        "virtual" => Some(AddressingStyle::Virtual),
        _ => panic!("unknown S3 addressing style"),
    }
}

fn fixture_flag(value: Option<&Json>) -> bool {
    value.map_or(false, |value| {
        value.as_bool().expect("fixture S3 endpoint flag")
    })
}
