pub mod endpoints_support;

use catalog_tool::endpoints::{Error, Fault, MAX_STRING, Outcome, RuleSet};
use endpoints_support::{TestResult, endpoint, result, rules};
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::Path};

#[test]
fn assignment_cannot_shadow_a_declared_but_unset_parameter_even_in_an_untaken_rule() -> TestResult {
    let document = rules(
        json!({"p":{"type":"string"}}),
        json!([
            endpoint(json!("first"), json!([])),
            endpoint(
                json!("unreachable"),
                json!([
                    {"fn":"stringEquals","argv":["x","x"],"assign":"p"}
                ])
            )
        ]),
    );
    assert_collision(document, "p")
}

#[test]
fn assignment_cannot_reuse_a_name_in_the_same_or_parent_scope() -> TestResult {
    let assign = json!({"fn":"isSet","argv":[true],"assign":"p"});
    assert_collision(
        rules(
            json!({}),
            json!([endpoint(json!("unused"), json!([assign, assign]))]),
        ),
        "p",
    )?;
    assert_collision(
        rules(
            json!({}),
            json!([
                {"type":"tree","conditions":[assign],"rules":[
                    endpoint(json!("unused"),json!([assign]))]}
            ]),
        ),
        "p",
    )
}

#[test]
fn separate_sibling_rules_can_reuse_the_same_assignment_name() -> TestResult {
    let assign = json!({"fn":"isSet","argv":[true],"assign":"p"});
    let document = rules(
        json!({}),
        json!([
            endpoint(json!("first"), json!([assign])),
            endpoint(json!("second"), json!([assign]))
        ]),
    );
    let parsed = RuleSet::parse(Path::new("scope.json"), &document)?;
    assert_eq!(parsed.evaluate(&BTreeMap::new())?, result("first"));
    Ok(())
}

fn assert_collision(document: Value, name: &str) -> TestResult {
    assert!(matches!(RuleSet::parse(Path::new("scope.json"), &document),
        Err(Error::Model { file, kind: Fault::Malformed(message) })
            if file == Path::new("scope.json") && message == format!("duplicate assignment: {name}")));
    Ok(())
}

#[test]
fn property_keys_accept_16384_bytes_and_reject_16385_before_expansion() -> TestResult {
    let key = "k".repeat(MAX_STRING);
    let properties = serde_json::Map::from_iter([(key.clone(), json!("value"))]);
    let document = rules(
        json!({}),
        json!([{"type":"endpoint","conditions":[],
        "endpoint":{"url":"https://keys.test","properties":properties}}]),
    );
    let parsed = RuleSet::parse(Path::new("keys.json"), &document)?;
    let Outcome::Endpoint(endpoint) = parsed.evaluate(&BTreeMap::new())? else {
        return Err("expected endpoint".into());
    };
    assert_eq!(endpoint.properties.get(&key), Some(&json!("value")));
    let properties =
        serde_json::Map::from_iter([("k".repeat(MAX_STRING.saturating_add(1)), json!("v"))]);
    let document = rules(
        json!({}),
        json!([{"type":"endpoint","conditions":[],
        "endpoint":{"url":"https://keys.test","properties":properties}}]),
    );
    assert_string_bound(RuleSet::parse(Path::new("keys.json"), &document))
}

#[test]
fn parameter_and_supplied_binding_identifiers_obey_the_same_exact_byte_limit() -> TestResult {
    let key = "p".repeat(MAX_STRING);
    let parameters = serde_json::Map::from_iter([(key.clone(), json!({"type":"string"}))]);
    let document = rules(
        json!(parameters),
        json!([endpoint(json!({"ref":key}), json!([]))]),
    );
    let parsed = RuleSet::parse(Path::new("keys.json"), &document)?;
    let input = BTreeMap::from([(key, json!("https://identifier.test"))]);
    assert_eq!(parsed.evaluate(&input)?, result("https://identifier.test"));
    let over = "p".repeat(MAX_STRING.saturating_add(1));
    let input = BTreeMap::from([(over.clone(), json!("value"))]);
    assert_string_bound(parsed.evaluate(&input))?;
    let parameters = serde_json::Map::from_iter([(over, json!({"type":"string"}))]);
    let document = rules(
        json!(parameters),
        json!([endpoint(json!("unused"), json!([]))]),
    );
    assert_string_bound(RuleSet::parse(Path::new("keys.json"), &document))
}

fn assert_string_bound<T>(actual: Result<T, Error>) -> TestResult {
    assert!(
        matches!(actual, Err(Error::Model { file, kind: Fault::Bound("string bytes") })
        if file == Path::new("keys.json"))
    );
    Ok(())
}
