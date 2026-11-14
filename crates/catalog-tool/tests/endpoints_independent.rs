pub mod endpoints_support;

use catalog_tool::endpoints::{Case, RuleSet};
use endpoints_support::{TestResult, call, endpoint, resolve, result, rules};
use serde_json::{Value, json};
use std::path::Path;

#[test]
fn boolean_equality_false_uses_the_fallback_without_an_evaluator_oracle() -> TestResult {
    let document = rules(
        json!({}),
        json!([
            endpoint(
                json!("wrong"),
                json!([call("booleanEquals", json!([true, false]))])
            ),
            endpoint(json!("https://fallback.test"), json!([]))
        ]),
    );
    assert_eq!(
        resolve(&document, json!({}))?,
        result("https://fallback.test")
    );
    Ok(())
}

#[test]
fn first_match_does_not_evaluate_later_unsupported_rules() -> TestResult {
    let document = rules(
        json!({}),
        json!([
            endpoint(json!("https://first.test"), json!([])),
            endpoint(json!("wrong"), json!([call("unknown", json!([]))]))
        ]),
    );
    assert_eq!(resolve(&document, json!({}))?, result("https://first.test"));
    Ok(())
}

#[test]
fn case_comparison_protects_url_properties_headers_and_error_text_independently() -> TestResult {
    let expected = json!({"endpoint":{"url":"https://actual.test",
        "properties":{"authSchemes":[{"name":"sigv4","signingName":"actual"}]},
        "headers":{"x-test":["actual"]}}});
    let document = rules(
        json!({"Region":{"type":"string"}}),
        json!([
            {"type":"error","conditions":[call("stringEquals",json!([{"ref":"Region"},"us-east-1"]))],
                "error":"actual error"},
            {"type":"endpoint","conditions":[],"endpoint":expected.get("endpoint")}
        ]),
    );
    let parsed = RuleSet::parse(Path::new("comparison.json"), &document)?;
    let mismatches = [
        json!({"endpoint":{"url":"wrong","properties":{
            "authSchemes":[{"name":"sigv4","signingName":"actual"}]},
            "headers":{"x-test":["actual"]}}}),
        json!({"endpoint":{"url":"https://actual.test",
            "properties":{"authSchemes":[{"name":"sigv4","signingName":"wrong"}]},
            "headers":{"x-test":["actual"]}}}),
        json!({"endpoint":{"url":"https://actual.test",
            "properties":{"authSchemes":[{"name":"sigv4","signingName":"actual"}]},
            "headers":{"x-test":["wrong"]}}}),
    ];
    for mismatch in mismatches {
        let counts = parsed.run_cases(&comparison_cases(expected.clone(), mismatch, false)?)?;
        assert_eq!((counts.in_scope, counts.passed, counts.failed), (2, 1, 1));
    }
    let counts = parsed.run_cases(&comparison_cases(
        json!({"error":"actual error"}),
        json!({"error":"wrong error"}),
        true,
    )?)?;
    assert_eq!((counts.in_scope, counts.passed, counts.failed), (2, 1, 1));
    Ok(())
}

fn comparison_cases(
    correct: Value,
    wrong: Value,
    fail: bool,
) -> Result<Vec<Case>, serde_json::Error> {
    // Use the baseline Region parameter as the branch selector; no scope-excluded parameters.
    let params = if fail {
        json!({"Region":"us-east-1"})
    } else {
        json!({"Region":"us-west-2"})
    };
    serde_json::from_value(json!([
        {"params":params,"expect":correct}, {"params":params,"expect":wrong}
    ]))
}

#[test]
fn function_valued_url_and_unicode_templates_preserve_exact_bytes() -> TestResult {
    let function_url = rules(
        json!({}),
        json!([endpoint(
            call("getAttr", json!([["https://expected.test"], "[0]"])),
            json!([])
        )]),
    );
    assert_eq!(
        resolve(&function_url, json!({}))?,
        result("https://expected.test")
    );
    let unicode = rules(
        json!({"Text":{"type":"string"}}),
        json!([endpoint(json!("préfixe-{Text}-東京"), json!([]))]),
    );
    assert_eq!(
        resolve(&unicode, json!({"Text":"雪☃"}))?,
        result("préfixe-雪☃-東京")
    );
    Ok(())
}
