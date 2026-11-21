use super::*;
use crate::sweep_runtime::{consumer_fixture as fixture, execution::page::cursor::Cursor};
use inventory_core::catalog::runtime::RuntimeCatalog;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[test]
fn denied_http_status_is_not_retried() {
    assert!(matches!(
        status_fallback(http::StatusCode::FORBIDDEN),
        ApiError::Denied(_)
    ));
    assert!(matches!(
        status_fallback(http::StatusCode::TOO_MANY_REQUESTS),
        ApiError::Throttled(_)
    ));
}

const FIREHOSE_OMITTED: &str = r#"{"DeliveryStreamNames":["alpha"]}"#;
const FIREHOSE_FALSE: &str = r#"{"DeliveryStreamNames":["alpha"],"HasMoreDeliveryStreams":false}"#;
const FIREHOSE_TRUE: &str = r#"{"DeliveryStreamNames":["alpha"],"HasMoreDeliveryStreams":true}"#;

const IAM_ROLES_OPEN: &str = "<ListRolesResponse xmlns=\"https://iam.amazonaws.com/doc/2010-05-08/\"><ListRolesResult><Roles><member><RoleName>alpha</RoleName></member></Roles>";
const IAM_ROLES_CLOSE: &str = "</ListRolesResult><ResponseMetadata><RequestId>r</RequestId></ResponseMetadata></ListRolesResponse>";

/// Decodes a raw successful HTTP body through production `decode_response`, then asks the
/// production cursor for the next page. `Ok(true)` means the cursor continues.
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
    let page = decode_response(plan, response)?;
    let next = Cursor::default().next(plan.pagination(), &page.record, &page.headers, 10)?;
    Ok(next.is_some())
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
