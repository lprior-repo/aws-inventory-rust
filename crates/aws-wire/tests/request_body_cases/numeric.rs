#[path = "exponents.rs"]
mod exponents;

use super::request_body_fixtures_support::compile_fixture_service;
use aws_wire::request::{
    CompressionPolicy, EncodeError, Protocol, RequestContext, RequestLimits, body,
};
use inventory_core::{
    catalog::{Catalog, Operation, Service},
    record::{Key, Record, Time, Value},
};
use serde_json::Value as Json;
use std::collections::BTreeMap;

#[test]
fn rpc_timestamp_integer_seconds_are_exact() {
    let fixture = rpc_fixture();
    let bytes = compile_fixture_service(&fixture, "smithy.protocoltests.rpcv2Cbor#RpcV2Protocol");
    let catalog = Catalog::borrow(&bytes).expect("fixture catalog validates");
    let service = catalog
        .service("RpcV2Protocol")
        .expect("service lookup")
        .expect("service exists");
    assert_exact_timestamp_seconds(service, scalar_operation(service));
}

#[test]
fn rpc_big_numbers_use_protocol_numeric_forms() {
    let fixture = rpc_fixture();
    let bytes = compile_fixture_service(&fixture, "smithy.protocoltests.rpcv2Cbor#RpcV2Protocol");
    let catalog = Catalog::borrow(&bytes).expect("fixture catalog validates");
    let service = catalog
        .service("RpcV2Protocol")
        .expect("service lookup")
        .expect("service exists");
    let operation = scalar_operation(service);
    assert_rpc_big_numbers(service, operation);
    assert_rejects_malformed_big_decimal(service, operation);
}

#[test]
fn aws_json_big_numbers_remain_json_numbers() {
    let mut fixture = rpc_fixture();
    let service_id = "smithy.protocoltests.rpcv2Cbor#RpcV2Protocol";
    let traits = fixture["shapes"][service_id]["traits"]
        .as_object_mut()
        .expect("service traits");
    traits.remove("smithy.protocols#rpcv2Cbor");
    traits.insert(
        "aws.protocols#awsJson1_1".to_owned(),
        Json::Object(Default::default()),
    );
    let bytes = compile_fixture_service(&fixture, service_id);
    let catalog = Catalog::borrow(&bytes).expect("AWS JSON fixture catalog validates");
    let service = catalog
        .service("RpcV2Protocol")
        .expect("service lookup")
        .expect("service exists");
    assert_json_big_numbers(service, scalar_operation(service));
}
#[test]
fn rpc_big_decimal_accepts_i64_min_exponent() {
    let fixture = rpc_fixture();
    let bytes = compile_fixture_service(&fixture, "smithy.protocoltests.rpcv2Cbor#RpcV2Protocol");
    let catalog = Catalog::borrow(&bytes).expect("fixture catalog validates");
    let service = catalog
        .service("RpcV2Protocol")
        .expect("service lookup")
        .expect("service exists");
    let numeric = [
        0xc4, 0x82, 0x3b, 0x7f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01,
    ];
    assert_eq!(
        fixture_body(
            service,
            scalar_operation(service),
            Protocol::RpcV2Cbor,
            "bigDecimalMember",
            Value::Text("1e-9223372036854775808".to_owned()),
        ),
        cbor_structure_body("bigDecimalMember", &numeric),
    );
}

#[test]
fn rpc_big_decimal_honors_small_body_budget_before_conversion() {
    let fixture = rpc_fixture();
    let bytes = compile_fixture_service(&fixture, "smithy.protocoltests.rpcv2Cbor#RpcV2Protocol");
    let catalog = Catalog::borrow(&bytes).expect("fixture catalog validates");
    let service = catalog
        .service("RpcV2Protocol")
        .expect("service lookup")
        .expect("service exists");
    let operation = scalar_operation(service);
    let input = Record::encode(&Value::Map(BTreeMap::from([(
        Key::Name("bigDecimalMember".to_owned()),
        Value::Text("9".repeat(10_000)),
    )])))
    .expect("large decimal record");
    let context = RequestContext {
        service,
        operation,
        protocol: Protocol::RpcV2Cbor,
        input: input.root(),
        endpoint: "https://example.com",
        limits: RequestLimits::new(64, 65_536, 16_384).expect("valid request limits"),
        idempotency_token: None,
        compression: CompressionPolicy::default(),
        addressing: None,
    };
    assert!(matches!(
        body::encode(context),
        Err(EncodeError::Limit("body"))
    ));
}

