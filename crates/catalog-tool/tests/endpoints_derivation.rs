pub mod endpoints_support;

use catalog_tool::endpoints::{DerivedEndpoint, RuleSet, ServiceClass, build_report};
use endpoints_support::report::{fixture, write_service};
use endpoints_support::{TestResult, rules};
use serde_json::{Value, json};
use std::path::Path;

#[test]
fn fixed_url_with_regional_signing_is_regional() -> TestResult {
    assert_eq!(
        derive(
            json!("https://fixed.test"),
            signing(json!("{Region}"), None)
        )?,
        resolved("https://fixed.test", "trait-sign", "{region}")
    );
    Ok(())
}

#[test]
fn regional_url_with_fixed_signing_is_regional() -> TestResult {
    assert_eq!(
        derive(
            json!("https://svc.{Region}.test"),
            signing(json!("us-east-1"), None)
        )?,
        resolved("https://svc.{region}.test", "trait-sign", "us-east-1")
    );
    Ok(())
}

#[test]
fn modeled_signing_name_overrides_the_sigv4_trait() -> TestResult {
    assert_eq!(
        derive(
            json!("https://svc.{Region}.test"),
            signing(json!("us-east-1"), Some("modeled-sign"))
        )?,
        resolved("https://svc.{region}.test", "modeled-sign", "us-east-1")
    );
    Ok(())
}

#[test]
fn absent_auth_schemes_fall_back_to_trait_name_and_each_probe_region() -> TestResult {
    assert_eq!(
        derive(json!("https://svc.{Region}.test"), json!({}))?,
        resolved("https://svc.{region}.test", "trait-sign", "{region}")
    );
    Ok(())
}

fn derive(url: Value, properties: Value) -> Result<DerivedEndpoint, Box<dyn std::error::Error>> {
    let document = rules(
        json!({"Region":{"type":"string"}}),
        json!([
            {"type":"endpoint","conditions":[],"endpoint":{"url":url,"properties":properties}}
        ]),
    );
    Ok(RuleSet::parse(Path::new("derive.json"), &document)?.derive(Some("trait-sign"))?)
}

fn signing(region: Value, name: Option<&str>) -> Value {
    let scheme = match name {
        Some(name) => json!({"name":"sigv4","signingRegion":region,"signingName":name}),
        None => json!({"name":"sigv4","signingRegion":region}),
    };
    json!({"authSchemes":[scheme]})
}

fn resolved(url: &str, name: &str, region: &str) -> DerivedEndpoint {
    DerivedEndpoint::Resolved {
        class: ServiceClass::Regional,
        url_template: url.into(),
        signing_name: name.into(),
        signing_region: region.into(),
    }
}

#[test]
fn modeled_errors_classify_optional_missing_parameters_and_unrelated_failures() -> TestResult {
    let overlay = error_rules("Bucket is required");
    let parsed = RuleSet::parse(Path::new("overlay.json"), &overlay)?;
    assert_eq!(
        parsed.derive(Some("svc"))?,
        DerivedEndpoint::NeedsOverlay {
            parameter: "Bucket".into()
        }
    );
    let unrelated = error_rules("unrelated modeled failure");
    let parsed = RuleSet::parse(Path::new("unsupported.json"), &unrelated)?;
    assert_eq!(
        parsed.derive(Some("svc"))?,
        DerivedEndpoint::Unsupported {
            reason: "unrelated modeled failure".into()
        }
    );
    Ok(())
}

fn error_rules(message: &str) -> Value {
    rules(
        json!({"Bucket":{"type":"string"}}),
        json!([{"type":"error","conditions":[],"error":message}]),
    )
}

#[test]
fn four_service_report_pins_non_resolved_rows_and_totals() -> TestResult {
    let dir = fixture()?;
    write_service(
        dir.path(),
        "overlay",
        error_rules("Bucket is required"),
        json!([]),
    )?;
    write_service(
        dir.path(),
        "unsupported",
        error_rules("unrelated modeled failure"),
        json!([]),
    )?;
    assert_eq!(
        build_report(dir.path())?,
        concat!(
            "cases 11\nin_scope 4\npassed 3\nfailed 1\nskipped 7\n",
            "skipped other_partition 1\nskipped fips 1\nskipped dual_stack 1\n",
            "skipped custom_endpoint 1\nskipped other_parameters 1\nskipped operation_inputs 1\n",
            "skipped request_time_function 1\nservices 4\nglobal 1\nregional 1\n",
            "needs_overlay 1\nunsupported 1\n",
            "services_with_exceptions 0\nexceptions 0\n",
            "service global global https://global.example global-sign us-east-1\n",
            "service overlay needs-overlay - - - missing:Bucket\n",
            "service regional regional https://regional.{region}.example regional-sign {region}\n",
            "unsupported_case regional 8 parseURL\n",
            "service unsupported unsupported - - - \"unrelated modeled failure\"\n"
        )
    );
    Ok(())
}
