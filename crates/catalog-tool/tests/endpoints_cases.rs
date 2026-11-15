pub mod endpoints_support;

use std::path::Path;

use catalog_tool::endpoints::{Case, RuleSet, SkipReason, classify_case};
use endpoints_support::{TestResult, call, endpoint, rules};
use serde_json::{Value, json};

#[test]
fn excluded_cases_are_never_interpreted_and_have_one_reason() -> TestResult {
    let document = rules(
        json!({}),
        json!([endpoint(
            json!("https://unused.test"),
            json!([call("unknown", json!([]))]),
        )]),
    );
    let rules = RuleSet::parse(Path::new("scope.json"), &document)?;
    let cases = vec![
        case(json!({"Region":"cn-north-1","UseFIPS":true}), json!([]))?,
        case(json!({"UseFIPS":true,"UseDualStack":true}), json!([]))?,
        case(
            json!({"Endpoint":"https://custom.test","Bucket":"b"}),
            json!([]),
        )?,
        case(json!({"Region":"us-west-2"}), json!([{"Bucket":"b"}]))?,
    ];

    let counts = rules.run_cases(&cases)?;

    assert_eq!(
        (counts.in_scope(), counts.passed(), counts.failed()),
        (0, 0, 0)
    );
    assert_eq!(counts.skipped().len(), 4);
    assert!(counts.unsupported().is_empty());
    for reason in [
        SkipReason::OtherPartition,
        SkipReason::Fips,
        SkipReason::CustomEndpoint,
        SkipReason::OperationInputs,
    ] {
        assert_eq!(counts.skipped().get(&reason), Some(&1));
    }
    Ok(())
}

#[test]
fn reached_request_time_function_is_a_named_skipped_case() -> TestResult {
    let document = rules(
        json!({}),
        json!([endpoint(
            json!("https://unused.test"),
            json!([call("parseURL", json!(["https://x"]))]),
        )]),
    );
    let rules = RuleSet::parse(Path::new("request-time.json"), &document)?;
    let counts = rules.run_cases(&[case(json!({}), json!([]))?])?;
    assert_eq!(
        (counts.in_scope(), counts.passed(), counts.failed()),
        (0, 0, 0)
    );
    assert_eq!(
        counts.skipped().get(&SkipReason::RequestTimeFunction),
        Some(&1)
    );
    let Some(unsupported) = counts.unsupported().first() else {
        return Err("missing request-time diagnostic".into());
    };
    assert_eq!(
        (unsupported.index, unsupported.name.as_str()),
        (0, "parseURL")
    );
    Ok(())
}

#[test]
fn only_actual_commercial_region_and_false_options_are_baseline() -> TestResult {
    for params in [
        json!({}),
        json!({"Region":"us-west-2"}),
        json!({"Region":"eu-west-1","UseFIPS":false,"UseDualStack":false}),
    ] {
        assert_eq!(classify_case(&case(params, json!([]))?), None);
    }
    for params in [
        json!({"Region":false}),
        json!({"UseFIPS":"false"}),
        json!({"UseDualStack":null}),
        json!({"Bucket":"b"}),
    ] {
        assert_eq!(
            classify_case(&case(params, json!([]))?),
            Some(SkipReason::OtherParameters)
        );
    }
    Ok(())
}

fn case(params: Value, operation_inputs: Value) -> Result<Case, serde_json::Error> {
    serde_json::from_value(json!({"params":params,"operationInputs":operation_inputs,
        "expect":{"error":"unused"}}))
}
