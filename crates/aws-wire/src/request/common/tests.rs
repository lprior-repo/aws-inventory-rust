use super::{BoundedBody, finish};
use crate::request::{EncodeError, Protocol, RequestContext, RequestLimits};
use catalog_tool::smithy::{Registry, build_catalog};
use http::{HeaderMap, HeaderValue, header::HOST};
use inventory_core::{
    catalog::{Catalog, Operation, Service},
    record::MapRef,
};
use std::{error::Error, fs};

const MODEL: &str = r#"{"smithy":"2.0","shapes":{
 "example#Svc":{"type":"service","version":"2020-01-01","operations":[{"target":"example#Call"}],"traits":{"aws.api#service":{"sdkId":"Example","endpointPrefix":"example"},"aws.protocols#restJson1":{}}},
 "example#Call":{"type":"operation","input":{"target":"example#Input"},"output":{"target":"example#Output"}},
 "example#Input":{"type":"structure","members":{}},
 "example#Output":{"type":"structure","members":{}}
}}"#;

fn catalog_fixture() -> Result<(tempfile::TempDir, Vec<u8>), Box<dyn Error>> {
    let root = tempfile::tempdir()?;
    let model = root.path().join("models/example/model.json");
    fs::create_dir_all(model.parent().ok_or("model has no parent")?)?;
    fs::write(model, MODEL)?;
    let registry = Registry::parse("")?;
    let bytes = build_catalog(root.path(), &registry, "request-bounds")?;
    Ok((root, bytes))
}

fn request_context<'a>(
    catalog: &'a Catalog<'a>,
    endpoint: &'a str,
    limits: RequestLimits,
) -> Result<RequestContext<'a>, Box<dyn Error>> {
    let service: Service<'a> = catalog
        .service("example")?
        .ok_or("service fixture missing")?;
    let operation: Operation<'a> = service
        .operations()?
        .next()
        .ok_or("operation fixture missing")??;
    Ok(RequestContext {
        service,
        operation,
        protocol: Protocol::RestJson1,
        input: MapRef::from_bytes_bounded(&[0xa0])?,
        endpoint,
        limits,
        idempotency_token: None,
        compression: Default::default(),
        addressing: None,
    })
}

fn default_limits() -> RequestLimits {
    RequestLimits::default()
}

#[test]
fn request_limits_reject_zero_and_values_above_global_caps() {
    assert!(matches!(
        RequestLimits::new(0, 1, 1),
        Err(EncodeError::Limit("body"))
    ));
    assert!(matches!(
        RequestLimits::new(1, 0, 1),
        Err(EncodeError::Limit("headers"))
    ));
    assert!(matches!(
        RequestLimits::new(1, 1, 0),
        Err(EncodeError::Limit("URI"))
    ));
    assert!(matches!(
        RequestLimits::new(16 * 1024 * 1024 + 1, 1, 1),
        Err(EncodeError::Limit("body"))
    ));
    assert!(matches!(
        RequestLimits::new(1, 65_537, 1),
        Err(EncodeError::Limit("headers"))
    ));
    assert!(matches!(
        RequestLimits::new(1, 1, 16_385),
        Err(EncodeError::Limit("URI"))
    ));
}

#[test]
fn bounded_body_accepts_exact_budget_and_rejects_next_byte_without_mutating_prefix() {
    let mut body = BoundedBody::new(3);
    assert!(matches!(body.write(b"ab"), Ok(())));
    assert!(matches!(body.write(b"c"), Ok(())));
    assert!(matches!(body.write(b"d"), Err(EncodeError::Limit("body"))));
    assert_eq!(body.into_vec(), b"abc");
}

#[test]
fn finish_rejects_malformed_endpoint_forms() -> Result<(), Box<dyn Error>> {
    let (_temp, bytes) = catalog_fixture()?;
    let catalog = Catalog::borrow(&bytes)?;
    for endpoint in [
        "/relative",
        "ftp://example.test",
        "https://example.test/path?q=1",
        "https://u:p@example.test/",
        "https://example.test/#frag",
    ] {
        let context = request_context(&catalog, endpoint, default_limits())?;
        assert!(matches!(
            finish(context, "POST", None, HeaderMap::new(), Vec::new()),
            Err(EncodeError::InvalidEndpoint)
        ));
    }
    Ok(())
}

#[test]
fn finish_appends_operation_target_without_losing_endpoint_base_path() -> Result<(), Box<dyn Error>>
{
    let (_temp, bytes) = catalog_fixture()?;
    let catalog = Catalog::borrow(&bytes)?;
    for endpoint in ["https://example.test/base/", "https://example.test/base"] {
        let context = request_context(&catalog, endpoint, default_limits())?;
        let request = finish(
            context,
            "POST",
            Some("/items?next=2"),
            HeaderMap::new(),
            Vec::new(),
        )?;
        assert_eq!(
            request.uri().to_string(),
            "https://example.test/base/items?next=2"
        );
        assert_eq!(
            request
                .headers()
                .get(HOST)
                .ok_or("installed host missing")?,
            "example.test"
        );
    }
    Ok(())
}

