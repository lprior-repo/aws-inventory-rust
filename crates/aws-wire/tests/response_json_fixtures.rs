#[path = "../src/response/json/fixture_catalog.rs"]
mod fixture_catalog;
#[path = "../src/response/json/fixture_oracle.rs"]
mod fixture_oracle;
use aws_wire::{
    request::Protocol,
    response::{self, DecodeError, DecodeLimits, ResponseContext, json},
};
use http::{
    HeaderMap, StatusCode,
    header::{HeaderName, HeaderValue},
};
use inventory_core::catalog::{Catalog, HttpBinding, Target};
use serde_json::Value as Json;

struct Suite {
    name: &'static str,
    protocol_id: &'static str,
    protocol: Protocol,
    fixture: &'static str,
    client_cases: usize,
    server_cases: usize,
}

#[test]
fn published_aws_json_10_response_cases_decode() {
    run_suite(Suite {
        name: "awsJson1_0",
        protocol_id: "aws.protocols#awsJson1_0",
        protocol: Protocol::AwsJson10,
        fixture: include_str!("fixtures/protocol-tests/awsJson1_0.json"),
        client_cases: 25,
        server_cases: 2,
    });
}

#[test]
fn published_aws_json_11_response_cases_decode() {
    run_suite(Suite {
        name: "awsJson1_1",
        protocol_id: "aws.protocols#awsJson1_1",
        protocol: Protocol::AwsJson11,
        fixture: include_str!("fixtures/protocol-tests/awsJson1_1.json"),
        client_cases: 48,
        server_cases: 1,
    });
}

#[test]
fn published_rest_json_1_response_cases_decode() {
    run_suite(Suite {
        name: "restJson1",
        protocol_id: "aws.protocols#restJson1",
        protocol: Protocol::RestJson1,
        fixture: include_str!("fixtures/protocol-tests/restJson1.json"),
        client_cases: 94,
        server_cases: 8,
    });
}

