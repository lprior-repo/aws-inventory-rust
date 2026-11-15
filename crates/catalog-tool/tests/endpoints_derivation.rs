pub mod endpoints_support;

use catalog_tool::endpoints::{
    DerivedEndpoint, Error, Fault, OverlayNeed, RuleSet, ServiceClass, build_report,
};
use endpoints_support::report::{fixture, write_service};
use endpoints_support::{TestResult, rules};
use serde_json::{Value, json};
use std::collections::BTreeMap;
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
fn modeled_error_text_is_not_typed_missing_parameter() -> TestResult {
    let overlay = error_rules("Bucket is required");
    let parsed = RuleSet::parse(Path::new("overlay.json"), &overlay)?;
    assert_eq!(
        parsed.derive(Some("svc"))?,
        DerivedEndpoint::Unsupported {
            reason: "Bucket is required".into()
        }
    );
    let misleading = error_rules("Not a Bucket problem");
    let parsed = RuleSet::parse(Path::new("unsupported.json"), &misleading)?;
    assert_eq!(
        parsed.derive(Some("svc"))?,
        DerivedEndpoint::Unsupported {
            reason: "Not a Bucket problem".into()
        }
    );
    Ok(())
}

#[test]
fn evaluator_missing_parameter_remains_a_typed_overlay_requirement() -> TestResult {
    let missing = rules(
        json!({"Bucket":{"type":"string","required":true}}),
        json!([endpoints_support::endpoint(
            json!({"ref":"Bucket"}),
            json!([])
        )]),
    );
    let parsed = RuleSet::parse(Path::new("missing.json"), &missing)?;
    assert!(matches!(
        parsed.evaluate(&BTreeMap::new()),
        Err(Error::Model { kind: Fault::MissingParameter(name), .. }) if name == "Bucket"
    ));
    assert_eq!(
        parsed.derive(Some("svc"))?,
        DerivedEndpoint::NeedsOverlay {
            need: OverlayNeed::Parameter("Bucket".into())
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
            "needs_overlay 0\nunsupported 2\n",
            "services_with_exceptions 0\nexceptions 0\n",
            "service global global https://global.example global-sign us-east-1\n",
            "service overlay unsupported - - - \"Bucket is required\"\n",
            "service regional regional https://regional.{region}.example regional-sign {region}\n",
            "unsupported_case regional 8 parseURL\n",
            "case_failure regional 1 \"actual outcome differs from expected outcome\"\n",
            "service unsupported unsupported - - - \"unrelated modeled failure\"\n"
        )
    );
    Ok(())
}