fn rpc_fixture() -> Json {
    serde_json::from_str(include_str!("../fixtures/protocol-tests/rpcv2Cbor.json"))
        .expect("published RPCv2 fixture is JSON")
}

fn scalar_operation(service: Service<'_>) -> Operation<'_> {
    service
        .operations()
        .expect("operations")
        .map(|item| item.expect("valid operation"))
        .find(|item| item.name().expect("operation name") == "ScalarMembers")
        .expect("scalar operation exists")
}

fn fixture_body<'a>(
    service: Service<'a>,
    operation: Operation<'a>,
    protocol: Protocol,
    name: &str,
    value: Value,
) -> Vec<u8> {
    let input = Record::encode(&Value::Map(BTreeMap::from([(
        Key::Name(name.to_owned()),
        value,
    )])))
    .expect("request record");
    body::encode(RequestContext {
        service,
        operation,
        protocol,
        input: input.root(),
        endpoint: "https://example.com",
        limits: RequestLimits::default(),
        idempotency_token: None,
        compression: CompressionPolicy::default(),
        addressing: None,
    })
    .expect("request encodes")
    .body()
    .to_vec()
}

fn assert_exact_timestamp_seconds(service: Service<'_>, operation: Operation<'_>) {
    for (seconds, head, integer) in [
        (
            9_007_199_254_740_993,
            0x1b,
            [0x00, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01],
        ),
        (
            -9_007_199_254_740_993,
            0x3b,
            [0x00, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        ),
    ] {
        let expected = timestamp_body(head, integer);
        let value = Value::Time(Time::new(seconds, 0).expect("timestamp range"));
        assert_eq!(
            fixture_body(
                service,
                operation,
                Protocol::RpcV2Cbor,
                "epochSecondsMember",
                value
            ),
            expected
        );
    }
}

fn timestamp_body(head: u8, integer: [u8; 8]) -> Vec<u8> {
    let mut expected = vec![0xbf, 0x72];
    expected.extend_from_slice(b"epochSecondsMember");
    expected.extend_from_slice(&[0xc1, head]);
    expected.extend_from_slice(&integer);
    expected.push(0xff);
    expected
}

fn assert_rpc_big_numbers(service: Service<'_>, operation: Operation<'_>) {
    for (name, value, numeric) in [
        ("bigIntegerMember", Value::Int(1), vec![0xc2, 0x41, 0x01]),
        ("bigIntegerMember", Value::Int(-2), vec![0xc3, 0x41, 0x01]),
        (
            "bigDecimalMember",
            Value::Text("1.5".to_owned()),
            vec![0xc4, 0x82, 0x20, 0x0f],
        ),
    ] {
        let expected = cbor_structure_body(name, &numeric);
        assert_eq!(
            fixture_body(service, operation, Protocol::RpcV2Cbor, name, value),
            expected
        );
    }
}

fn cbor_structure_body(name: &str, value: &[u8]) -> Vec<u8> {
    let mut expected = vec![0xbf, 0x70];
    expected.extend_from_slice(name.as_bytes());
    expected.extend_from_slice(value);
    expected.push(0xff);
    expected
}

fn assert_rejects_malformed_big_decimal(service: Service<'_>, operation: Operation<'_>) {
    let input = Record::encode(&Value::Map(BTreeMap::from([(
        Key::Name("bigDecimalMember".to_owned()),
        Value::Text("01".to_owned()),
    )])))
    .expect("malformed decimal record");
    let context = RequestContext {
        service,
        operation,
        protocol: Protocol::RpcV2Cbor,
        input: input.root(),
        endpoint: "https://example.com",
        limits: RequestLimits::default(),
        idempotency_token: None,
        compression: CompressionPolicy::default(),
        addressing: None,
    };
    assert!(matches!(
        body::encode(context),
        Err(EncodeError::InvalidValue { .. })
    ));
}

fn assert_json_big_numbers(service: Service<'_>, operation: Operation<'_>) {
    for (name, value, expected) in [
        (
            "bigIntegerMember",
            Value::Int(1),
            b"{\"bigIntegerMember\":1}".as_slice(),
        ),
        (
            "bigDecimalMember",
            Value::Text("1.5".to_owned()),
            b"{\"bigDecimalMember\":1.5}".as_slice(),
        ),
    ] {
        assert_eq!(
            fixture_body(service, operation, Protocol::AwsJson11, name, value),
            expected
        );
    }
}
