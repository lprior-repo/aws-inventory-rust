pub mod endpoints_support;
use catalog_tool::endpoints::{Error, Fault, MAX_DEPTH, MAX_STEPS, Outcome, RuleSet};
use endpoints_support::{TestResult, call, endpoint, resolve, result, rules};
use serde_json::json;
use std::path::Path;

#[test]
fn unsupported_functions_are_lazy_typed_results_only_on_taken_branches() -> TestResult {
    [
        "aws.parseArn",
        "aws.isVirtualHostableS3Bucket",
        "uriEncode",
        "substring",
        "parseURL",
        "unknown.fn",
    ]
    .into_iter()
    .try_for_each(|name| {
        let unreachable = rules(
            json!({}),
            json!([
                endpoint(
                    json!("wrong"),
                    json!([call("not", json!([true])), call(name, json!([]))])
                ),
                endpoint(json!("right"), json!([]))
            ]),
        );
        assert_eq!(resolve(&unreachable, json!({}))?, result("right"));

        let reached = rules(
            json!({}),
            json!([endpoint(json!("wrong"), json!([call(name, json!([]))]))]),
        );
        assert_eq!(
            resolve(&reached, json!({}))?,
            Outcome::UnsupportedFunction { name: name.into() }
        );
        Ok(())
    })
}

#[test]
fn missing_rules_and_unknown_rule_types_return_filename_bearing_schema_errors() -> TestResult {
    [
        json!({"version":"1.0","parameters":{}}),
        rules(json!({}), json!([{"type":"surprise","conditions":[]}])),
        rules(json!({}), json!([])),
    ]
    .into_iter()
    .try_for_each(|document| {
        let parsed = RuleSet::parse(Path::new("bad-schema.json"), &document);
        assert!(
            matches!(parsed,Err(Error::Model {file,kind:Fault::Malformed(_)})
                if file==Path::new("bad-schema.json"))
        );
        Ok(())
    })
}

#[test]
fn wrong_function_argument_counts_are_rejected_during_parsing() -> TestResult {
    [
        ("isSet", json!([])),
        ("not", json!([true, false])),
        ("stringEquals", json!(["a"])),
        ("getAttr", json!([])),
        ("aws.partition", json!([])),
        ("isValidHostLabel", json!(["a"])),
    ]
    .into_iter()
    .try_for_each(|(name, args)| {
        let document = rules(
            json!({}),
            json!([endpoint(json!("unused"), json!([call(name, args)]))]),
        );
        assert!(matches!(RuleSet::parse(Path::new("args.json"),&document),
                Err(Error::Model {kind:Fault::Arguments(actual),..}) if actual==name));
        Ok(())
    })
}

#[test]
fn wrong_function_argument_types_return_typed_evaluation_errors() -> TestResult {
    [
        ("not", json!(["false"])),
        ("booleanEquals", json!([true, "true"])),
        ("stringEquals", json!([1, "1"])),
        ("getAttr", json!([false, "a"])),
        ("aws.partition", json!([false])),
        ("isValidHostLabel", json!(["host", "true"])),
    ]
    .into_iter()
    .try_for_each(|(name, args)| {
        let document = rules(
            json!({}),
            json!([endpoint(json!("unused"), json!([call(name, args)]))]),
        );
        let parsed = RuleSet::parse(Path::new("types.json"), &document)?;
        assert!(matches!(
            parsed.evaluate(&Default::default()),
            Err(Error::Model {
                kind: Fault::Arguments(_),
                ..
            })
        ));
        Ok(())
    })
}

#[test]
fn nesting_and_node_count_limits_return_typed_bounds() -> TestResult {
    let nested = (0..MAX_DEPTH).fold(
        endpoint(json!("deep"), json!([])),
        |inner, _| json!({"type":"tree","conditions":[],"rules":[inner]}),
    );
    assert!(matches!(
        RuleSet::parse(Path::new("depth.json"), &rules(json!({}), json!([nested]))),
        Err(Error::Model {
            kind: Fault::Bound("depth"),
            ..
        })
    ));
    let many = rules(
        json!({}),
        json!([endpoint(
            json!("many"),
            json!([call("isSet", json!([vec![false; MAX_STEPS]]))])
        )]),
    );
    assert!(matches!(
        RuleSet::parse(Path::new("nodes.json"), &many),
        Err(Error::Model {
            kind: Fault::Bound("steps"),
            ..
        })
    ));
    Ok(())
}

#[test]
fn invalid_templates_and_non_string_substitutions_are_typed_errors() -> TestResult {
    [
        "{",
        "}",
        "{}",
        "{missing}",
        "{p#supportsFIPS}",
        "{{{unclosed",
    ]
    .into_iter()
    .try_for_each(|url| {
        let document = rules(
            json!({}),
            json!([endpoint(
                json!(url),
                json!([
                {"fn":"aws.partition","argv":["us-west-2"],"assign":"p"}])
            )]),
        );
        let parsed = RuleSet::parse(Path::new("template.json"), &document)?;
        assert!(matches!(
            parsed.evaluate(&Default::default()),
            Err(Error::Model {
                kind: Fault::Template(_),
                ..
            })
        ));
        Ok(())
    })
}
