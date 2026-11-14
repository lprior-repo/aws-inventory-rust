use super::support::{OK, Server, TestResult, UNSIGNED, exchange, field, split};
use inventory_harness::proxy::{BodyStyle, Fault, Proxy};

const STYLES: [BodyStyle; 5] = [
    BodyStyle::Json,
    BodyStyle::QueryXml,
    BodyStyle::RestXml,
    BodyStyle::Ec2Xml,
    BodyStyle::Cbor,
];
type Expected = (u16, &'static str);

async fn exercise(fault: Fault) -> TestResult {
    for style in STYLES {
        super::clock::within("exact canned error envelope", exercise_style(fault, style)).await?;
    }
    Ok(())
}

async fn exercise_style(fault: Fault, style: BodyStyle) -> TestResult {
    let server = Server::start(OK).await?;
    let mut config = server.fault(fault);
    config.default_style = style;
    let proxy = Proxy::start(config).await?;
    let bytes = exchange(proxy.address(), UNSIGNED).await?;
    let (status, code) = expected(fault, style)?;
    check_headers(&bytes, status, code)?;
    super::envelopes::check(style, &bytes, code, fault)?;
    assert_eq!(proxy.snapshot()?.counters.forwarded, 0);
    assert_eq!(proxy.snapshot()?.counters.faults.get(&fault), Some(&1));
    super::support::shutdown(proxy).await?;
    Ok(())
}

fn check_headers(bytes: &[u8], status: u16, code: &str) -> TestResult {
    assert!(bytes.starts_with(format!("HTTP/1.1 {status} ").as_bytes()));
    assert_eq!(field(bytes, "x-amzn-ErrorType")?, code);
    assert!(field(bytes, "x-amzn-RequestId")?.starts_with("proxy-0000000000000000-"));
    assert_eq!(
        field(bytes, "content-length")?.parse::<usize>()?,
        split(bytes)?.1.len()
    );
    Ok(())
}

fn expected(fault: Fault, style: BodyStyle) -> Result<Expected, Box<dyn std::error::Error>> {
    use BodyStyle::{Ec2Xml, QueryXml, RestXml};
    Ok(match (fault, style) {
        (Fault::Throttle, RestXml) => (503, "SlowDown"),
        (Fault::Throttle, Ec2Xml) => (503, "RequestLimitExceeded"),
        (Fault::Throttle, _) => (400, "ThrottlingException"),
        (Fault::InternalFailure, _) => (500, "InternalFailure"),
        (Fault::ServiceUnavailable, _) => (503, "ServiceUnavailable"),
        (Fault::AccessDenied, RestXml) => (403, "AccessDenied"),
        (Fault::AccessDenied, Ec2Xml) => (403, "UnauthorizedOperation"),
        (Fault::AccessDenied, _) => (403, "AccessDeniedException"),
        (Fault::ExpiredPaginationToken, _) => (400, "ExpiredNextTokenException"),
        (Fault::ExpiredCredentials, RestXml) => (403, "ExpiredToken"),
        (Fault::ExpiredCredentials, Ec2Xml) => (403, "RequestExpired"),
        (Fault::ExpiredCredentials, _) => (403, "ExpiredTokenException"),
        (Fault::ExpiredSignature, RestXml) => (403, "RequestTimeTooSkewed"),
        (Fault::ExpiredSignature, QueryXml) => (403, "SignatureDoesNotMatch"),
        (Fault::ExpiredSignature, _) => (403, "InvalidSignatureException"),
        _ => return Err("not a canned fault".into()),
    })
}

#[tokio::test]
async fn throttle_observes_each_protocol_status_and_code() -> TestResult {
    exercise(Fault::Throttle).await
}
#[tokio::test]
async fn internal_failure_observes_each_protocol_status_and_code() -> TestResult {
    exercise(Fault::InternalFailure).await
}
#[tokio::test]
async fn unavailable_observes_each_protocol_status_and_code() -> TestResult {
    exercise(Fault::ServiceUnavailable).await
}
#[tokio::test]
async fn access_denied_observes_each_protocol_status_and_code() -> TestResult {
    exercise(Fault::AccessDenied).await
}
#[tokio::test]
async fn pagination_expiration_observes_each_protocol_status_and_code() -> TestResult {
    exercise(Fault::ExpiredPaginationToken).await
}
#[tokio::test]
async fn credential_expiration_observes_each_protocol_status_and_code() -> TestResult {
    exercise(Fault::ExpiredCredentials).await
}
#[tokio::test]
async fn signature_expiration_observes_each_protocol_status_code_and_prefix() -> TestResult {
    exercise(Fault::ExpiredSignature).await
}

#[tokio::test]
async fn service_style_overrides_default_without_affecting_other_services() -> TestResult {
    // Given a REST XML S3 override and JSON default.
    let server = Server::start(OK).await?;
    let config = style_override(&server)?;
    let proxy = Proxy::start(config).await?;
    // When separate service scopes hit the same proxy, then envelopes differ.
    let s3 = exchange(
        proxy.address(),
        &super::support::request("key", "us-east-1", "s3"),
    )
    .await?;
    let ec2 = exchange(
        proxy.address(),
        &super::support::request("key", "us-east-1", "ec2"),
    )
    .await?;
    assert_eq!(field(&s3, "x-amzn-ErrorType")?, "SlowDown");
    assert_eq!(field(&ec2, "x-amzn-ErrorType")?, "ThrottlingException");
    super::envelopes::check(BodyStyle::RestXml, &s3, "SlowDown", Fault::Throttle)?;
    super::envelopes::check(
        BodyStyle::Json,
        &ec2,
        "ThrottlingException",
        Fault::Throttle,
    )?;
    super::support::shutdown(proxy).await?;
    Ok(())
}

fn style_override(
    server: &Server,
) -> Result<inventory_harness::proxy::Config, Box<dyn std::error::Error>> {
    let mut config = server.config();
    config.rules.push(inventory_harness::proxy::Rule {
        filter: Default::default(),
        probability: inventory_harness::proxy::Probability::new(1_000_000)?,
        fault: Fault::Throttle,
    });
    config.styles.push((
        inventory_harness::proxy::ScopeName::parse("s3")?,
        BodyStyle::RestXml,
    ));
    Ok(config)
}
