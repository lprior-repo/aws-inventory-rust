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

type TestResult = Result<(), Box<dyn std::error::Error>>;

struct Suite {
    name: &'static str,
    protocol_id: &'static str,
    protocol: Protocol,
    fixture: &'static str,
    client_cases: usize,
    server_cases: usize,
}

#[test]
fn published_aws_json_10_response_cases_decode() -> TestResult {
    run_suite(Suite {
        name: "awsJson1_0",
        protocol_id: "aws.protocols#awsJson1_0",
        protocol: Protocol::AwsJson10,
        fixture: include_str!("fixtures/protocol-tests/awsJson1_0.json"),
        client_cases: 25,
        server_cases: 2,
    })
}

#[test]
fn published_aws_json_11_response_cases_decode() -> TestResult {
    run_suite(Suite {
        name: "awsJson1_1",
        protocol_id: "aws.protocols#awsJson1_1",
        protocol: Protocol::AwsJson11,
        fixture: include_str!("fixtures/protocol-tests/awsJson1_1.json"),
        client_cases: 48,
        server_cases: 1,
    })
}

#[test]
fn published_rest_json_1_response_cases_decode() -> TestResult {
    run_suite(Suite {
        name: "restJson1",
        protocol_id: "aws.protocols#restJson1",
        protocol: Protocol::RestJson1,
        fixture: include_str!("fixtures/protocol-tests/restJson1.json"),
        client_cases: 94,
        server_cases: 8,
    })
}

fn run_suite(suite: Suite) -> TestResult {
    let fixture: Json = serde_json::from_str(suite.fixture)?;
    let shapes = fixture
        .get("shapes")
        .and_then(Json::as_object)
        .ok_or("fixture shape table")?;
    let mut service_ids: Vec<String> = Vec::new();
    for (id, shape) in shapes {
        if shape["type"] == "service"
            && shape["version"].is_string()
            && shape["traits"].get(suite.protocol_id).is_some()
        {
            service_ids.push(id.clone());
        }
    }
    let mut client_count = 0usize;
    let mut server_count = 0usize;
    let mut malformed_tested = false;
    let mut body_limit_tested = false;
    let mut node_limit_tested = false;
    let mut ignored_malformed_tested = false;
    let mut depth_limit_tested = false;
    let ignored = br#"{"unknown":{"nested":[{"value":1},{"value":2}]}}"#;
    for service_id in service_ids {
        let bytes = fixture_catalog::compile_service(&fixture, &service_id)?;
        let catalog = Catalog::borrow(&bytes)?;
        let service_name = service_id.rsplit('#').next().ok_or("service name")?;
        let model_name = service_name.replace('.', "-");
        let service = catalog
            .service(&model_name)?
            .ok_or("fixture service exists")?;
        let service_shape = shapes.get(&service_id).ok_or("service shape")?;
        for operation_ref in service_shape["operations"]
            .as_array()
            .ok_or("service operations")?
        {
            let operation_id = operation_ref["target"].as_str().ok_or("operation target")?;
            let operation_name = operation_id.rsplit('#').next().ok_or("operation name")?;
            let mut found = None;
            for entry in service.operations()? {
                let item = entry?;
                if item.name()? == operation_name {
                    found = Some(item);
                    break;
                }
            }
            let operation = found.ok_or("catalog operation")?;
            let body_decodable = supports_object_body(service, operation)?;
            let operation_shape = shapes.get(operation_id).ok_or("operation shape")?;
            let cases = operation_shape
                .get("traits")
                .and_then(|traits| traits.get("smithy.test#httpResponseTests"))
                .and_then(Json::as_array)
                .into_iter()
                .flatten();
            for case in cases {
                if case["appliesTo"] == "server" {
                    server_count = server_count.saturating_add(1);
                    continue;
                }
                let id = case["id"].as_str().ok_or("fixture case id")?;
                let body = case["body"].as_str().unwrap_or("").as_bytes();
                let mut headers = HeaderMap::new();
                if let Some(values) = case["headers"].as_object() {
                    for (name, value) in values {
                        let header_name = HeaderName::from_bytes(name.as_bytes())?;
                        if let Some(text) = value.as_str() {
                            headers.append(header_name, HeaderValue::from_str(text)?);
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
                    client_count = client_count.saturating_add(1);
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
                    .map_err(|error| format!("{} {id}: {error}", suite.name))?;
                fixture_oracle::assert_fixture_root(
                    &record,
                    service,
                    operation.output()?,
                    &case["params"],
                    &format!("{} {id}", suite.name),
                );
                if !body_limit_tested && body.len() > 1 {
                    let limits = DecodeLimits::new(body.len().saturating_sub(1), 65_536, 32)?;
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
                    let limits = DecodeLimits::new(16_777_216, 1, 32)?;
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
                    let limits = DecodeLimits::new(16_777_216, 65_536, 1)?;
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
                client_count = client_count.saturating_add(1);
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
    Ok(())
}

fn supports_object_body(
    service: inventory_core::catalog::Service<'_>,
    operation: inventory_core::catalog::Operation<'_>,
) -> Result<bool, Box<dyn std::error::Error>> {
    let Target::Shape(index) = operation.output()? else {
        return Ok(false);
    };
    let shape = service.shape(index)?.ok_or("output shape exists")?;
    for member in shape.members()? {
        if member?.wire()?.binding()? == HttpBinding::Payload {
            return Ok(false);
        }
    }
    Ok(true)
}