#[test]
fn finish_counts_absolute_uri_scheme_authority_and_path_at_boundary() -> Result<(), Box<dyn Error>>
{
    let (_temp, bytes) = catalog_fixture()?;
    let catalog = Catalog::borrow(&bytes)?;
    let endpoint = "https://example.test/base";
    let target = "/x";
    let exact = endpoint.len() + target.len();
    let limits = RequestLimits::new(1, 1024, exact)?;
    let request = finish(
        request_context(&catalog, endpoint, limits)?,
        "POST",
        Some(target),
        HeaderMap::new(),
        Vec::new(),
    )?;
    assert_eq!(request.uri().to_string(), "https://example.test/base/x");
    let short = RequestLimits::new(1, 1024, exact.checked_sub(1).ok_or("URI budget underflow")?)?;
    assert!(matches!(
        finish(
            request_context(&catalog, endpoint, short)?,
            "POST",
            Some(target),
            HeaderMap::new(),
            Vec::new()
        ),
        Err(EncodeError::Limit("URI"))
    ));
    Ok(())
}

#[test]
fn finish_rejects_mismatched_or_duplicate_host_and_retains_ipv6_authority()
-> Result<(), Box<dyn Error>> {
    let (_temp, bytes) = catalog_fixture()?;
    let catalog = Catalog::borrow(&bytes)?;
    let mut wrong = HeaderMap::new();
    wrong.insert(HOST, HeaderValue::from_static("elsewhere.test"));
    assert!(matches!(
        finish(
            request_context(&catalog, "https://example.test", default_limits())?,
            "POST",
            None,
            wrong,
            Vec::new()
        ),
        Err(EncodeError::InvalidEndpoint)
    ));
    let mut duplicate = HeaderMap::new();
    duplicate.append(HOST, HeaderValue::from_static("example.test"));
    duplicate.append(HOST, HeaderValue::from_static("example.test"));
    assert!(matches!(
        finish(
            request_context(&catalog, "https://example.test", default_limits())?,
            "POST",
            None,
            duplicate,
            Vec::new()
        ),
        Err(EncodeError::InvalidEndpoint)
    ));
    let request = finish(
        request_context(
            &catalog,
            "https://[2001:db8::1]:8443/base",
            default_limits(),
        )?,
        "POST",
        None,
        HeaderMap::new(),
        Vec::new(),
    )?;
    assert_eq!(
        request.headers().get(HOST).ok_or("IPv6 host missing")?,
        "[2001:db8::1]:8443"
    );
    Ok(())
}

#[test]
fn finish_applies_aggregate_header_budget_including_host_and_header_overhead()
-> Result<(), Box<dyn Error>> {
    let (_temp, bytes) = catalog_fixture()?;
    let catalog = Catalog::borrow(&bytes)?;
    let endpoint = "https://x";
    let mut headers = HeaderMap::new();
    headers.insert("x-a", HeaderValue::from_static("b"));
    let total = "x-a".len()
        + "b".len()
        + 4
        + "host".len()
        + "x".len()
        + 4
        + "content-length".len()
        + "1".len()
        + 4;
    let exact = RequestLimits::new(1, total, 1024)?;
    let request = finish(
        request_context(&catalog, endpoint, exact)?,
        "POST",
        None,
        headers.clone(),
        vec![b'x'],
    )?;
    assert_eq!(
        request
            .headers()
            .get("x-a")
            .ok_or("custom header missing")?,
        "b"
    );
    assert_eq!(
        request
            .headers()
            .get("content-length")
            .ok_or("length missing")?,
        "1"
    );
    let short = RequestLimits::new(
        1,
        total.checked_sub(1).ok_or("header budget underflow")?,
        1024,
    )?;
    assert!(matches!(
        finish(
            request_context(&catalog, endpoint, short)?,
            "POST",
            None,
            headers,
            vec![b'x']
        ),
        Err(EncodeError::Limit("headers"))
    ));
    Ok(())
}

#[test]
fn finish_rejects_body_already_over_caller_budget() -> Result<(), Box<dyn Error>> {
    let (_temp, bytes) = catalog_fixture()?;
    let catalog = Catalog::borrow(&bytes)?;
    let limits = RequestLimits::new(2, 1024, 1024)?;
    assert!(matches!(
        finish(
            request_context(&catalog, "https://example.test", limits)?,
            "POST",
            None,
            HeaderMap::new(),
            vec![1, 2, 3]
        ),
        Err(EncodeError::Limit("body"))
    ));
    Ok(())
}
