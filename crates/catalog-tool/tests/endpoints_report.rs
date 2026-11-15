pub mod endpoints_support;
use catalog_tool::ReportMode;
use catalog_tool::endpoints::{
    DerivedEndpoint, Error, OverlayNeed, RuleSet, ServiceClass, build_report, run,
};
use endpoints_support::report::{REPORT, fixture, global_rules, regional_rules, write_service};
use endpoints_support::{TestResult, call, endpoint, rules};
use serde_json::json;
use std::path::Path;

#[test]
fn the_two_service_report_counts_each_scope_bucket_and_identifies_a_reached_function() -> TestResult
{
    let dir = fixture()?;
    assert_eq!(build_report(dir.path())?, REPORT);
    assert_eq!(build_report(dir.path())?, REPORT);
    Ok(())
}

#[test]
fn print_does_not_write_and_write_check_use_exact_report_bytes() -> TestResult {
    let dir = fixture()?;
    let path = dir.path().join("catalog/aws-endpoints.report");
    run(dir.path(), dir.path(), ReportMode::Print)?;
    assert!(!path.exists());

    run(dir.path(), dir.path(), ReportMode::Write)?;
    assert_eq!(std::fs::read_to_string(&path)?, REPORT);
    run(dir.path(), dir.path(), ReportMode::Check)?;
    std::fs::write(&path, REPORT.replace('\n', "\r\n"))?;
    let expected = REPORT
        .split_inclusive('\n')
        .enumerate()
        .map(|(index, line)| {
            format!(
                "line {} differs: expected Some({line:?}), got Some({:?})\n",
                index.saturating_add(1),
                line.replace('\n', "\r\n")
            )
        })
        .collect::<Vec<_>>()
        .concat();
    match run(dir.path(), dir.path(), ReportMode::Check) {
        Err(Error::Report(catalog_tool::ReportError::Mismatch(actual))) => {
            assert_eq!(actual, expected);
        }
        other => return Err(format!("expected Report(Mismatch), got {other:?}").into()),
    }
    Ok(())
}

#[test]
fn malformed_later_input_never_overwrites_a_preexisting_report() -> TestResult {
    let dir = fixture()?;
    let path = dir.path().join("catalog/aws-endpoints.report");
    std::fs::create_dir_all(dir.path().join("catalog"))?;
    std::fs::write(&path, "preserve\n")?;
    std::fs::write(dir.path().join("models/z-malformed.json"), "{")?;
    assert!(matches!(
        run(dir.path(), dir.path(), ReportMode::Write),
        Err(Error::Source(_))
    ));
    assert_eq!(std::fs::read_to_string(&path)?, "preserve\n");
    Ok(())
}

#[test]
fn duplicate_shape_names_are_rejected_at_the_shared_model_source_boundary() -> TestResult {
    let dir = tempfile::tempdir()?;
    std::fs::create_dir_all(dir.path().join("models"))?;
    std::fs::write(
        dir.path().join("models/duplicate.json"),
        r#"{"shapes":{"sample#Svc":{"type":"service"},"sample#Svc":{"type":"service"}}}"#,
    )?;
    assert!(matches!(build_report(dir.path()), Err(Error::Source(_))));
    Ok(())
}

#[test]
fn two_commercial_region_probes_distinguish_global_and_regional_signing() -> TestResult {
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
fn missing_request_parameters_are_typed_overlay_requirements() -> TestResult {
    let required = rules(
        json!({"Bucket":{"type":"string","required":true}}),
        json!([endpoint(json!("unused"), json!([]))]),
    );
    let parsed = RuleSet::parse(Path::new("bucket.json"), &required)?;
    assert_eq!(
        parsed.derive(Some("s3"))?,
        DerivedEndpoint::NeedsOverlay {
            need: OverlayNeed::Parameter("Bucket".into())
        }
    );
    Ok(())
}

#[test]
fn reached_request_functions_preserve_their_typed_reason() -> TestResult {
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
            need: OverlayNeed::UnsupportedFunction("parseURL".into())
        }
    );
    let dir = tempfile::tempdir()?;
    write_service(dir.path(), "request", request, json!([]))?;
    assert!(
        build_report(dir.path())?
            .contains("service request needs-overlay - - - unsupported-function:parseURL")
    );
    Ok(())
}

#[test]
fn verified_endpoint_bridge_exports_only_case_checked_capabilities() -> TestResult {
    let dir = tempfile::tempdir()?;
    let passing = json!([{"params":{"Region":"us-west-2"},"expect":{"endpoint":{
        "url":"https://global.example","properties":{"authSchemes":[{
            "name":"sigv4","signingRegion":"us-east-1"}]}}}}]);
    write_service(dir.path(), "checked", global_rules(), passing)?;
    write_service(
        dir.path(),
        "failed",
        global_rules(),
        json!([{
            "params":{"Region":"us-west-2"},"expect":{"endpoint":{"url":"wrong"}}
        }]),
    )?;
    let capabilities = catalog_tool::endpoints::verified_endpoints(dir.path())?;
    assert_eq!(capabilities.len(), 1);
    assert!(matches!(
        capabilities
            .get("checked")
            .map(|endpoint| endpoint.endpoint()),
        Some(DerivedEndpoint::Resolved {
            class: ServiceClass::Global,
            ..
        })
    ));
    assert!(!capabilities.contains_key("failed"));
    Ok(())
}

#[test]
fn a_service_without_sigv4_is_explicitly_unsupported() -> TestResult {
    let parsed = RuleSet::parse(Path::new("bearer.json"), &global_rules())?;
    assert_eq!(
        parsed.derive(None)?,
        DerivedEndpoint::Unsupported {
            reason: "no SigV4 trait".into()
        }
    );
    Ok(())
}
