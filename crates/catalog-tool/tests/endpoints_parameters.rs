pub mod endpoints_support;

use catalog_tool::endpoints::{Error, Fault, RuleSet};
use endpoints_support::{TestResult, call, endpoint, resolve, result, rules};
use serde_json::json;
use std::{collections::BTreeMap, path::Path};

#[test]
fn capitalized_string_and_boolean_types_parse_and_preserve_values() -> TestResult {
    let document = rules(
        json!({"Text":{"type":"String"},"Flag":{"type":"Boolean"}}),
        json!([
            endpoint(
                json!("{Text}"),
                json!([call("booleanEquals", json!([{"ref":"Flag"},true]))])
            ),
            endpoint(json!("https://false.test"), json!([]))
        ]),
    );
    assert_eq!(
        resolve(
            &document,
            json!({"Text":"https://capital.test","Flag":true})
        )?,
        result("https://capital.test")
    );
    assert_eq!(
        resolve(&document, json!({"Text":"unused","Flag":false}))?,
        result("https://false.test")
    );
    Ok(())
}

#[test]
fn supplied_parameter_types_are_checked_even_when_the_parameter_is_unused() -> TestResult {
    for (kind, value) in [
        ("string", json!(false)),
        ("boolean", json!("true")),
        ("stringArray", json!(["valid", false])),
        ("stringArray", json!("not an array")),
    ] {
        let document = rules(
            json!({"Param":{"type":kind}}),
            json!([endpoint(json!("https://unused.test"), json!([]))]),
        );
        let parsed = RuleSet::parse(Path::new("parameters.json"), &document)?;
        let supplied: BTreeMap<_, _> = serde_json::from_value(json!({"Param":value}))?;
        assert!(matches!(parsed.evaluate(&supplied),
            Err(Error::Model { file, kind: Fault::Arguments(name) })
                if file == Path::new("parameters.json") && name == "Param"));
    }
    Ok(())
}

#[test]
fn invalid_defaults_are_rejected_with_the_exact_parameter_name() -> TestResult {
    for parameter in [
        json!({"type":"string","required":true,"default":false}),
        json!({"type":"boolean","required":true,"default":"false"}),
        json!({"type":"stringArray","required":true,"default":[1]}),
        json!({"type":"string","default":"optional defaults are invalid"}),
    ] {
        let document = rules(
            json!({"Param":parameter}),
            json!([endpoint(json!("https://unused.test"), json!([]))]),
        );
        assert!(
            matches!(RuleSet::parse(Path::new("defaults.json"), &document),
            Err(Error::Model { file, kind: Fault::Malformed(message) })
                if file == Path::new("defaults.json") && message == "invalid default for Param")
        );
    }
    Ok(())
}

#[test]
fn unknown_supplied_parameter_is_a_filename_bearing_argument_error() -> TestResult {
    let document = rules(
        json!({}),
        json!([endpoint(json!("https://unused.test"), json!([]))]),
    );
    let parsed = RuleSet::parse(Path::new("unknown.json"), &document)?;
    let supplied = serde_json::from_value(json!({"Unknown":"value"}))?;
    assert!(matches!(parsed.evaluate(&supplied),
        Err(Error::Model { file, kind: Fault::Arguments(message) })
            if file == Path::new("unknown.json") && message == "unknown parameter"));
    Ok(())
}
