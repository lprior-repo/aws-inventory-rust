use super::{call, rules};
use catalog_tool::endpoints::{Case, DerivedService, RuleSet};
use serde_json::{Value, json};
use std::path::Path;

#[derive(Debug)]
pub struct Fixture {
    pub name: &'static str,
    pub document: Value,
    pub cases: Value,
}

pub fn fixtures() -> Vec<Fixture> {
    vec![
        Fixture {
            name: "health",
            document: routing("health", "aws-global", "https://global.health.test"),
            cases: json!([
                case("aws-global", "https://global.health.test", "us-east-1"),
                case("us-west-2", "https://health.us-west-2.test", "us-west-2"),
                case("eu-west-1", "https://health.eu-west-1.test", "eu-west-1"),
                case("aws-global", "https://global.health.test", "us-east-1")
            ]),
        },
        Fixture {
            name: "simpledbv2",
            document: routing("sdb", "us-east-1", "https://sdb.test"),
            cases: json!([case("us-east-1", "https://sdb.test", "us-east-1")]),
        },
        Fixture {
            name: "sts",
            document: routing("sts", "aws-global", "https://sts.test"),
            cases: json!([
                case("us-west-2", "https://sts.us-west-2.test", "us-west-2"),
                case("aws-global", "https://sts.test", "us-east-1"),
                case("eu-west-1", "https://sts.eu-west-1.test", "eu-west-1")
            ]),
        },
        Fixture {
            name: "west-fixed",
            document: rules(
                json!({"Region":{"type":"string"}}),
                json!([route("https://fixed.{Region}.test", json!([]), "us-west-2")]),
            ),
            cases: json!([
                case("us-west-2", "https://fixed.us-west-2.test", "us-west-2"),
                case("eu-west-1", "https://fixed.eu-west-1.test", "us-west-2")
            ]),
        },
    ]
}

pub fn routing(prefix: &str, special: &str, special_url: &str) -> Value {
    rules(
        json!({"Region":{"type":"string"}}),
        json!([
            route(
                special_url,
                json!([call("stringEquals", json!([{"ref":"Region"},special]))]),
                "us-east-1"
            ),
            route(
                &format!("https://{prefix}.{{Region}}.test"),
                json!([]),
                "{Region}"
            )
        ]),
    )
}

pub fn route(url: &str, conditions: Value, signing_region: &str) -> Value {
    json!({"type":"endpoint","conditions":conditions,"endpoint":{"url":url,
        "properties":{"authSchemes":[{"name":"sigv4","signingRegion":signing_region}]}}})
}

pub fn case(region: &str, url: &str, signing_region: &str) -> Value {
    json!({"params":{"Region":region},"expect":{"endpoint":{"url":url,
        "properties":{"authSchemes":[{"name":"sigv4","signingRegion":signing_region}]}}}})
}

pub fn verified(
    document: &Value,
    cases: Value,
    name: &str,
) -> Result<DerivedService, Box<dyn std::error::Error>> {
    let parsed = RuleSet::parse(Path::new("evidence.json"), document)?;
    let cases: Vec<Case> = serde_json::from_value(cases)?;
    let counts = parsed.run_cases(&cases)?;
    assert_eq!(
        (counts.in_scope(), counts.passed(), counts.failed()),
        (cases.len(), cases.len(), 0)
    );
    Ok(counts.derive(Some(name))?)
}

pub const REPORT: &str = concat!(
    "cases 10\nin_scope 10\npassed 10\nfailed 0\nskipped 0\n",
    "skipped other_partition 0\nskipped fips 0\nskipped dual_stack 0\n",
    "skipped custom_endpoint 0\nskipped other_parameters 0\nskipped operation_inputs 0\n",
    "skipped request_time_function 0\nservices 4\nglobal 0\nregional 3\n",
    "needs_overlay 1\nunsupported 0\nservices_with_exceptions 3\nexceptions 3\n",
    "service health regional https://health.{region}.test health-sign {region}\n",
    "exception health aws-global https://global.health.test health-sign us-east-1\n",
    "service simpledbv2 needs-overlay https://sdb.{region}.test simpledbv2-sign {region} ",
    "reason:\"regional template not confirmed by any published case\"\n",
    "exception simpledbv2 us-east-1 https://sdb.test simpledbv2-sign us-east-1\n",
    "service sts regional https://sts.{region}.test sts-sign {region}\n",
    "exception sts aws-global https://sts.test sts-sign us-east-1\n",
    "service west-fixed regional https://fixed.{region}.test west-fixed-sign us-west-2\n"
);
