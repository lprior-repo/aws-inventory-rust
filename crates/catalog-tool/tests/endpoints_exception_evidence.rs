pub mod endpoints_support;

use catalog_tool::endpoints::{
    Case, DerivedEndpoint, EndpointException, RuleSet, SkipReason, build_report,
};
use endpoints_support::exceptions::{case, route, verified};
use endpoints_support::report::write_service;
use endpoints_support::{TestResult, call, rules};
use serde_json::json;
use std::path::Path;

#[test]
fn correctly_expected_fips_case_contributes_neither_template_matches_nor_exceptions() -> TestResult
{
    let document = rules(
        json!({"Region":{"type":"string"},"UseFIPS":{"type":"boolean"}}),
        json!([route("https://svc.{Region}.test", json!([]), "{Region}")]),
    );
    let parsed = RuleSet::parse(Path::new("skipped-evidence.json"), &document)?;
    let cases: Vec<Case> = serde_json::from_value(json!([{
        "params":{"Region":"us-west-2","UseFIPS":true},
        "expect":{"endpoint":{"url":"https://svc.us-west-2.test",
            "properties":{"authSchemes":[{"name":"sigv4","signingRegion":"us-west-2"}]}}}
    }]))?;
    let first = cases.first().ok_or("missing fixture case")?;
    assert_eq!(parsed.evaluate(&first.params)?, first.expect);

    let counts = parsed.run_cases(&cases)?;
    assert_eq!(
        (counts.in_scope(), counts.passed(), counts.failed()),
        (0, 0, 0)
    );
    assert_eq!(counts.skipped().get(&SkipReason::Fips), Some(&1));
    let derived = counts.derive(Some("svc"))?;
    assert!(matches!(
        derived.endpoint(),
        DerivedEndpoint::NeedsOverlayRouting {
            matches: 0,
            exceptions: 0,
            ..
        }
    ));
    assert!(derived.exceptions().is_empty());
    Ok(())
}

#[test]
fn a_region_routed_like_us_west_2_is_an_exact_exception_not_a_probe_fallback() -> TestResult {
    let document = rules(
        json!({"Region":{"type":"string"}}),
        json!([
            route(
                "https://svc.us-west-2.test",
                json!([call(
                    "stringEquals",
                    json!([{"ref":"Region"},"eu-central-1"])
                )]),
                "us-west-2"
            ),
            route("https://svc.{Region}.test", json!([]), "{Region}")
        ]),
    );
    let cases = json!([
        case("us-west-2", "https://svc.us-west-2.test", "us-west-2"),
        case("eu-west-1", "https://svc.eu-west-1.test", "eu-west-1"),
        case("eu-central-1", "https://svc.us-west-2.test", "us-west-2")
    ]);
    let derived = verified(&document, cases.clone(), "fallback-sign")?;
    assert_eq!(
        derived.exceptions().to_vec(),
        vec![EndpointException {
            region: "eu-central-1".into(),
            url: "https://svc.us-west-2.test".into(),
            signing_name: "fallback-sign".into(),
            signing_region: "us-west-2".into(),
        }]
    );
    let dir = tempfile::tempdir()?;
    write_service(dir.path(), "fallback", document, cases)?;
    let report = build_report(dir.path())?;
    let rows: Vec<_> = report
        .lines()
        .filter(|line| line.starts_with("exception "))
        .collect();
    assert_eq!(
        rows,
        vec!["exception fallback eu-central-1 https://svc.us-west-2.test fallback-sign us-west-2"]
    );
    Ok(())
}
