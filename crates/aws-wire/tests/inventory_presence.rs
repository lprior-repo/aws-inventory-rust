#[path = "../src/response/json/fixture_catalog.rs"]
mod fixture_catalog;
use aws_wire::{
    request::Protocol,
    response::{self, DecodeLimits, ResponseContext},
};
use inventory_core::{catalog::Catalog, record::ValueRef};
type TestResult = Result<(), Box<dyn std::error::Error>>;

const MODEL: &str = r#"{"smithy":"2.0","shapes":{
"n#InventoryPresence":{"type":"service","version":"1","operations":[{"target":"n#Tags"}],"traits":{"aws.api#service":{"sdkId":"InventoryPresence","endpointPrefix":"presence"},"aws.protocols#awsJson1_1":{}}},
"n#Tags":{"type":"operation","output":{"target":"n#Output"}},
"n#Output":{"type":"structure","members":{"Tags":{"target":"n#TagsList","traits":{"smithy.api#required":{}}},"TagMap":{"target":"n#TagMap","traits":{"smithy.api#required":{}}}}},
"n#TagsList":{"type":"list","member":{"target":"n#Tag"}},
"n#TagMap":{"type":"map","key":{"target":"smithy.api#String"},"value":{"target":"smithy.api#String"}},
"n#Tag":{"type":"structure","members":{"Key":{"target":"smithy.api#String","traits":{"smithy.api#required":{}}},"Value":{"target":"smithy.api#String","traits":{"smithy.api#required":{}}}}}
}}"#;

#[test]
fn given_json_and_cbor_null_tag_entries_when_inventory_presence_is_required_then_they_cannot_become_empty_snapshots()
-> TestResult {
    let model = serde_json::from_str(MODEL)?;
    let bytes = fixture_catalog::compile_service(&model, "n#InventoryPresence")?;
    let catalog = Catalog::borrow(&bytes)?;
    let service = catalog
        .service("InventoryPresence")?
        .ok_or("service missing")?;
    let operation = service.operations()?.next().ok_or("operation missing")??;
    let headers = http::HeaderMap::new();
    let cases: &[(Protocol, &[u8])] = &[
        (Protocol::AwsJson11, br#"{"Tags":[null]}"#),
        (Protocol::AwsJson11, br#"{"TagMap":{"known":null}}"#),
        (
            Protocol::RpcV2Cbor,
            &[0xa1, 0x64, b'T', b'a', b'g', b's', 0x81, 0xf6],
        ),
        (
            Protocol::RpcV2Cbor,
            &[
                0xa1, 0x66, b'T', b'a', b'g', b'M', b'a', b'p', 0xa1, 0x65, b'k', b'n', b'o', b'w',
                b'n', 0xf6,
            ],
        ),
    ];
    for (protocol, body) in cases {
        let context = ResponseContext {
            service,
            operation,
            protocol: *protocol,
            body,
            headers: &headers,
            status: http::StatusCode::OK,
            limits: DecodeLimits::default(),
        };
        // Given: SDK compatibility deliberately elides non-sparse null entries.
        assert!(response::decode(context).is_ok());
        // When/Then: inventory decoding rejects their loss instead of authorizing tag deletion.
        assert!(
            response::decode(ResponseContext {
                limits: DecodeLimits::default().preserve_field_presence(),
                ..context
            })
            .is_err()
        );
    }
    for (protocol, body) in [
        (Protocol::AwsJson11, b"{}".as_slice()),
        (Protocol::RpcV2Cbor, [0xa0].as_slice()),
    ] {
        let record = response::decode(ResponseContext {
            service,
            operation,
            protocol,
            body,
            headers: &headers,
            status: http::StatusCode::OK,
            limits: DecodeLimits::default().preserve_field_presence(),
        })?;
        assert_eq!(record.root().iter()?.count(), 0);
    }
    let record = response::decode(ResponseContext {
        service,
        operation,
        protocol: Protocol::AwsJson11,
        body: br#"{"Tags":[],"TagMap":{}}"#,
        headers: &headers,
        status: http::StatusCode::OK,
        limits: DecodeLimits::default().preserve_field_presence(),
    })?;
    assert_eq!(record.root().iter()?.count(), 2);
    for entry in record.root().iter()? {
        assert!(matches!(entry?.1, ValueRef::List(_) | ValueRef::Map(_)));
    }
    Ok(())
}
