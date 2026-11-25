mod fixture;

use bytes::Bytes;
use inventory_core::{
    catalog::runtime::{CallablePlan, RuntimeCatalog, RuntimeProtocol},
    record::Record,
};

use crate::{
    SweepError,
    input::{InputField, root_bindings},
};

type TestResult = Result<(), Box<dyn std::error::Error>>;
type Encoder = for<'catalog> fn(
    &CallablePlan<'catalog>,
    &str,
    &Record,
    &[InputField],
) -> Result<http::Request<Bytes>, SweepError>;

const CASES: [(&str, RuntimeProtocol, &str); 7] = [
    ("aws.protocols#restJson1", RuntimeProtocol::RestJson1, "GET"),
    ("aws.protocols#restXml", RuntimeProtocol::RestXml, "HEAD"),
    ("aws.protocols#ec2Query", RuntimeProtocol::Ec2Query, "POST"),
    ("aws.protocols#awsQuery", RuntimeProtocol::AwsQuery, "POST"),
    (
        "aws.protocols#awsJson1_0",
        RuntimeProtocol::AwsJson10,
        "POST",
    ),
    (
        "aws.protocols#awsJson1_1",
        RuntimeProtocol::AwsJson11,
        "POST",
    ),
    (
        "smithy.protocols#rpcv2Cbor",
        RuntimeProtocol::RpcV2Cbor,
        "POST",
    ),
];

#[test]
fn given_callable_plans_when_encoding_then_http_methods_are_catalog_derived() -> TestResult {
    let encoder: Encoder = super::encode;
    for (trait_name, protocol, method) in CASES {
        let bytes = fixture::compiled(trait_name, method)?;
        let catalog = RuntimeCatalog::from_bytes(&bytes)?;
        let plan = catalog
            .lookup("page", "ListThings")
            .ok_or("fixture plan missing")?;
        assert_eq!(plan.protocol(), protocol, "{trait_name}");
        let base = root_bindings(plan, None, None)?;
        let request = encoder(plan, "https://127.0.0.1:1", &base, &[])?;
        let advertised = plan.http()?.ok_or("fixture HTTP metadata missing")?.method;
        assert_eq!(request.method().as_str(), advertised, "{trait_name}");
        assert_eq!(request.method().as_str(), method, "{trait_name}");
    }
    Ok(())
}
