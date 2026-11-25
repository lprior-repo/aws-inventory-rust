use super::*;

fn decode_error(
    body: &[u8],
    headers: &http::HeaderMap,
) -> Result<ServiceError, Box<dyn std::error::Error>> {
    let bytes = compile_service_model(&modeled_error_fixture(), "example#ErrorService")?;
    let catalog = Catalog::borrow(&bytes)?;
    let service = catalog.service("ErrorService")?.ok_or("service")?;
    let operation = service.operations()?.next().ok_or("operation")??;
    Ok(decode(ResponseContext {
        service,
        operation,
        protocol: Protocol::RpcV2Cbor,
        body,
        headers,
        status: http::StatusCode::BAD_REQUEST,
        limits: DecodeLimits::default(),
    })?)
}

#[test]
fn given_json_rpc_error_when_decoded_then_code_is_preserved_without_message() -> TestResult {
    let headers = http::HeaderMap::from_iter([(
        http::header::CONTENT_TYPE,
        http::HeaderValue::from_static("application/x-amz-json-1.1; charset=utf-8"),
    )]);
    let error = decode_error(br#"{"__type":"SerializationException","message":"missing required header `x-amz-target`"}"#, &headers)?;
    assert_eq!(error.code.as_deref(), Some("SerializationException"));
    assert_eq!(error.class, ErrorClass::Fatal);
    assert!(!error.to_string().contains("x-amz-target"));
    Ok(())
}

#[test]
fn given_unsupported_header_when_body_is_other_protocol_then_gap_remains_visible() -> TestResult {
    let headers = http::HeaderMap::from_iter([(
        http::header::HeaderName::from_static("x-amzn-errortype"),
        http::HeaderValue::from_static("NotImplemented"),
    )]);
    let error = decode_error(br#"{"message":"unimplemented"}"#, &headers)?;
    assert_eq!(error.code.as_deref(), Some("NotImplemented"));
    assert_eq!(error.class, ErrorClass::NotSupported);
    Ok(())
}

#[test]
fn given_unknown_operation_when_classified_then_absence_is_not_a_decoder_failure() -> TestResult {
    let headers = http::HeaderMap::from_iter([(
        http::header::CONTENT_TYPE,
        http::HeaderValue::from_static("application/json"),
    )]);
    [
        "NotImplemented",
        "UnknownOperationException",
        "UnknownOperationError",
        "UnknownService",
        "InvalidAction",
    ]
    .into_iter()
    .try_for_each(|code| -> TestResult {
        let body = format!(r#"{{"__type":"{code}"}}"#);
        let error = decode_error(body.as_bytes(), &headers)?;
        assert_eq!(error.class, ErrorClass::NotSupported, "{code}");
        Ok(())
    })
}

#[test]
fn given_resource_or_validation_error_when_classified_then_failure_is_not_hidden() -> TestResult {
    let headers = http::HeaderMap::from_iter([(
        http::header::CONTENT_TYPE,
        http::HeaderValue::from_static("application/json"),
    )]);
    [
        "ResourceNotFoundException",
        "ValidationException",
        "AccessDeniedException",
    ]
    .into_iter()
    .try_for_each(|code| -> TestResult {
        let body = format!(r#"{{"__type":"{code}"}}"#);
        let error = decode_error(body.as_bytes(), &headers)?;
        assert_ne!(error.class, ErrorClass::NotSupported, "{code}");
        Ok(())
    })
}
