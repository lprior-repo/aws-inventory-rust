use super::*;
use crate::{consumer_fixture as fixture, execution::page::cursor::Cursor};
use inventory_core::catalog::runtime::RuntimeCatalog;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[test]
fn denied_http_status_is_not_retried() {
    let error = aws_wire::response::DecodeError::Invalid("service error CBOR is not a map");
    assert!(matches!(
        status_fallback(http::StatusCode::FORBIDDEN, &error),
        ApiError::Denied(message)
            if message == "HTTP 403 Forbidden: invalid response: service error CBOR is not a map"
    ));
    assert!(matches!(
        status_fallback(http::StatusCode::TOO_MANY_REQUESTS, &error),
        ApiError::Throttled(message)
            if message == "HTTP 429 Too Many Requests: invalid response: service error CBOR is not a map"
    ));
}

#[test]
fn given_code_free_service_error_when_classified_then_status_is_reported() {
    let error = aws_wire::response::errors::ServiceError {
        class: aws_wire::response::errors::ErrorClass::Fatal,
        code: None,
        retry_after: None,
    };
    assert!(matches!(
        classify_service(error, http::StatusCode::NOT_FOUND),
        ApiError::Permanent(message) if message == "HTTP 404 Not Found"
    ));
}

const FIREHOSE_OMITTED: &str = r#"{"DeliveryStreamNames":["alpha"]}"#;
const FIREHOSE_FALSE: &str = r#"{"DeliveryStreamNames":["alpha"],"HasMoreDeliveryStreams":false}"#;
const FIREHOSE_TRUE: &str = r#"{"DeliveryStreamNames":["alpha"],"HasMoreDeliveryStreams":true}"#;

const IAM_ROLES_OPEN: &str = "<ListRolesResponse xmlns=\"https://iam.amazonaws.com/doc/2010-05-08/\"><ListRolesResult><Roles><member><RoleName>alpha</RoleName></member></Roles>";
const IAM_ROLES_CLOSE: &str = "</ListRolesResult><ResponseMetadata><RequestId>r</RequestId></ResponseMetadata></ListRolesResponse>";

fn continues(
    compiled: &fixture::Compiled,
    service: &str,
    operation: &str,
    body: &str,
) -> TestResult<bool> {
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let plan = catalog
        .lookup(service, operation)
        .ok_or("enumeration plan missing")?;
    let response = Response {
        status: http::StatusCode::OK,
        headers: http::HeaderMap::new(),
        body: bytes::Bytes::copy_from_slice(body.as_bytes()),
        version: http::Version::HTTP_11,
    };
    let page = decode_response(plan, response, false)?;
    let next = Cursor::default().next(plan.pagination(), &page.record, &page.headers, 10)?;
    Ok(next.is_some())
}

