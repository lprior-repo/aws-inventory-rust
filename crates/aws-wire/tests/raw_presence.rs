#[path = "../src/response/json/fixture_catalog.rs"]
mod fixture_catalog;
use aws_wire::{
    request::Protocol,
    response::{self, DecodeError, DecodeLimits, ResponseContext},
};
use inventory_core::{
    catalog::Catalog,
    record::{Key, Value},
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn catalog() -> TestResult<Vec<u8>> {
    let model = serde_json::json!({"smithy":"2.0","shapes":{
        "n#RawPresence":{"type":"service","version":"1","operations":[{"target":"n#Read"}],"traits":{"aws.api#service":{"sdkId":"RawPresence","endpointPrefix":"raw"},"aws.protocols#awsJson1_1":{}}},
        "n#Read":{"type":"operation","output":{"target":"n#Output"}},
        "n#Output":{"type":"structure","members":{"Member":{"target":"smithy.api#String"},"Values":{"target":"n#Values"},"Map":{"target":"n#Map"}}},
        "n#Values":{"type":"list","member":{"target":"smithy.api#String"}},
        "n#Map":{"type":"map","key":{"target":"smithy.api#String"},"value":{"target":"smithy.api#String"}}
    }});
    fixture_catalog::compile_service(&model, "n#RawPresence")
}

#[test]
fn given_json_nulls_when_raw_mode_is_opted_in_then_inventory_mode_still_refuses_them() -> TestResult
{
    for (body, expected) in [
        (br#"{"Member":null}"#.as_slice(), Value::Null),
        (
            br#"{"Values":[null]}"#.as_slice(),
            Value::List(vec![Value::Null]),
        ),
        (
            br#"{"Map":{"k":null}}"#.as_slice(),
            Value::Map([(Key::Name("k".into()), Value::Null)].into()),
        ),
    ] {
        assert_raw_null(Protocol::AwsJson11, body, expected)?;
    }
    Ok(())
}

#[test]
fn given_cbor_nulls_when_raw_mode_is_opted_in_then_inventory_mode_still_refuses_them() -> TestResult
{
    for (body, expected) in [
        (b"\xa1\x66Member\xf6".as_slice(), Value::Null),
        (
            b"\xa1\x66Values\x81\xf6".as_slice(),
            Value::List(vec![Value::Null]),
        ),
        (
            b"\xa1\x63Map\xa1\x61k\xf6".as_slice(),
            Value::Map([(Key::Name("k".into()), Value::Null)].into()),
        ),
    ] {
        assert_raw_null(Protocol::RpcV2Cbor, body, expected)?;
    }
    Ok(())
}

fn assert_raw_null(protocol: Protocol, body: &[u8], expected: Value) -> TestResult {
    let bytes = catalog()?;
    let catalog = Catalog::borrow(&bytes)?;
    let service = catalog.service("RawPresence")?.ok_or("service missing")?;
    let operation = service.operations()?.next().ok_or("operation missing")??;
    let headers = http::HeaderMap::new();
    let context = ResponseContext {
        service,
        operation,
        protocol,
        body,
        headers: &headers,
        status: http::StatusCode::OK,
        limits: DecodeLimits::default().preserve_field_presence(),
    };
    assert!(matches!(
        response::decode(context),
        Err(DecodeError::Invalid(_))
    ));
    let record = response::decode(ResponseContext {
        limits: DecodeLimits::default().preserve_raw_values(),
        ..context
    })?;
    assert_eq!(record.root().iter()?.count(), 1);
    let observed = record
        .root()
        .iter()?
        .next()
        .ok_or("raw member missing")??
        .1
        .to_value()?;
    assert_eq!(observed, expected);
    Ok(())
}

fn states(protocol: Protocol, bodies: &[&[u8]]) -> TestResult<Vec<Option<Value>>> {
    let bytes = catalog()?;
    let catalog = Catalog::borrow(&bytes)?;
    let service = catalog.service("RawPresence")?.ok_or("service missing")?;
    let operation = service.operations()?.next().ok_or("operation missing")??;
    let headers = http::HeaderMap::new();
    bodies
        .iter()
        .map(|body| {
            let record = response::decode(ResponseContext {
                service,
                operation,
                protocol,
                body,
                headers: &headers,
                status: http::StatusCode::OK,
                limits: DecodeLimits::default().preserve_raw_values(),
            })?;
            record
                .root()
                .iter()?
                .next()
                .map(|entry| Ok(entry?.1.to_value()?))
                .transpose()
        })
        .collect()
}

#[test]
fn given_json_and_cbor_absent_null_and_value_when_raw_decoded_then_all_three_states_are_distinct()
-> TestResult {
    let expected = vec![None, Some(Value::Null), Some(Value::Text("filled".into()))];
    assert_eq!(
        states(
            Protocol::AwsJson11,
            &[b"{}", br#"{"Member":null}"#, br#"{"Member":"filled"}"#]
        )?,
        expected
    );
    assert_eq!(
        states(
            Protocol::RpcV2Cbor,
            &[b"\xa0", b"\xa1\x66Member\xf6", b"\xa1\x66Member\x66filled"]
        )?,
        expected
    );
    Ok(())
}

#[test]
fn given_unmodeled_null_when_default_and_raw_decoded_then_both_discard_it() -> TestResult {
    let bytes = catalog()?;
    let catalog = Catalog::borrow(&bytes)?;
    let service = catalog.service("RawPresence")?.ok_or("service missing")?;
    let operation = service.operations()?.next().ok_or("operation missing")??;
    let headers = http::HeaderMap::new();
    for (protocol, body) in [
        (Protocol::AwsJson11, br#"{"Unknown":null}"#.as_slice()),
        (Protocol::RpcV2Cbor, b"\xa1\x67Unknown\xf6".as_slice()),
    ] {
        for limits in [
            DecodeLimits::default(),
            DecodeLimits::default().preserve_raw_values(),
        ] {
            let record = response::decode(ResponseContext {
                service,
                operation,
                protocol,
                body,
                headers: &headers,
                status: http::StatusCode::OK,
                limits,
            })?;
            assert_eq!(record.root().iter()?.count(), 0, "{protocol:?}");
        }
    }
    Ok(())
}
