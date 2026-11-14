use super::{
    Catalog, CompressionPolicy, EncodeError, Key, Protocol, Record, RequestContext, RequestLimits,
    Value, body, cbor_structure_body, compile_fixture_service, fixture_body, rpc_fixture,
    scalar_operation,
};
use std::collections::BTreeMap;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn rpc_decimal_retains_exact_exponents_across_the_full_cbor_integer_range() -> TestResult {
    let fixture = rpc_fixture();
    let bytes = compile_fixture_service(&fixture, "smithy.protocoltests.rpcv2Cbor#RpcV2Protocol");
    let catalog = Catalog::borrow(&bytes)?;
    let service = catalog.service("RpcV2Protocol")?.ok_or("service missing")?;
    for (text, head, argument, coefficient) in [
        ("1e9223372036854775808", 0x1b, 0x8000000000000000_u64, 1),
        ("1e18446744073709551615", 0x1b, u64::MAX, 1),
        ("1e-9223372036854775809", 0x3b, 0x8000000000000000, 1),
        ("1e-18446744073709551616", 0x3b, u64::MAX, 1),
        ("0.1e18446744073709551616", 0x1b, u64::MAX, 1),
        ("1.0e-18446744073709551615", 0x3b, u64::MAX, 10),
    ] {
        let mut numeric = vec![0xc4, 0x82, head];
        numeric.extend_from_slice(&argument.to_be_bytes());
        numeric.push(coefficient);
        let actual = fixture_body(
            service,
            scalar_operation(service),
            Protocol::RpcV2Cbor,
            "bigDecimalMember",
            Value::Text(text.to_owned()),
        );
        assert_eq!(
            actual,
            cbor_structure_body("bigDecimalMember", &numeric),
            "{text}"
        );
    }
    Ok(())
}

#[test]
fn rpc_decimal_rejects_normalized_exponents_outside_the_cbor_integer_range() -> TestResult {
    let fixture = rpc_fixture();
    let bytes = compile_fixture_service(&fixture, "smithy.protocoltests.rpcv2Cbor#RpcV2Protocol");
    let catalog = Catalog::borrow(&bytes)?;
    let service = catalog.service("RpcV2Protocol")?.ok_or("service missing")?;
    for text in [
        "1e18446744073709551616",
        "1e-18446744073709551617",
        "0.1e-18446744073709551616",
        "1e170141183460469231731687303715884105728",
    ] {
        let input = Record::encode(&Value::Map(BTreeMap::from([(
            Key::Name("bigDecimalMember".to_owned()),
            Value::Text(text.to_owned()),
        )])))?;
        let observed = body::encode(RequestContext {
            service,
            operation: scalar_operation(service),
            protocol: Protocol::RpcV2Cbor,
            input: input.root(),
            endpoint: "https://example.com",
            limits: RequestLimits::default(),
            idempotency_token: None,
            compression: CompressionPolicy::default(),
            addressing: None,
        });
        assert!(
            matches!(observed, Err(EncodeError::InvalidValue { .. })),
            "{text}"
        );
    }
    Ok(())
}
