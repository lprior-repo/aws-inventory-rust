pub mod endpoints_support;

use catalog_tool::endpoints::{Error, Fault, MAX_DEPTH, MAX_STEPS, MAX_STRING, RuleSet};
use endpoints_support::{TestResult, endpoint, rules};
use serde_json::{Value, json};
use std::path::Path;

#[test]
fn document_depth_exactly_64_succeeds_and_65_fails() -> TestResult {
    let metadata = (1..MAX_DEPTH).fold(Value::Null, |inner, _| Value::Array(vec![inner]));
    assert!(parse(metadata.clone()).is_ok());
    assert!(matches!(parse(Value::Array(vec![metadata])),
        Err(Error::Model { file, kind: Fault::Bound("depth") })
            if file == Path::new("bounds.json")));
    Ok(())
}

#[test]
fn document_nodes_exactly_100000_succeed_and_100001_fail() -> TestResult {
    let count = MAX_STEPS.saturating_sub(10);
    assert!(parse(json!(vec![Value::Null; count])).is_ok());
    assert!(
        matches!(parse(json!(vec![Value::Null; count.saturating_add(1)])),
        Err(Error::Model { file, kind: Fault::Bound("steps") })
            if file == Path::new("bounds.json"))
    );
    Ok(())
}

#[test]
fn document_string_exactly_16384_bytes_succeeds_and_16385_fails() -> TestResult {
    assert!(parse(json!("x".repeat(MAX_STRING))).is_ok());
    assert!(
        matches!(parse(json!("x".repeat(MAX_STRING.saturating_add(1)))),
        Err(Error::Model { file, kind: Fault::Bound("string bytes") })
            if file == Path::new("bounds.json"))
    );
    Ok(())
}

fn parse(metadata: Value) -> Result<RuleSet, Error> {
    let document = json!({"version":"1.0","parameters":{},"metadata":metadata,
        "rules":[endpoint(json!("https://bounds.test"), json!([]))]});
    RuleSet::parse(Path::new("bounds.json"), &document)
}

#[test]
fn parameter_count_exactly_256_succeeds_and_257_fails() -> TestResult {
    let parameters: serde_json::Map<String, Value> = (0..256)
        .map(|index| (format!("P{index}"), json!({"type":"string"})))
        .collect();
    let document = rules(
        json!(parameters),
        json!([endpoint(json!("https://bounds.test"), json!([]))]),
    );
    assert!(RuleSet::parse(Path::new("parameters.json"), &document).is_ok());
    let mut excess = parameters;
    excess.insert("Excess".into(), json!({"type":"string"}));
    let document = rules(
        json!(excess),
        json!([endpoint(json!("https://bounds.test"), json!([]))]),
    );
    assert!(
        matches!(RuleSet::parse(Path::new("parameters.json"), &document),
        Err(Error::Model { file, kind: Fault::Bound("parameters") })
            if file == Path::new("parameters.json"))
    );
    Ok(())
}
