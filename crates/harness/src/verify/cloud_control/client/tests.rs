use super::*;

#[test]
fn given_sdk_error_when_classified_then_original_source_and_context_are_retained()
-> Result<(), Box<dyn std::error::Error>> {
    let sdk = aws_sdk_cloudcontrol::error::SdkError::construction_failure(std::io::Error::other(
        "bad request",
    ));
    let error = list_error("AWS::SNS::Topic", 0, sdk);
    assert!(std::error::Error::source(&error).is_some());
    let CloudControlError::Service {
        type_name,
        code,
        source,
    } = error
    else {
        return Err("expected service error".into());
    };
    assert_eq!(type_name, "AWS::SNS::Topic");
    assert_eq!(code, None);
    assert!(
        source
            .downcast_ref::<aws_sdk_cloudcontrol::error::SdkError<
                aws_sdk_cloudcontrol::operation::list_resources::ListResourcesError,
            >>()
            .is_some()
    );
    Ok(())
}

#[test]
fn given_nested_response_bound_when_classified_then_typed_error_retains_type_page_and_limit()
-> Result<(), Box<dyn std::error::Error>> {
    let connector = aws_smithy_runtime_api::client::result::ConnectorError::other(
        Box::new(transport::ResponseBound {
            limit: transport::MAX_RESPONSE_BYTES,
        }),
        None,
    );
    let sdk = aws_sdk_cloudcontrol::error::SdkError::dispatch_failure(connector);
    let error = list_error("AWS::SNS::Topic", 1, sdk);
    assert_eq!(
        error.to_string(),
        "Cloud Control AWS::SNS::Topic page 2 response exceeds 1048576 bytes"
    );
    let CloudControlError::ResponseBytes {
        type_name,
        page,
        limit,
    } = error
    else {
        return Err("expected response byte bound".into());
    };
    assert_eq!(
        (type_name.as_str(), page, limit),
        ("AWS::SNS::Topic", 2, 1_048_576)
    );
    Ok(())
}

#[test]
fn given_multibyte_identifier_when_bounded_then_bytes_not_characters_determine_acceptance()
-> Result<(), Box<dyn std::error::Error>> {
    let at_bound = "é".repeat(2048);
    assert_eq!(identifier(Some(&at_bound), "AWS::SNS::Topic", 2)?, at_bound);
    let oversized = format!("{at_bound}x");
    let error = identifier(Some(&oversized), "AWS::SNS::Topic", 2)
        .err()
        .ok_or("expected identifier byte bound")?;
    assert_eq!(
        error.to_string(),
        "Cloud Control AWS::SNS::Topic page 2 identifier exceeds 4096 bytes"
    );
    let CloudControlError::IdentifierBytes {
        type_name,
        page,
        limit,
    } = error
    else {
        return Err("expected identifier byte bound".into());
    };
    assert_eq!(
        (type_name.as_str(), page, limit),
        ("AWS::SNS::Topic", 2, 4096)
    );
    Ok(())
}

#[test]
fn given_modeled_throttling_error_when_classified_then_typed_service_code_is_retained()
-> Result<(), Box<dyn std::error::Error>> {
    use aws_sdk_cloudcontrol::operation::list_resources::ListResourcesError;
    use aws_smithy_runtime_api::http::Response;
    use aws_smithy_types::{body::SdkBody, error::ErrorMetadata};

    let throttled = aws_sdk_cloudcontrol::types::error::ThrottlingException::builder()
        .message("slow down")
        .meta(ErrorMetadata::builder().code("ThrottlingException").build())
        .build();
    let sdk = aws_sdk_cloudcontrol::error::SdkError::service_error(
        ListResourcesError::ThrottlingException(throttled),
        Response::new(400_u16.try_into()?, SdkBody::empty()),
    );
    let error = list_error("AWS::SNS::Topic", 1, sdk);
    assert_eq!(
        error.to_string(),
        "Cloud Control ListResources AWS::SNS::Topic: service error"
    );
    let CloudControlError::Service {
        type_name,
        code,
        source,
    } = error
    else {
        return Err("expected typed service error".into());
    };
    assert_eq!(type_name, "AWS::SNS::Topic");
    assert_eq!(code.as_deref(), Some("ThrottlingException"));
    assert_eq!(source.to_string(), "service error");
    Ok(())
}
