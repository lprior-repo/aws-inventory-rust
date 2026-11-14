pub mod exceptions;
pub mod report;

use catalog_tool::endpoints::{Endpoint, Outcome, RuleSet};
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::Path};

pub type TestResult = Result<(), Box<dyn std::error::Error>>;

pub fn rules(parameters: Value, rules: Value) -> Value {
    json!({"version":"1.0", "parameters":parameters, "rules":rules})
}

pub fn endpoint(url: Value, conditions: Value) -> Value {
    json!({"type":"endpoint", "conditions":conditions, "endpoint":{"url":url}})
}

pub fn resolve(document: &Value, params: Value) -> Result<Outcome, Box<dyn std::error::Error>> {
    let rules = RuleSet::parse(Path::new("hand-written.json"), document)?;
    let params: BTreeMap<String, Value> = serde_json::from_value(params)?;
    Ok(rules.evaluate(&params)?)
}

pub fn result(url: &str) -> Outcome {
    Outcome::Endpoint(Endpoint {
        url: url.into(),
        properties: json!({}),
        headers: json!({}),
    })
}

pub fn call(name: &str, args: Value) -> Value {
    json!({"fn":name,"argv":args})
}

pub fn assert_value(expression: Value, expected: Value) -> TestResult {
    match expected {
        Value::Bool(value) => assert_condition(expression, value),
        Value::Null => assert_condition(call("isSet", json!([expression])), false),
        Value::String(value) => {
            let mut condition = expression;
            condition
                .as_object_mut()
                .ok_or("expected a function expression")?
                .insert("assign".into(), json!("Actual"));
            let document = rules(
                json!({}),
                json!([endpoint(json!("{Actual}"), json!([condition]))]),
            );
            assert_eq!(resolve(&document, json!({}))?, result(&value));
            Ok(())
        }
        _ => Err("unsupported assertion value".into()),
    }
}

pub fn assert_condition(condition: Value, expected: bool) -> TestResult {
    let document = rules(
        json!({}),
        json!([
            endpoint(json!("https://true.test"), json!([condition])),
            endpoint(json!("https://false.test"), json!([]))
        ]),
    );
    let url = if expected {
        "https://true.test"
    } else {
        "https://false.test"
    };
    assert_eq!(resolve(&document, json!({}))?, result(url));
    Ok(())
}