fn run_suite(suite: Suite) {
    let fixture: Json = serde_json::from_str(suite.fixture).expect("valid published fixture");
    let shapes = fixture["shapes"].as_object().expect("fixture shape table");
    let service_ids: Vec<String> = shapes
        .iter()
        .filter_map(|(id, shape)| {
            (shape["type"] == "service"
                && shape["version"].is_string()
                && shape["traits"].get(suite.protocol_id).is_some())
            .then(|| id.clone())
        })
        .collect();
    let mut client_count = 0usize;
    let mut server_count = 0usize;
    let mut malformed_tested = false;
    let mut body_limit_tested = false;
    let mut node_limit_tested = false;
    let mut ignored_malformed_tested = false;
    let mut depth_limit_tested = false;
    let ignored = br#"{"unknown":{"nested":[{"value":1},{"value":2}]}}"#;
    for service_id in service_ids {
        let bytes = fixture_catalog::compile_service(&fixture, &service_id);
        let catalog = Catalog::borrow(&bytes).expect("fixture catalog validates");
        let service_name = service_id.rsplit('#').next().expect("service name");
        let model_name = service_name.replace('.', "-");
        let service = catalog
            .service(&model_name)
            .expect("service lookup")
            .expect("fixture service exists");
        let service_shape = &shapes[&service_id];
        for operation_ref in service_shape["operations"]
            .as_array()
            .expect("service operations")
        {
            let operation_id = operation_ref["target"].as_str().expect("operation target");
            let operation_name = operation_id.rsplit('#').next().expect("operation name");
            let operation = service
                .operations()
                .expect("operation iterator")
                .map(|entry| entry.expect("valid operation"))
                .find(|item| item.name().expect("operation name") == operation_name)
                .expect("catalog operation");
            let body_decodable = supports_object_body(service, operation);
            let cases = shapes[operation_id]["traits"]["smithy.test#httpResponseTests"]
                .as_array()
                .into_iter()
                .flatten();
            for case in cases {
                if case["appliesTo"] == "server" {
                    server_count += 1;
                    continue;
                }
                let id = case["id"].as_str().expect("fixture case id");
                let body = case["body"].as_str().unwrap_or("").as_bytes();
                let mut headers = HeaderMap::new();
                if let Some(values) = case["headers"].as_object() {
                    for (name, value) in values {
                        let header_name = HeaderName::from_bytes(name.as_bytes())
                            .expect("valid fixture header name");
                        if let Some(text) = value.as_str() {
                            headers.append(
                                header_name,
                                HeaderValue::from_str(text).expect("valid fixture header value"),
                            );
                        }
                    }
                }
                let status = case["code"]
                    .as_u64()
                    .and_then(|code| u16::try_from(code).ok())
                    .and_then(|code| StatusCode::from_u16(code).ok())
                    .unwrap_or(StatusCode::OK);
                if !status.is_success() {
                    let context = ResponseContext {
                        service,
                        operation,
                        protocol: suite.protocol,
                        body,
                        headers: &headers,
                        status,
                        limits: DecodeLimits::default(),
                    };
                    assert!(matches!(
                        json::decode(context),
                        Err(DecodeError::Invalid(
                            "non-success response belongs to error decoder"
                        ))
                    ));
                    client_count += 1;
                    continue;
                }
                let context = ResponseContext {
                    service,
                    operation,
                    protocol: suite.protocol,
                    body,
                    headers: &headers,
                    status,
                    limits: DecodeLimits::default(),
                };
                let record = response::decode(context)
                    .unwrap_or_else(|error| panic!("{} {id}: {error}", suite.name));
                fixture_oracle::assert_fixture_root(
                    &record,
                    service,
                    operation.output().expect("output"),
                    &case["params"],
                    &format!("{} {id}", suite.name),
                );
                if !body_limit_tested && body.len() > 1 {
                    let limits = DecodeLimits::new(body.len() - 1, 65_536, 32)
                        .expect("valid body test limit");
                    assert!(
                        matches!(
                            json::decode(ResponseContext { limits, ..context }),
                            Err(DecodeError::Limit("response body"))
                        ),
                        "{} {id} body cap must reach decoder",
                        suite.name
                    );
                    body_limit_tested = true;
                }
                if body_decodable && !malformed_tested && !body.is_empty() {
                    let mut malformed = body.to_vec();
                    malformed.push(b'!');
                    assert!(
                        json::decode(ResponseContext {
                            body: &malformed,
                            ..context
                        })
                        .is_err(),
                        "{} {id} malformed syntax must reach decoder",
                        suite.name
                    );
                    malformed_tested = true;
                }
                if body_decodable && !ignored_malformed_tested {
                    let malformed = br#"{"unknown":{"nested":[}"#;
                    assert!(matches!(
                        json::decode(ResponseContext {
                            body: malformed,
                            ..context
                        }),
                        Err(DecodeError::Invalid("JSON syntax"))
                    ));
                    ignored_malformed_tested = true;
                }
                if body_decodable && !node_limit_tested {
                    let limits =
                        DecodeLimits::new(16_777_216, 1, 32).expect("valid node test limit");
                    assert!(
                        matches!(
                            json::decode(ResponseContext {
                                body: ignored,
                                limits,
                                ..context
                            }),
                            Err(DecodeError::Limit("response nodes"))
                        ),
                        "{} ignored node cap must reach decoder",
                        suite.name
                    );
                    node_limit_tested = true;
                }
                if body_decodable && !depth_limit_tested {
                    let limits =
                        DecodeLimits::new(16_777_216, 65_536, 1).expect("valid depth test limit");
                    assert!(
                        matches!(
                            json::decode(ResponseContext {
                                body: ignored,
                                limits,
                                ..context
                            }),
                            Err(DecodeError::Limit("response depth"))
                        ),
                        "{} ignored depth cap must reach decoder",
                        suite.name
                    );
                    depth_limit_tested = true;
                }
                client_count += 1;
            }
        }
    }
    assert_eq!(
        client_count, suite.client_cases,
        "{} client response fixture accounting",
        suite.name
    );
    assert_eq!(
        server_count, suite.server_cases,
        "{} server response fixture accounting",
        suite.name
    );
    assert!(
        malformed_tested,
        "{} malformed input accounting",
        suite.name
    );
    assert!(body_limit_tested, "{} body limit accounting", suite.name);
    assert!(node_limit_tested, "{} node limit accounting", suite.name);
    assert!(depth_limit_tested, "{} depth limit accounting", suite.name);
    assert!(
        ignored_malformed_tested,
        "{} ignored malformed accounting",
        suite.name
    );
}

fn supports_object_body(
    service: inventory_core::catalog::Service<'_>,
    operation: inventory_core::catalog::Operation<'_>,
) -> bool {
    let Target::Shape(index) = operation.output().expect("operation output") else {
        return false;
    };
    let shape = service
        .shape(index)
        .expect("output shape lookup")
        .expect("output shape exists");
    !shape.members().expect("output members").any(|item| {
        item.expect("output member")
            .wire()
            .expect("output member wire")
            .binding()
            .expect("output member binding")
            == HttpBinding::Payload
    })
}
