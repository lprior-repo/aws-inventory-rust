#[path = "../src/response/json/fixture_catalog.rs"]
mod fixture_catalog;
use aws_wire::{
    request::Protocol,
    response::{self, DecodeLimits, ResponseContext},
};
use inventory_core::{
    catalog::Catalog,
    record::{FieldId, MemberExclusions, ValueRef},
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const MODEL: &str = r#"{"smithy":"2.0","shapes":{
"n#Omission":{"type":"service","version":"1","operations":[{"target":"n#Do"}],"traits":{"aws.api#service":{"sdkId":"Omission","endpointPrefix":"omission"},"aws.protocols#awsJson1_1":{}}},
"n#Do":{"type":"operation","output":{"target":"n#Output"}},
"n#Output":{"type":"structure","members":{"Items":{"target":"n#Items"},"SecretBlob":{"target":"smithy.api#Blob"},"Secret":{"target":"smithy.api#String","traits":{"smithy.api#default":"synthesized-secret"}}}},
"n#Items":{"type":"list","member":{"target":"n#Item"}},
"n#Item":{"type":"structure","members":{"Id":{"target":"smithy.api#String"},"Secret":{"target":"smithy.api#String"}}}
}}"#;
const JSON: &[u8] = br#"{"Items":[{"Id":"metadata","Secret":"NEVER_RETAIN"}],"Secret":"NEVER_RETAIN","SecretBlob":"INVALID_BASE64_NEVER_RETAIN"}"#;
const XML: &[u8] = b"<DoResponse><DoResult><Items><member><Id>metadata</Id><Secret>NEVER_RETAIN</Secret></member></Items><Secret>NEVER_RETAIN</Secret><SecretBlob>INVALID_BASE64_NEVER_RETAIN</SecretBlob></DoResult></DoResponse>";

fn catalog() -> TestResult<Vec<u8>> {
    fixture_catalog::compile_service(&serde_json::from_str(MODEL)?, "n#Omission")
}

fn policy() -> TestResult<MemberExclusions> {
    Ok(MemberExclusions::new([FieldId::new(3)?, FieldId::new(4)?])?)
}

fn decode(
    bytes: &[u8],
    protocol: Protocol,
    body: &[u8],
    excluded: bool,
) -> TestResult<inventory_core::record::Record> {
    let catalog = Catalog::borrow(bytes)?;
    let service = catalog.service("Omission")?.ok_or("missing service")?;
    let operation = service.operations()?.next().ok_or("missing operation")??;
    let limits = if excluded {
        DecodeLimits::default().excluding_members(policy()?)
    } else {
        DecodeLimits::default()
    };
    Ok(response::decode(ResponseContext {
        service,
        operation,
        protocol,
        body,
        headers: &http::HeaderMap::new(),
        status: http::StatusCode::OK,
        limits,
    })?)
}

fn assert_metadata(record: &inventory_core::record::Record) -> TestResult {
    assert_eq!(record.root().iter()?.count(), 1);
    let ValueRef::List(items) = record
        .root()
        .get_field(FieldId::new(2)?)?
        .ok_or("items missing")?
    else {
        return Err("items not list".into());
    };
    let ValueRef::Map(item) = items.iter()?.next().ok_or("item missing")?? else {
        return Err("item not map".into());
    };
    assert_eq!(item.iter()?.count(), 1);
    assert!(matches!(
        item.get_field(FieldId::new(1)?)?,
        Some(ValueRef::Text("metadata"))
    ));
    assert_persistence(record)
}

fn assert_persistence(record: &inventory_core::record::Record) -> TestResult {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("record.cbor");
    std::fs::write(&path, record.as_bytes())?;
    let persisted = std::fs::read(path)?;
    assert!(
        !persisted
            .windows(b"NEVER_RETAIN".len())
            .any(|value| value == b"NEVER_RETAIN")
    );
    assert!(!format!("{record:?}").contains("NEVER_RETAIN"));
    Ok(())
}

#[test]
fn given_secret_members_when_json_is_decoded_then_only_metadata_reaches_persistence() -> TestResult
{
    let bytes = catalog()?;
    for protocol in [
        Protocol::AwsJson10,
        Protocol::AwsJson11,
        Protocol::RestJson1,
    ] {
        assert_metadata(&decode(&bytes, protocol, JSON, true)?)?;
        let error = decode(&bytes, protocol, JSON, false)
            .err()
            .ok_or("blob unexpectedly decoded")?;
        assert!(!error.to_string().contains("NEVER_RETAIN"));
    }
    Ok(())
}

#[test]
fn given_secret_members_when_query_xml_is_decoded_then_invalid_blob_is_not_decoded() -> TestResult {
    let bytes = catalog()?;
    for (protocol, body) in [
        (Protocol::AwsQuery, XML),
        (Protocol::Ec2Query, b"<DoResponse><Items><member><Id>metadata</Id><Secret>NEVER_RETAIN</Secret></member></Items><Secret>NEVER_RETAIN</Secret><SecretBlob>INVALID_BASE64_NEVER_RETAIN</SecretBlob></DoResponse>".as_slice()),
    ] {
        assert_metadata(&decode(&bytes, protocol, body, true)?)?;
    }
    Ok(())
}

#[test]
fn given_secret_members_when_rest_xml_is_decoded_then_metadata_and_no_secret_are_retained()
-> TestResult {
    let body = b"<Output><Items><member><Id>metadata</Id><Secret>NEVER_RETAIN</Secret></member></Items><Secret>NEVER_RETAIN</Secret><SecretBlob>INVALID_BASE64_NEVER_RETAIN</SecretBlob></Output>";
    assert_metadata(&decode(&catalog()?, Protocol::RestXml, body, true)?)
}

#[test]
fn given_secret_members_when_cbor_is_decoded_then_metadata_and_no_secret_are_retained() -> TestResult
{
    let mut encoder = minicbor::Encoder::new(Vec::new());
    encoder
        .map(3)?
        .str("Items")?
        .array(1)?
        .map(2)?
        .str("Id")?
        .str("metadata")?
        .str("Secret")?
        .str("NEVER_RETAIN")?
        .str("Secret")?
        .str("NEVER_RETAIN")?
        .str("SecretBlob")?
        .str("INVALID_BASE64_NEVER_RETAIN")?;
    assert_metadata(&decode(
        &catalog()?,
        Protocol::RpcV2Cbor,
        &encoder.into_writer(),
        true,
    )?)
}

#[test]
fn given_absent_excluded_member_when_sdk_defaults_apply_then_no_secret_is_synthesized() -> TestResult
{
    let record = decode(
        &catalog()?,
        Protocol::AwsJson11,
        br#"{"Items":[{"Id":"metadata"}]}"#,
        true,
    )?;
    assert_metadata(&record)
}
