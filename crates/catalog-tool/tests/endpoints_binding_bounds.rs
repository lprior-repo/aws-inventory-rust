pub mod endpoints_support;

use catalog_tool::endpoints::{Error, Fault, MAX_STEPS, MAX_STRING, RuleSet};
use endpoints_support::{TestResult, endpoint, result, rules};
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::Path};

#[test]
fn input_binding_nodes_exactly_100000_succeed_and_100001_fail() -> TestResult {
    let document = rules(
        json!({"Items":{"type":"stringArray"}}),
        json!([endpoint(json!("https://bounds.test"), json!([]))]),
    );
    let parsed = RuleSet::parse(Path::new("bindings.json"), &document)?;
    let mut values = vec![json!(""); MAX_STEPS.saturating_sub(1)];
    let input = BTreeMap::from([("Items".into(), json!(values))]);
    assert_eq!(parsed.evaluate(&input)?, result("https://bounds.test"));
    values.push(json!(""));
    let input = BTreeMap::from([("Items".into(), json!(values))]);
    assert!(matches!(parsed.evaluate(&input),
        Err(Error::Model { file, kind: Fault::Bound("steps") })
            if file == Path::new("bindings.json")));
    Ok(())
}

#[test]
fn supplied_string_exactly_16384_bytes_succeeds_and_16385_fails() -> TestResult {
    let document = rules(
        json!({"Text":{"type":"string"}}),
        json!([endpoint(json!({"ref":"Text"}), json!([]))]),
    );
    let parsed = RuleSet::parse(Path::new("strings.json"), &document)?;
    let text = "x".repeat(MAX_STRING);
    let input = BTreeMap::from([("Text".into(), json!(text))]);
    assert_eq!(parsed.evaluate(&input)?, result(&text));
    let input = BTreeMap::from([(
        "Text".into(),
        json!("x".repeat(MAX_STRING.saturating_add(1))),
    )]);
    assert!(matches!(parsed.evaluate(&input),
        Err(Error::Model { file, kind: Fault::Bound("string bytes") })
            if file == Path::new("strings.json")));
    Ok(())
}

#[test]
fn expanded_ascii_and_unicode_strings_accept_exact_byte_limit_not_one_more() -> TestResult {
    let unicode_count = MAX_STRING
        .checked_div(2)
        .ok_or("zero divisor")?
        .saturating_sub(1);
    for (prefix, text) in [
        ("p", "x".repeat(MAX_STRING.saturating_sub(1))),
        ("é", "é".repeat(unicode_count)),
    ] {
        let document = rules(
            json!({"Text":{"type":"string"}}),
            json!([endpoint(json!(format!("{prefix}{{Text}}")), json!([]))]),
        );
        let parsed = RuleSet::parse(Path::new("expanded.json"), &document)?;
        let input = BTreeMap::from([("Text".into(), json!(text))]);
        let expected = format!("{prefix}{text}");
        assert_eq!(expected.len(), MAX_STRING);
        assert_eq!(parsed.evaluate(&input)?, result(&expected));
        let mut over = text;
        over.push('a');
        let input = BTreeMap::from([("Text".into(), json!(over))]);
        assert!(matches!(parsed.evaluate(&input),
            Err(Error::Model { file, kind: Fault::Bound("expanded string bytes") })
                if file == Path::new("expanded.json")));
    }
    Ok(())
}

#[test]
fn assignment_accepts_binding_256_but_rejects_257_and_duplicate_names() -> TestResult {
    let accepted = assignment_rules(255, "Assigned");
    let parsed = RuleSet::parse(Path::new("assignments.json"), &accepted)?;
    assert_eq!(
        parsed.evaluate(&BTreeMap::new())?,
        result("https://bounds.test")
    );
    let document = assignment_rules(256, "Assigned");
    let parsed = RuleSet::parse(Path::new("assignments.json"), &document)?;
    assert!(matches!(parsed.evaluate(&BTreeMap::new()),
        Err(Error::Model { file, kind: Fault::Malformed(message) })
            if file == Path::new("assignments.json")
                && message == "duplicate or excessive assignment: Assigned"));
    let duplicate = assignment_rules(255, "P0");
    assert!(
        matches!(RuleSet::parse(Path::new("assignments.json"), &duplicate),
        Err(Error::Model { file, kind: Fault::Malformed(message) })
            if file == Path::new("assignments.json") && message == "duplicate assignment: P0")
    );
    Ok(())
}

fn assignment_rules(count: usize, name: &str) -> Value {
    let parameters: serde_json::Map<String, Value> = (0..count)
        .map(|index| {
            (
                format!("P{index}"),
                json!({"type":"boolean","required":true,"default":true}),
            )
        })
        .collect();
    rules(
        json!(parameters),
        json!([endpoint(
            json!("https://bounds.test"),
            json!([
                {"fn":"isSet","argv":[true],"assign":name}
            ])
        )]),
    )
}