#[test]
fn numeric_datetime_fixture_is_accepted_only_for_explicit_endpoint_override() -> TestResult {
    let compiled = fixture::compile(
        "resource-explorer-2",
        "2022-07-28",
        &["GetIndex"],
    )?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let plan = catalog
        .lookup("resource-explorer-2", "GetIndex")
        .ok_or("GetIndex fixture plan missing")?;
    let response = || Response {
        status: http::StatusCode::OK,
        headers: http::HeaderMap::new(),
        body: bytes::Bytes::from_static(br#"{"CreatedAt":1795446406.337384223}"#),
        version: http::Version::HTTP_11,
    };
    assert!(decode_response(plan, response(), false).is_err());
    assert!(decode_response(plan, response(), true).is_ok());
    Ok(())
}

#[test]
fn given_json_page_omitting_required_truncation_flag_when_decoding_then_page_errors() -> TestResult
{
    let compiled = fixture::compile("firehose", "2015-08-04", &["ListDeliveryStreams"])?;
    let error = continues(
        &compiled,
        "firehose",
        "ListDeliveryStreams",
        FIREHOSE_OMITTED,
    )
    .err()
    .ok_or("omitted completion flag must not complete the enumeration")?;
    assert!(error.to_string().contains("truncated flag"), "{error}");
    Ok(())
}

#[test]
fn given_json_page_with_explicit_false_flag_when_decoding_then_enumeration_completes() -> TestResult
{
    let compiled = fixture::compile("firehose", "2015-08-04", &["ListDeliveryStreams"])?;
    assert!(!continues(
        &compiled,
        "firehose",
        "ListDeliveryStreams",
        FIREHOSE_FALSE
    )?);
    Ok(())
}

#[test]
fn given_json_page_with_explicit_true_flag_when_decoding_then_enumeration_continues() -> TestResult
{
    let compiled = fixture::compile("firehose", "2015-08-04", &["ListDeliveryStreams"])?;
    assert!(continues(
        &compiled,
        "firehose",
        "ListDeliveryStreams",
        FIREHOSE_TRUE
    )?);
    Ok(())
}

#[test]
fn given_xml_page_omitting_defaulted_truncation_flag_when_decoding_then_page_errors() -> TestResult
{
    let compiled = fixture::compile("iam", "2010-05-08", &["ListRoles"])?;
    let body = format!("{IAM_ROLES_OPEN}<Marker>next</Marker>{IAM_ROLES_CLOSE}");
    let error = continues(&compiled, "iam", "ListRoles", &body)
        .err()
        .ok_or("omitted completion flag must not complete the enumeration")?;
    assert!(error.to_string().contains("truncated flag"), "{error}");
    Ok(())
}

#[test]
fn given_xml_page_with_explicit_false_flag_when_decoding_then_enumeration_completes() -> TestResult
{
    let compiled = fixture::compile("iam", "2010-05-08", &["ListRoles"])?;
    let body = format!("{IAM_ROLES_OPEN}<IsTruncated>false</IsTruncated>{IAM_ROLES_CLOSE}");
    assert!(!continues(&compiled, "iam", "ListRoles", &body)?);
    Ok(())
}

#[test]
fn given_xml_page_with_explicit_true_flag_when_decoding_then_enumeration_continues() -> TestResult {
    let compiled = fixture::compile("iam", "2010-05-08", &["ListRoles"])?;
    let body = format!(
        "{IAM_ROLES_OPEN}<IsTruncated>true</IsTruncated><Marker>next</Marker>{IAM_ROLES_CLOSE}"
    );
    assert!(continues(&compiled, "iam", "ListRoles", &body)?);
    Ok(())
}

#[test]
fn given_retryable_credentials_when_mapped_then_failure_is_transient() {
    let failure = CredentialFailure {
        retryable: true,
        message: "credential provider unavailable".to_owned(),
    };
    let outcome = credential_error(failure);
    assert!(
        matches!(outcome, ApiError::Transient(message) if message == "credential provider unavailable")
    );
}

#[test]
fn given_permanent_credentials_when_mapped_then_failure_is_permanent() {
    let failure = CredentialFailure {
        retryable: false,
        message: "credential configuration invalid".to_owned(),
    };
    let outcome = credential_error(failure);
    assert!(
        matches!(outcome, ApiError::Permanent(message) if message == "credential configuration invalid")
    );
}
#[test]
fn given_s3_list_buckets_when_selecting_payload_header_then_checksum_is_included() {
    assert_eq!(
        payload_header(
            inventory_core::catalog::runtime::EndpointPathMode::Standard,
            "s3",
            "ListBuckets",
        ),
        PayloadHeader::Include,
    );
}

#[test]
fn given_other_standard_operation_when_selecting_payload_header_then_generic_mode_is_preserved() {
    assert_eq!(
        payload_header(
            inventory_core::catalog::runtime::EndpointPathMode::Standard,
            "s3",
            "GetBucketLocation",
        ),
        PayloadHeader::Omit,
    );
}

#[test]
fn given_s3_path_mode_when_selecting_payload_header_then_checksum_is_included() {
    assert_eq!(
        payload_header(
            inventory_core::catalog::runtime::EndpointPathMode::S3,
            "s3",
            "GetObject",
        ),
        PayloadHeader::Include,
    );
}
#[test]
fn given_standard_path_s3_list_buckets_when_signed_then_empty_hash_is_outgoing_and_signed() -> TestResult {
    let credentials = aws_wire::sign::Credentials::new("AKIDEXAMPLE", "fixture-secret", None)?;
    let scope = Scope {
        region: Region::new("us-east-1")?,
        name: SigningName::new("s3")?,
    };
    let headers = aws_wire::sign::sign(
        &credentials,
        &scope,
        SystemTime::UNIX_EPOCH,
        Request {
            method: &http::Method::GET,
            uri: "http://s3.amazonaws.com/",
            headers: &http::HeaderMap::new(),
            body: Body::Bytes(b""),
            payload_header: payload_header(
                inventory_core::catalog::runtime::EndpointPathMode::Standard,
                "s3",
                "ListBuckets",
            ),
        },
        PathMode::Standard,
    )?;
    assert_eq!(
        headers
            .get("x-amz-content-sha256")
            .and_then(|value| value.to_str().ok()),
        Some("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855")
    );
    let authorization = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .ok_or("authorization header absent")?;
    let signed_headers = authorization
        .split("SignedHeaders=")
        .nth(1)
        .and_then(|value| value.split(',').next())
        .ok_or("signed header list absent")?;
    assert_eq!(signed_headers, "host;x-amz-content-sha256;x-amz-date");
    Ok(())
}
