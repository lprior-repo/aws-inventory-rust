pub mod endpoints_support;
use catalog_tool::endpoints::{Error, Fault, MAX_STRING, Outcome, RuleSet};
use endpoints_support::{TestResult, call, endpoint, resolve, result, rules};
use serde_json::json;
use std::path::Path;

#[test]
fn assignments_are_visible_to_later_conditions_children_and_endpoint_templates() -> TestResult {
    // Given a partition assignment and escaped braces, when a tree matches, then values propagate.
    let document = rules(
        json!({"Region":{"type":"string"}}),
        json!([{
            "type":"tree","conditions":[
                {"fn":"aws.partition","argv":[{"ref":"Region"}],"assign":"p"},
                call("isSet", json!([{"ref":"p"}]))],
            "rules":[endpoint(json!("https://{{literal}}.{Region}.{p#dnsSuffix}"), json!([]))]
        }]),
    );
    assert_eq!(
        resolve(&document, json!({"Region":"us-west-2"}))?,
        result("https://{literal}.us-west-2.amazonaws.com")
    );
    Ok(())
}

#[test]
fn failed_rule_assignments_do_not_leak_to_siblings() -> TestResult {
    // Given a rule assigning a partition then failing, when its sibling runs, then p is absent.
    let document = rules(
        json!({}),
        json!([
            endpoint(
                json!("wrong"),
                json!([
            {"fn":"aws.partition","argv":["us-west-2"],"assign":"p"},
            call("booleanEquals", json!([false,true]))])
            ),
            endpoint(
                json!("right"),
                json!([call("not", json!([call("isSet", json!([{"ref":"p"}]))]))])
            )
        ]),
    );
    assert_eq!(resolve(&document, json!({}))?, result("right"));
    Ok(())
}

#[test]
fn the_first_matching_rule_wins_without_evaluating_later_rules() -> TestResult {
    // Given overlapping unconditional rules, when resolved, then the first URL is retained.
    let document = rules(
        json!({}),
        json!([
            endpoint(json!("first"), json!([])),
            endpoint(json!("second"), json!([]))
        ]),
    );
    assert_eq!(resolve(&document, json!({}))?, result("first"));
    Ok(())
}

#[test]
fn a_selected_tree_exhaustion_is_terminal_even_with_a_matching_outer_sibling() -> TestResult {
    // Given an empty successful tree branch, when exhausted, then fallback siblings are not tried.
    let document = rules(
        json!({}),
        json!([{"type":"tree","conditions":[],"rules":[
        endpoint(json!("wrong"),json!([call("not",json!([true]))]))]},
        endpoint(json!("not-a-fallback"),json!([]))]),
    );
    let parsed = RuleSet::parse(Path::new("tree.json"), &document)?;
    assert!(matches!(
        parsed.evaluate(&Default::default()),
        Err(Error::Model {
            kind: Fault::Exhausted,
            ..
        })
    ));
    Ok(())
}

#[test]
fn modeled_errors_expand_their_message_and_are_not_interpreter_failures() -> TestResult {
    // Given an error rule, when its conditions match, then its exact message is returned.
    let document = rules(
        json!({"Region":{"type":"string"}}),
        json!([
        {"type":"error","conditions":[],"error":"No endpoint for {Region}"}]),
    );
    assert_eq!(
        resolve(&document, json!({"Region":"us-west-2"}))?,
        Outcome::Error("No endpoint for us-west-2".into())
    );
    Ok(())
}

#[test]
fn required_defaults_are_applied_but_explicit_parameters_win() -> TestResult {
    // Given a required default, when absent or overridden, then binding is deterministic.
    let document = rules(
        json!({"Region":{"type":"string","required":true,"default":"us-east-1"}}),
        json!([endpoint(json!("{Region}"), json!([]))]),
    );
    assert_eq!(resolve(&document, json!({}))?, result("us-east-1"));
    assert_eq!(
        resolve(&document, json!({"Region":"eu-west-1"}))?,
        result("eu-west-1")
    );
    Ok(())
}

#[test]
fn absent_required_parameters_are_named_before_rule_evaluation() -> TestResult {
    // Given a required Bucket, when omitted, then a filename-bearing missing-input error is returned.
    let document = rules(
        json!({"Bucket":{"type":"string","required":true}}),
        json!([endpoint(json!("unreachable"), json!([]))]),
    );
    let parsed = RuleSet::parse(Path::new("required.json"), &document)?;
    assert!(
        matches!(parsed.evaluate(&Default::default()), Err(Error::Model {
        file, kind:Fault::MissingParameter(name) }) if file==Path::new("required.json") && name=="Bucket")
    );
    Ok(())
}

#[test]
fn expanded_templates_are_bounded_without_truncation() -> TestResult {
    // Given a repeated maximum-sized value, when expanded, then typed bounds reject the whole result.
    let document = rules(
        json!({"Region":{"type":"string"}}),
        json!([endpoint(json!("{Region}{Region}"), json!([]))]),
    );
    let parsed = RuleSet::parse(Path::new("expansion.json"), &document)?;
    let input = [("Region".into(), json!("x".repeat(MAX_STRING)))]
        .into_iter()
        .collect();
    assert!(matches!(
        parsed.evaluate(&input),
        Err(Error::Model {
            kind: Fault::Bound("expanded string bytes"),
            ..
        })
    ));
    Ok(())
}
