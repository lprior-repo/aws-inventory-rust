pub mod endpoints_support;

use catalog_tool::endpoints::{DerivedEndpoint, EndpointException, build_report};
use endpoints_support::exceptions::{case, route, routing, verified};
use endpoints_support::report::write_service;
use endpoints_support::{TestResult, call, rules};
use serde_json::json;

#[test]
fn signing_name_only_and_signing_region_only_differences_are_each_exceptions() -> TestResult {
    let document = rules(
        json!({"Region":{"type":"string"}}),
        json!([
            {"type":"endpoint","conditions":[
                call("stringEquals",json!([{"ref":"Region"},"af-south-1"]))
            ],"endpoint":{"url":"https://svc.{Region}.test","properties":{"authSchemes":[{
                "name":"sigv4","signingName":"modeled-name","signingRegion":"{Region}"
            }]}}},
            route("https://svc.{Region}.test",
                json!([call("stringEquals",json!([{"ref":"Region"},"ap-south-1"]))]),
                "us-east-1"),
            route("https://svc.{Region}.test",json!([]),"{Region}")
        ]),
    );
    let cases = json!([
        case("us-west-2", "https://svc.us-west-2.test", "us-west-2"),
        case("eu-west-1", "https://svc.eu-west-1.test", "eu-west-1"),
        {"params":{"Region":"af-south-1"},"expect":{"endpoint":{
            "url":"https://svc.af-south-1.test","properties":{"authSchemes":[{
                "name":"sigv4","signingName":"modeled-name","signingRegion":"af-south-1"
            }]}}}},
        case("ap-south-1", "https://svc.ap-south-1.test", "us-east-1")
    ]);
    let derived = verified(&document, cases, "svc-sign")?;
    assert_eq!(
        derived.exceptions().to_vec(),
        vec![
            EndpointException {
                region: "af-south-1".into(),
                url: "https://svc.af-south-1.test".into(),
                signing_name: "modeled-name".into(),
                signing_region: "af-south-1".into(),
            },
            EndpointException {
                region: "ap-south-1".into(),
                url: "https://svc.ap-south-1.test".into(),
                signing_name: "svc-sign".into(),
                signing_region: "us-east-1".into(),
            },
        ]
    );
    Ok(())
}

#[test]
fn duplicate_exception_cases_outvote_one_match_but_emit_exactly_one_exception_row() -> TestResult {
    let document = routing("svc", "af-south-1", "https://special.test");
    let cases = json!([
        case("us-west-2", "https://svc.us-west-2.test", "us-west-2"),
        case("af-south-1", "https://special.test", "us-east-1"),
        case("af-south-1", "https://special.test", "us-east-1")
    ]);
    let derived = verified(&document, cases.clone(), "repeated-sign")?;
    assert!(matches!(
        derived.endpoint(),
        DerivedEndpoint::NeedsOverlayRouting {
            matches: 1,
            exceptions: 2,
            ..
        }
    ));
    assert_eq!(
        derived.exceptions().to_vec(),
        vec![EndpointException {
            region: "af-south-1".into(),
            url: "https://special.test".into(),
            signing_name: "repeated-sign".into(),
            signing_region: "us-east-1".into(),
        }]
    );
    let dir = tempfile::tempdir()?;
    write_service(dir.path(), "repeated", document, cases)?;
    let report = build_report(dir.path())?;
    let rows: Vec<_> = report
        .lines()
        .filter(|line| line.starts_with("exception "))
        .collect();
    assert_eq!(
        rows,
        vec!["exception repeated af-south-1 https://special.test repeated-sign us-east-1"]
    );
    Ok(())
}
