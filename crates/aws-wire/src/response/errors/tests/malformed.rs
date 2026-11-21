use super::*;

#[test]
fn cbor_breaks_cannot_terminate_definite_containers_or_replace_values() -> TestResult {
    for body in [
        b"\xa1\xff".as_slice(),
        b"\xa1\x61x\xff".as_slice(),
        b"\xa1\x61x\x81\xff".as_slice(),
        b"\xbf\x61x\xff".as_slice(),
    ] {
        assert!(
            matches!(
                decode_body(Protocol::RpcV2Cbor, body, DecodeLimits::default())?,
                Err(DecodeError::Invalid(_))
            ),
            "{body:?}"
        );
    }
    let empty = decode_body(Protocol::RpcV2Cbor, b"\xbf\xff", DecodeLimits::default())??;
    assert_eq!(empty.code, None);
    assert_eq!(empty.class, ErrorClass::Fatal);
    Ok(())
}

#[test]
fn xml_rejects_malformed_unknown_content_and_text_outside_the_root() -> TestResult {
    for body in [
        b"<Error/>junk".as_slice(),
        b"junk<Error/>".as_slice(),
        b"<Error><Code><x/>ExpiredToken</Code></Error>".as_slice(),
        b"<Error unknown='&bogus;'/>".as_slice(),
        b"<Error broken/>".as_slice(),
        b"<Error same='a' same='b'/>".as_slice(),
        b"<Error><Message>&bogus;</Message></Error>".as_slice(),
        b"<!DOCTYPE Error><Error/>".as_slice(),
    ] {
        assert!(
            matches!(
                decode_body(Protocol::RestXml, body, DecodeLimits::default())?,
                Err(DecodeError::Invalid(_))
            ),
            "{body:?}"
        );
    }
    Ok(())
}

#[test]
fn xml_accepts_standard_references_but_not_nested_code_spoofing() -> TestResult {
    let body = b"<Error><Message>A &amp; B &#65;</Message><Code>ExpiredToken</Code></Error>";
    let error = decode_body(Protocol::RestXml, body, DecodeLimits::default())??;
    assert_eq!(error.code.as_deref(), Some("ExpiredToken"));
    assert_eq!(error.class, ErrorClass::ExpiredToken);
    let body = b"<Error><Ignored><Code>ExpiredToken</Code></Ignored></Error>";
    let error = decode_body(Protocol::RestXml, body, DecodeLimits::default())??;
    assert_eq!(error.code, None);
    assert_eq!(error.class, ErrorClass::Fatal);
    Ok(())
}

#[test]
fn xml_counts_unknown_attributes_and_empty_element_depth() -> TestResult {
    let limits = DecodeLimits::new(1024, 1, 4)?;
    assert!(matches!(
        decode_body(Protocol::RestXml, b"<Error unknown='x'/>", limits)?,
        Err(DecodeError::Limit("response nodes"))
    ));
    let limits = DecodeLimits::new(1024, 100, 1)?;
    assert!(matches!(
        decode_body(Protocol::RestXml, b"<Error><Unknown/></Error>", limits)?,
        Err(DecodeError::Limit("response depth"))
    ));
    Ok(())
}

/// Outer error: fixture setup failed. Inner result: the decoder's verdict.
fn decode_body(
    protocol: Protocol,
    body: &[u8],
    limits: DecodeLimits,
) -> Result<Result<ServiceError, DecodeError>, Box<dyn std::error::Error>> {
    let bytes = protocol_catalog(protocol)?;
    let catalog = Catalog::borrow(&bytes)?;
    let service = catalog.service("ErrorService")?.ok_or("service")?;
    let operation = service
        .operations()?
        .next()
        .ok_or("operation reference")??;
    Ok(super::decode(ResponseContext {
        service,
        operation,
        protocol,
        body,
        limits,
        headers: &http::HeaderMap::new(),
        status: http::StatusCode::BAD_REQUEST,
    }))
}

fn protocol_catalog(protocol: Protocol) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut fixture = modeled_error_fixture();
    let name = match protocol {
        Protocol::AwsJson10 => "aws.protocols#awsJson1_0",
        Protocol::AwsJson11 => "aws.protocols#awsJson1_1",
        Protocol::AwsQuery => "aws.protocols#awsQuery",
        Protocol::Ec2Query => "aws.protocols#ec2Query",
        Protocol::RestJson1 => "aws.protocols#restJson1",
        Protocol::RestXml => "aws.protocols#restXml",
        Protocol::RpcV2Cbor => "smithy.protocols#rpcv2Cbor",
    };
    let traits = fixture
        .pointer_mut("/shapes/example#ErrorService/traits")
        .ok_or("fixture traits")?;
    *traits = json!({name: {}});
    compile_service_model(&fixture, "example#ErrorService")
}
