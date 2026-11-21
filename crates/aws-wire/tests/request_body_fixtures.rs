#[path = "request_body_cases/support.rs"]
mod request_body_fixtures_support;
#[path = "request_body_cases/numeric.rs"]
mod request_body_numeric_values;
#[path = "request_body_cases/values.rs"]
mod request_body_values;
use aws_wire::request::{
    CompressionPolicy, EncodeError, Protocol, RequestContext, RequestLimits, body,
};
use inventory_core::{
    catalog::{Catalog, Operation, Service},
    record::{Key, Record, Value},
};
use request_body_fixtures_support::{check_request, compile_fixture_service, fixture_endpoint};
use request_body_values::{BoxError, expect_ok, input_record};
use serde_json::Value as Json;
use std::collections::BTreeMap;

struct Suite {
    name: &'static str,
    protocol_id: &'static str,
    protocol: Protocol,
    source: &'static str,
    expected_client: usize,
    expected_server: usize,
}

#[test]
fn published_aws_json_10_client_request_fixtures_match() -> Result<(), BoxError> {
    run_suite(&Suite {
        name: "awsJson1_0",
        protocol_id: "aws.protocols#awsJson1_0",
        protocol: Protocol::AwsJson10,
        source: include_str!("fixtures/protocol-tests/awsJson1_0.json"),
        expected_client: 29,
        expected_server: 4,
    })
}

#[test]
fn published_aws_json_11_client_request_fixtures_match() -> Result<(), BoxError> {
    run_suite(&Suite {
        name: "awsJson1_1",
        protocol_id: "aws.protocols#awsJson1_1",
        protocol: Protocol::AwsJson11,
        source: include_str!("fixtures/protocol-tests/awsJson1_1.json"),
        expected_client: 57,
        expected_server: 3,
    })
}

#[test]
fn published_rpc_v2_cbor_client_request_fixtures_match() -> Result<(), BoxError> {
    run_suite(&Suite {
        name: "rpcv2Cbor",
        protocol_id: "smithy.protocols#rpcv2Cbor",
        protocol: Protocol::RpcV2Cbor,
        source: include_str!("fixtures/protocol-tests/rpcv2Cbor.json"),
        expected_client: 31,
        expected_server: 14,
    })
}

fn find_operation<'a>(service: Service<'a>, name: &str) -> Result<Operation<'a>, BoxError> {
    for item in expect_ok(service.operations(), "operations")? {
        let item = expect_ok(item, "valid operation")?;
        if expect_ok(item.name(), "operation name")? == name {
            return Ok(item);
        }
    }
    Err(format!("fixture operation {name} is in compiled service").into())
}

fn run_suite(suite: &Suite) -> Result<(), BoxError> {
    let fixture: Json = expect_ok(
        serde_json::from_str(suite.source),
        "published fixture is JSON",
    )?;
    let shapes = fixture
        .pointer("/shapes")
        .and_then(Json::as_object)
        .ok_or("fixture shapes")?;
    let service_ids: Vec<String> = shapes
        .iter()
        .filter(|(_, shape)| shape["type"] == "service" && shape["version"].is_string())
        .map(|(id, _)| id.clone())
        .collect();
    let mut client_cases = 0usize;
    let mut server_cases = 0usize;
    let mut exercised_limit = false;
    let mut exercised_malformed = false;
    for service_id in service_ids {
        let service_shape = shapes.get(&service_id).ok_or("service shape")?;
        if service_shape["traits"].get(suite.protocol_id).is_none() {
            continue;
        }
        let bytes = compile_fixture_service(&fixture, &service_id)?;
        let catalog = expect_ok(Catalog::borrow(&bytes), "fixture catalog validates")?;
        let service_name = service_id.rsplit('#').next().ok_or("service name")?;
        let model_name = service_name.replace('.', "-");
        let service =
            expect_ok(catalog.service(&model_name), "service lookup")?.ok_or("service exists")?;
        let operation_ids = service_shape["operations"]
            .as_array()
            .ok_or("service operations")?;
        for operation_ref in operation_ids {
            let operation_id = operation_ref["target"].as_str().ok_or("operation target")?;
            let operation_name = operation_id.rsplit('#').next().ok_or("operation name")?;
            let operation = find_operation(service, operation_name)?;
            let operation_shape = shapes.get(operation_id).ok_or("operation shape")?;
            let cases = operation_shape
                .pointer("/traits/smithy.test#httpRequestTests")
                .and_then(Json::as_array)
                .into_iter()
                .flatten();
            for case in cases {
                let applies_to = case["appliesTo"].as_str().unwrap_or("both");
                if applies_to == "server" {
                    server_cases = server_cases.saturating_add(1);
                    continue;
                }
                assert!(
                    applies_to == "client" || applies_to == "both",
                    "{} {}: unknown appliesTo",
                    suite.name,
                    case["id"]
                );
                client_cases = client_cases.saturating_add(1);
                let input = input_record(
                    service,
                    operation,
                    case.get("params").unwrap_or(&Json::Null),
                )?;
                let endpoint = fixture_endpoint(case);
                let context = RequestContext {
                    service,
                    operation,
                    protocol: suite.protocol,
                    input: input.root(),
                    endpoint: &endpoint,
                    limits: RequestLimits::default(),
                    idempotency_token: None,
                    compression: CompressionPolicy::default(),
                    addressing: None,
                };
                let request = body::encode(context).map_err(|error| {
                    format!("{} {} {}: {error}", suite.name, operation_name, case["id"])
                })?;
                if suite.protocol == Protocol::RpcV2Cbor {
                    let query_mode = request
                        .headers()
                        .get("x-amzn-query-mode")
                        .and_then(|value| value.to_str().ok());
                    let expected_query_mode = expect_ok(
                        service.aws_query_compatible(),
                        "query compatibility metadata",
                    )?
                    .then_some("true");
                    assert_eq!(
                        query_mode, expected_query_mode,
                        "{} {}: query mode header",
                        suite.name, case["id"]
                    );
                }
                let body_bytes = request.body().len();
                check_request(suite, operation_name, case, request)?;
                if !exercised_limit && body_bytes > 1 {
                    let limited = RequestContext {
                        limits: expect_ok(
                            RequestLimits::new(1, 65_536, 16_384),
                            "valid low limit",
                        )?,
                        ..context
                    };
                    assert!(
                        matches!(body::encode(limited), Err(EncodeError::Limit("body"))),
                        "{} {}: body limit must be enforced by encoder",
                        suite.name,
                        case["id"]
                    );
                    exercised_limit = true;
                }
                if !exercised_malformed {
                    let malformed = expect_ok(
                        Record::encode(&Value::Map(BTreeMap::from([(
                            Key::Name("__unknown_fixture_field".to_owned()),
                            Value::Text("x".to_owned()),
                        )]))),
                        "malformed test record",
                    )?;
                    let malformed_context = RequestContext {
                        input: malformed.root(),
                        ..context
                    };
                    assert!(
                        matches!(
                            body::encode(malformed_context),
                            Err(EncodeError::InvalidValue { .. })
                        ),
                        "{} {}: malformed record must be rejected by encoder",
                        suite.name,
                        case["id"]
                    );
                    exercised_malformed = true;
                }
            }
        }
    }
    assert_eq!(
        client_cases, suite.expected_client,
        "{} client/both request accounting",
        suite.name
    );
    assert_eq!(
        server_cases, suite.expected_server,
        "{} server-only request accounting",
        suite.name
    );
    assert!(exercised_limit, "{} body limit case ran", suite.name);
    assert!(
        exercised_malformed,
        "{} malformed record case ran",
        suite.name
    );
    Ok(())
}
