pub mod endpoints_support;
use catalog_tool::endpoints::{Error, Fault, Outcome, RuleSet};
use endpoints_support::{TestResult, assert_value, call, endpoint, resolve, rules};
use serde_json::json;
use std::path::Path;

#[test]
fn commercial_partition_exposes_the_published_signing_and_dns_properties() -> TestResult {
    [
        "us-west-2",
        "eu-west-1",
        "il-central-1",
        "mx-central-1",
        "aws-global",
        "ap-future-99",
    ]
    .into_iter()
    .try_for_each(|region| {
        [
            ("name", json!("aws")),
            ("dnsSuffix", json!("amazonaws.com")),
            ("dualStackDnsSuffix", json!("api.aws")),
            ("supportsFIPS", json!(true)),
            ("supportsDualStack", json!(true)),
            ("implicitGlobalRegion", json!("us-east-1")),
        ]
        .into_iter()
        .try_for_each(|(key, expected)| {
            assert_value(
                call(
                    "getAttr",
                    json!([call("aws.partition", json!([region])), key]),
                ),
                expected,
            )
        })
    })
}

#[test]
fn partition_rejects_sovereign_gov_and_unknown_regions_with_a_typed_filename_error() -> TestResult {
    [
        "cn-north-1",
        "us-gov-west-1",
        "us-iso-east-1",
        "eu-isoe-west-1",
        "eusc-de-east-1",
        "moon-1",
    ]
    .into_iter()
    .try_for_each(|region| {
        let document = rules(
            json!({}),
            json!([endpoint(
                json!("https://unused"),
                json!([call("aws.partition", json!([region]))])
            )]),
        );
        let rules = RuleSet::parse(Path::new("partition.json"), &document)?;
        let error = rules.evaluate(&Default::default());
        assert!(
            matches!(error, Err(Error::Model { file, kind: Fault::Partition(r) })
                if file == Path::new("partition.json") && r == region)
        );
        Ok(())
    })
}

#[test]
fn attribute_paths_follow_nested_keys_and_checked_array_indices() -> TestResult {
    [
        (json!([["zero", "one"], "[1]"]), json!("one")),
        (
            json!([[{"a":{"b":["nested"]}}], "[0].a.b[0]"]),
            json!("nested"),
        ),
        (json!([["zero"], "[1]"]), json!(null)),
        (json!([["zero"], "[-1]"]), json!(null)),
        (json!([["zero"], "[18446744073709551616]"]), json!(null)),
        (json!([[{"a":"v"}], "[0].missing"]), json!(null)),
    ]
    .into_iter()
    .try_for_each(|(args, expected)| assert_value(call("getAttr", args), expected))
}

#[test]
fn attribute_missing_partition_key_returns_none() -> TestResult {
    assert_value(
        call(
            "getAttr",
            json!([call("aws.partition", json!(["us-east-1"])), "missing"]),
        ),
        json!(null),
    )
}

#[test]
fn endpoint_function_url_and_headers_expand_together() -> TestResult {
    let document = rules(
        json!({"Region":{"type":"string"}}),
        json!([{
        "type":"endpoint","conditions":[],"endpoint":{
            "url":"https://svc.{Region}.test", "properties":{"authSchemes":[{
                "name":"sigv4","signingName":"svc","signingRegion":"{Region}"}]},
            "headers":{"x-region":["{Region}","fixed"]}}}]),
    );
    let actual = resolve(&document, json!({"Region":"us-west-2"}))?;
    let expected: Outcome = serde_json::from_value(json!({"endpoint":{
        "url":"https://svc.us-west-2.test","properties":{"authSchemes":[{
            "name":"sigv4","signingName":"svc","signingRegion":"us-west-2"}]},
        "headers":{"x-region":["us-west-2","fixed"]}}}))?;
    assert_eq!(actual, expected);
    Ok(())
}
