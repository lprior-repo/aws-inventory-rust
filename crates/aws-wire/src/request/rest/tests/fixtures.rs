mod assertions;
#[path = "input.rs"]
mod input;
mod model;
mod protocol;
mod runner;
#[path = "xml_infoset.rs"]
mod xml_infoset;

use crate::protocol_fixtures::load;
use crate::request::{
    AddressingStyle, CompressionPolicy, Protocol, RequestContext, RequestLimits, S3EndpointConfig,
    s3_endpoint,
};
use http::Uri;
use inventory_core::catalog::{Operation, Service};
use serde_json::Value as Json;

pub(super) type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[derive(Clone, Copy)]
pub(super) struct Suite<'a> {
    pub(super) name: &'static str,
    pub(super) protocol_id: &'static str,
    pub(super) protocol: Protocol,
    pub(super) fixture: &'a str,
    pub(super) clients: usize,
    pub(super) servers: &'static [&'static str],
    pub(super) events: usize,
}

#[test]
fn every_published_rest_request_client_fixture_runs() -> TestResult {
    let restjson = load("restJson1.json")?;
    let restxml = load("restXml.json")?;
    let suites = [
        Suite {
            name: "restJson1",
            protocol_id: "aws.protocols#restJson1",
            protocol: Protocol::RestJson1,
            fixture: &restjson,
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
            fixture: &restxml,
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
    let mut failures = Vec::new();
    for suite in suites {
        failures.extend(runner::run_suite(suite)?);
    }
    assert!(
        failures.is_empty(),
        "{} failing published cases: {failures:#?}",
        failures.len()
    );
    Ok(())
}

fn run_case(
    suite: Suite,
    service: Service<'_>,
    operation: Operation<'_>,
    operation_name: &str,
    case: &Json,
) -> TestResult {
    let request = encode_case(suite, service, operation, operation_name, case, true)?;
    let plain = encode_case(suite, service, operation, operation_name, case, false)?;
    let label = format!(
        "{} {operation_name} {}",
        suite.name,
        case["id"].as_str().ok_or("fixture case id")?
    );
    check_method(case, &request, &label)?;
    check_uri(case, &request, &label)?;
    assertions::check_headers(case, &request, &label)?;
    assertions::check_body(
        case,
        &request,
        plain.body(),
        suite.protocol,
        &label,
        service,
        operation,
    )
}

fn encode_case(
    suite: Suite,
    service: Service<'_>,
    operation: Operation<'_>,
    operation_name: &str,
    case: &Json,
    compress: bool,
) -> TestResult<http::Request<bytes::Bytes>> {
    let id = case["id"].as_str().ok_or("fixture case id")?;
    let input = input::record(
        service,
        operation,
        case.get("params").unwrap_or(&Json::Null),
    )?;
    let base_endpoint = fixture_endpoint(case);
    let endpoint = fixture_addressing(service, case, &base_endpoint)?;
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
        .map_err(|error| format!("{} {operation_name} {id}: {error}", suite.name).into())
}

fn check_method(case: &Json, request: &http::Request<bytes::Bytes>, label: &str) -> TestResult {
    assert_eq!(
        request.method().as_str(),
        case["method"].as_str().ok_or("fixture method")?,
        "{label} method"
    );
    Ok(())
}

fn check_uri(case: &Json, request: &http::Request<bytes::Bytes>, label: &str) -> TestResult {
    let expected = case["uri"].as_str().unwrap_or("/").parse::<Uri>()?;
    assert_eq!(request.uri().path(), expected.path(), "{label} path");

    let mut expected_query = expected
        .query()
        .into_iter()
        .flat_map(|query| query.split('&'))
        .map(str::to_owned)
        .collect::<Vec<_>>();
    for parameter in case
        .get("queryParams")
        .and_then(Json::as_array)
        .into_iter()
        .flatten()
    {
        expected_query.push(
            parameter
                .as_str()
                .ok_or("fixture query parameter")?
                .to_owned(),
        );
    }
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
    Ok(())
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

fn find_operation<'a>(service: Service<'a>, name: &str) -> TestResult<Operation<'a>> {
    for item in service.operations()? {
        let operation = item?;
        if operation.name()? == name {
            return Ok(operation);
        }
    }
    Err("fixture operation in catalog".into())
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
) -> TestResult<Option<crate::request::ResolvedAddressing<'a>>> {
    if service.endpoint_prefix()? != Some("s3") {
        return Ok(None);
    }
    let Some(vendor) = case.get("vendorParams") else {
        return Ok(None);
    };
    let Some(client) = vendor
        .get("scopedConfig")
        .and_then(|config| config.get("client"))
    else {
        return Ok(None);
    };
    let Some(region) = client.get("region").and_then(Json::as_str) else {
        return Ok(None);
    };
    let Some(bucket_value) = case.get("params").and_then(|params| params.get("Bucket")) else {
        return Ok(None);
    };
    let bucket = bucket_value.as_str();
    let config = fixture_s3_config(endpoint, region, bucket, client, vendor)?;
    Ok(Some(s3_endpoint(config)?))
}

fn fixture_s3_config<'a>(
    endpoint: &'a str,
    region: &'a str,
    bucket: Option<&'a str>,
    client: &Json,
    vendor: &Json,
) -> TestResult<S3EndpointConfig<'a>> {
    let s3 = client.get("s3").unwrap_or(&Json::Null);
    let operation = vendor
        .get("scopedConfig")
        .and_then(|value| value.get("operation"))
        .and_then(|value| value.get("s3"))
        .unwrap_or(&Json::Null);
    Ok(S3EndpointConfig {
        endpoint,
        region,
        bucket,
        style: fixture_style(s3.get("addressing_style"))?,
        dualstack: fixture_flag(s3.get("use_dualstack_endpoint"))?,
        accelerate: fixture_flag(s3.get("use_accelerate_endpoint"))?,
        operation_style: fixture_style(operation.get("addressing_style"))?,
        explicit_endpoint: false,
    })
}

fn fixture_style(value: Option<&Json>) -> TestResult<Option<AddressingStyle>> {
    let Some(value) = value else {
        return Ok(None);
    };
    match value.as_str().ok_or("fixture S3 addressing style")? {
        "auto" => Ok(Some(AddressingStyle::Auto)),
        "path" => Ok(Some(AddressingStyle::Path)),
        "virtual" => Ok(Some(AddressingStyle::Virtual)),
        _ => Err("unknown S3 addressing style".into()),
    }
}

fn fixture_flag(value: Option<&Json>) -> TestResult<bool> {
    match value {
        None => Ok(false),
        Some(value) => value
            .as_bool()
            .ok_or_else(|| "fixture S3 endpoint flag".into()),
    }
}
