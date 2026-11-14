pub mod endpoints_support;

use catalog_tool::endpoints::{
    Case, DerivedEndpoint, DerivedService, EndpointException, Outcome, RuleSet, build_report,
};
use endpoints_support::exceptions::{REPORT, case, fixtures, route};
use endpoints_support::report::write_service;
use endpoints_support::{TestResult, call, rules};
use serde_json::json;
use std::path::Path;

#[test]
fn every_passing_fixture_case_is_reproduced_by_its_template_or_an_exact_exception() -> TestResult {
    for fixture in fixtures() {
        let parsed = RuleSet::parse(Path::new(fixture.name), &fixture.document)?;
        let cases: Vec<Case> = serde_json::from_value(fixture.cases)?;
        let counts = parsed.run_cases(&cases)?;
        assert_eq!((counts.passed, counts.failed), (cases.len(), 0));
        let name = format!("{}-sign", fixture.name);
        let derived = counts.derive(&parsed, Some(&name))?;
        for case in cases {
            reproduced(&derived, &case, &name)?;
        }
    }
    Ok(())
}

fn reproduced(derived: &DerivedService, case: &Case, name: &str) -> TestResult {
    let Outcome::Endpoint(expected) = &case.expect else {
        return Err("expected endpoint".into());
    };
    let region = case
        .params
        .get("Region")
        .and_then(serde_json::Value::as_str)
        .ok_or("region")?;
    let signing_region = expected
        .properties
        .get("authSchemes")
        .and_then(serde_json::Value::as_array)
        .and_then(|schemes| schemes.first())
        .and_then(|scheme| scheme.get("signingRegion"))
        .and_then(serde_json::Value::as_str)
        .ok_or("signing region")?;
    let matched = match &derived.endpoint {
        DerivedEndpoint::Resolved {
            url_template,
            signing_name,
            signing_region: template,
            ..
        }
        | DerivedEndpoint::NeedsOverlayRouting {
            url_template,
            signing_name,
            signing_region: template,
            ..
        } => {
            url_template.replace("{region}", region) == expected.url
                && signing_name == name
                && template.replace("{region}", region) == signing_region
        }
        _ => false,
    };
    assert!(
        matched
            || derived
                .exceptions
                .iter()
                .any(|exception| exception.region == region
                    && exception.url == expected.url
                    && exception.signing_name == name
                    && exception.signing_region == signing_region)
    );
    Ok(())
}

#[test]
fn real_shape_fixtures_report_global_region_exceptions_and_fixed_west_signing_exactly() -> TestResult
{
    let dir = tempfile::tempdir()?;
    for fixture in fixtures() {
        write_service(dir.path(), fixture.name, fixture.document, fixture.cases)?;
    }
    assert_eq!(build_report(dir.path())?, REPORT);
    assert_eq!(build_report(dir.path())?, REPORT);
    Ok(())
}

#[test]
fn exceptions_outnumbering_matches_require_overlay_and_are_sorted_by_region() -> TestResult {
    let document = rules(
        json!({"Region":{"type":"string"}}),
        json!([
            route(
                "https://global.test",
                json!([call("stringEquals", json!([{"ref":"Region"},"aws-global"]))]),
                "us-east-1"
            ),
            route(
                "https://africa.test",
                json!([call("stringEquals", json!([{"ref":"Region"},"af-south-1"]))]),
                "us-east-1"
            ),
            route("https://svc.{Region}.test", json!([]), "{Region}")
        ]),
    );
    let parsed = RuleSet::parse(Path::new("dominant.json"), &document)?;
    let cases = serde_json::from_value::<Vec<Case>>(json!([
        case("aws-global", "https://global.test", "us-east-1"),
        case("us-west-2", "https://svc.us-west-2.test", "us-west-2"),
        case("af-south-1", "https://africa.test", "us-east-1")
    ]))?;
    let derived = parsed.run_cases(&cases)?.derive(&parsed, Some("svc"))?;
    assert!(matches!(&derived.endpoint,
        DerivedEndpoint::NeedsOverlayRouting { matches: 1, exceptions: 2, url_template, .. }
            if url_template == "https://svc.{region}.test"));
    assert_eq!(
        derived.exceptions,
        vec![
            EndpointException {
                region: "af-south-1".into(),
                url: "https://africa.test".into(),
                signing_name: "svc".into(),
                signing_region: "us-east-1".into()
            },
            EndpointException {
                region: "aws-global".into(),
                url: "https://global.test".into(),
                signing_name: "svc".into(),
                signing_region: "us-east-1".into()
            },
        ]
    );
    let dir = tempfile::tempdir()?;
    write_service(
        dir.path(),
        "dominant",
        document,
        json!([
            case("aws-global", "https://global.test", "us-east-1"),
            case("us-west-2", "https://svc.us-west-2.test", "us-west-2"),
            case("af-south-1", "https://africa.test", "us-east-1")
        ]),
    )?;
    assert!(build_report(dir.path())?.contains(concat!(
        "service dominant needs-overlay https://svc.{region}.test dominant-sign {region} ",
        "reason:\"regional template contradicted by more published cases than it matches\"\n"
    )));
    Ok(())
}

#[test]
fn no_published_endpoint_case_retains_candidate_but_requires_overlay() -> TestResult {
    let document = rules(
        json!({"Region":{"type":"string"}}),
        json!([route("https://svc.{Region}.test", json!([]), "{Region}")]),
    );
    let parsed = RuleSet::parse(Path::new("unconfirmed.json"), &document)?;
    let derived = parsed.run_cases(&[])?.derive(&parsed, Some("svc"))?;
    assert_eq!(
        derived.endpoint,
        DerivedEndpoint::NeedsOverlayRouting {
            url_template: "https://svc.{region}.test".into(),
            signing_name: "svc".into(),
            signing_region: "{region}".into(),
            matches: 0,
            exceptions: 0,
        }
    );
    assert!(derived.exceptions.is_empty());
    Ok(())
}

#[test]
fn failed_and_skipped_cases_do_not_confirm_a_template_or_create_exceptions() -> TestResult {
    let document = rules(
        json!({"Region":{"type":"string"},"UseFIPS":{"type":"boolean"}}),
        json!([route("https://svc.{Region}.test", json!([]), "{Region}")]),
    );
    let parsed = RuleSet::parse(Path::new("evidence.json"), &document)?;
    let cases = serde_json::from_value::<Vec<Case>>(json!([
        {"params":{"Region":"us-west-2"},"expect":{"endpoint":{"url":"wrong"}}},
        {"params":{"Region":"us-west-2","UseFIPS":true},"expect":{"endpoint":{"url":"wrong"}}}
    ]))?;
    let counts = parsed.run_cases(&cases)?;
    assert_eq!((counts.passed, counts.failed), (0, 1));
    let derived = counts.derive(&parsed, Some("svc"))?;
    assert!(matches!(
        derived.endpoint,
        DerivedEndpoint::NeedsOverlayRouting {
            matches: 0,
            exceptions: 0,
            ..
        }
    ));
    assert!(derived.exceptions.is_empty());
    Ok(())
}
