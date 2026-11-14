pub mod endpoints_support;

use catalog_tool::endpoints::{Error, Fault, RuleSet};
use endpoints_support::{TestResult, assert_condition, call, endpoint, rules};
use serde_json::{Value, json};
use std::path::Path;

#[test]
fn all_seven_functions_reject_missing_and_excess_arguments_with_exact_faults() -> TestResult {
    for (name, arity) in [
        ("isSet", 1_usize),
        ("booleanEquals", 2),
        ("stringEquals", 2),
        ("not", 1),
        ("aws.partition", 1),
        ("getAttr", 2),
        ("isValidHostLabel", 2),
    ] {
        for count in [arity.saturating_sub(1), arity.saturating_add(1)] {
            let document = rules(
                json!({}),
                json!([endpoint(
                    json!("unused"),
                    json!([call(name, json!(vec![Value::Null; count]))])
                )]),
            );
            assert!(matches!(RuleSet::parse(Path::new("arity.json"), &document),
                Err(Error::Model { file, kind: Fault::Arguments(actual) })
                    if file == Path::new("arity.json") && actual == name));
        }
    }
    Ok(())
}

#[test]
fn every_constrained_argument_position_rejects_wrong_types_with_exact_faults() -> TestResult {
    // isSet has no type restriction; its accepted values are tested separately.
    for (name, label, args) in [
        (
            "booleanEquals",
            "BooleanEquals",
            vec![json!(true), json!(false)],
        ),
        ("stringEquals", "StringEquals", vec![json!("a"), json!("b")]),
        ("not", "Not", vec![json!(true)]),
        ("aws.partition", "Partition", vec![json!("us-west-2")]),
        ("getAttr", "GetAttr", vec![json!([]), json!("[0]")]),
        (
            "isValidHostLabel",
            "HostLabel",
            vec![json!("host"), json!(true)],
        ),
    ] {
        for position in 0..args.len() {
            let invalid: Vec<Value> = args
                .iter()
                .enumerate()
                .map(|(index, value)| {
                    if index == position {
                        Value::Null
                    } else {
                        value.clone()
                    }
                })
                .collect();
            let document = rules(
                json!({}),
                json!([endpoint(
                    json!("unused"),
                    json!([call(name, json!(invalid))])
                )]),
            );
            let parsed = RuleSet::parse(Path::new("types.json"), &document)?;
            assert!(matches!(parsed.evaluate(&Default::default()),
                Err(Error::Model { file, kind: Fault::Arguments(actual) })
                    if file == Path::new("types.json") && actual == label));
        }
    }
    Ok(())
}

#[test]
fn is_set_accepts_every_expression_value_type_without_coercion() -> TestResult {
    for (value, expected) in [
        (json!(null), false),
        (json!(false), true),
        (json!(0), true),
        (json!(""), true),
        (json!([]), true),
    ] {
        assert_condition(call("isSet", json!([value])), expected)?;
    }
    assert_condition(
        call(
            "isSet",
            json!([call("aws.partition", json!(["us-west-2"]))]),
        ),
        true,
    )
}
