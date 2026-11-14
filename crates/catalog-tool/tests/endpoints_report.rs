pub mod endpoints_support;
use catalog_tool::endpoints::{
    DerivedEndpoint, Error, ReportMode, RuleSet, ServiceClass, build_report, run,
};
use endpoints_support::report::{REPORT, fixture, global_rules, regional_rules};
use endpoints_support::{TestResult, call, endpoint, rules};
use serde_json::json;
use std::path::Path;

#[test]
fn the_two_service_report_counts_each_scope_bucket_and_identifies_a_reached_function() -> TestResult
{
    // Given independently authored global/regional models and cases, when reported, then text is exact.
    let dir = fixture()?;
    assert_eq!(build_report(dir.path())?, REPORT);
    assert_eq!(build_report(dir.path())?, REPORT);
    Ok(())
}

#[test]
fn print_does_not_write_and_write_check_use_exact_report_bytes() -> TestResult {
    // Given the fixture, when printing, then no report file is created.
    let dir = fixture()?;
    let path = dir.path().join("catalog/aws-endpoints.report");
    run(dir.path(), dir.path(), ReportMode::Print)?;
    assert!(!path.exists());
    // When writing and checking, then complete canonical bytes are persisted unchanged.
    run(dir.path(), dir.path(), ReportMode::Write)?;
    assert_eq!(std::fs::read_to_string(&path)?, REPORT);
    run(dir.path(), dir.path(), ReportMode::Check)?;
    std::fs::write(&path, REPORT.replace('\n', "\r\n"))?;
    assert!(matches!(run(dir.path(),dir.path(),ReportMode::Check),
        Err(Error::ReportMismatch(actual)) if actual==path));
    Ok(())
}

#[test]
fn malformed_later_input_never_overwrites_a_preexisting_report() -> TestResult {
    // Given a later invalid model and a committed report, when writing, then old bytes survive.
    let dir = fixture()?;
    let path = dir.path().join("catalog/aws-endpoints.report");
    std::fs::create_dir_all(dir.path().join("catalog"))?;
    std::fs::write(&path, "preserve\n")?;
    std::fs::write(dir.path().join("models/z-malformed.json"), "{")?;
    assert!(matches!(
        run(dir.path(), dir.path(), ReportMode::Write),
        Err(Error::Input(_))
    ));
    assert_eq!(std::fs::read_to_string(&path)?, "preserve\n");
    Ok(())
}

#[test]
fn two_commercial_region_probes_distinguish_global_and_regional_signing() -> TestResult {
    // Given fixed and regional endpoints, when derived, then URL and signing defaults are preserved.
    let global = RuleSet::parse(Path::new("global.json"), &global_rules())?;
    assert_eq!(
        global.derive(Some("global-sign"))?,
        DerivedEndpoint::Resolved {
            class: ServiceClass::Global,
            url_template: "https://global.example".into(),
            signing_name: "global-sign".into(),
            signing_region: "us-east-1".into()
        }
    );
    let regional = RuleSet::parse(Path::new("regional.json"), &regional_rules())?;
    assert_eq!(
        regional.derive(Some("regional-sign"))?,
        DerivedEndpoint::Resolved {
            class: ServiceClass::Regional,
            url_template: "https://regional.{region}.example".into(),
            signing_name: "regional-sign".into(),
            signing_region: "{region}".into()
        }
    );
    Ok(())
}

#[test]
fn missing_request_parameters_and_reached_request_functions_name_the_overlay_requirement()
-> TestResult {
    // Given a required request input, when deriving without it, then its name is retained.
    let required = rules(
        json!({"Bucket":{"type":"string","required":true}}),
        json!([endpoint(json!("unused"), json!([]))]),
    );
    let parsed = RuleSet::parse(Path::new("bucket.json"), &required)?;
    assert_eq!(
        parsed.derive(Some("s3"))?,
        DerivedEndpoint::NeedsOverlay {
            parameter: "Bucket".into()
        }
    );
    let request = rules(
        json!({}),
        json!([endpoint(
            json!("unused"),
            json!([call("parseURL", json!([]))])
        )]),
    );
    let parsed = RuleSet::parse(Path::new("request.json"), &request)?;
    assert_eq!(
        parsed.derive(Some("svc"))?,
        DerivedEndpoint::NeedsOverlay {
            parameter: "function:parseURL".into()
        }
    );
    Ok(())
}

#[test]
fn a_service_without_sigv4_is_explicitly_unsupported() -> TestResult {
    // Given no SigV4 trait, when deriving, then signing is not invented from its directory name.
    let parsed = RuleSet::parse(Path::new("bearer.json"), &global_rules())?;
    assert_eq!(
        parsed.derive(None)?,
        DerivedEndpoint::Unsupported {
            reason: "no SigV4 trait".into()
        }
    );
    Ok(())
}
