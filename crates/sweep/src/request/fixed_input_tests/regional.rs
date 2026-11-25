use super::*;
#[test]
fn given_regional_exception_when_request_resolved_then_only_its_region_is_overridden() -> TestResult
{
    let bytes = compiled_inputs(
        "aws.protocols#awsQuery",
        filters("neptune")?,
        overlay::FixedValue::map([("owner", overlay::FixedValue::String("inventory".into()))])?,
        exception_endpoint()?,
    )?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let plan = catalog
        .lookup("fixed", "ListClusters")
        .ok_or("missing plan")?;
    let exception = routing::resolve(plan, &RegionId::parse("eu-west-1")?)?;
    assert_eq!(exception.url, "https://fixed-fips.eu-west-1.test");
    assert_eq!(exception.signing_region, "eu-west-1");
    let template = routing::resolve(plan, &RegionId::parse("us-west-2")?)?;
    assert_eq!(template.url, "https://fixed.us-west-2.test");
    assert_eq!(template.signing_region, "us-west-2");
    Ok(())
}
