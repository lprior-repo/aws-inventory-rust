use super::*;

#[derive(Default)]
pub(super) struct FakeTransport {
    pub(super) seen: BTreeMap<(String, String), usize>,
}

impl FakeTransport {
    pub(super) fn respond(
        &mut self,
        plan: &CallablePlan<'_>,
        wire: &http::Request<bytes::Bytes>,
    ) -> TestResult<Json> {
        let native: Json = serde_json::from_slice(wire.body())?;
        let operation = plan.operation_name();
        let (items, selected, resource) = resource_shape(operation)?;
        let root = ROOTS.iter().any(|(name, _)| *name == operation);
        let scope = request_scope(&native, root)?;
        assert_endpoint(wire, scope);
        assert_child(&native, scope, root, resource);
        let count = self
            .seen
            .entry((operation.to_owned(), scope.to_owned()))
            .or_default();
        *count = count.saturating_add(1);
        let row = response_row(scope, resource);
        Ok(if root {
            json!({items:[row]})
        } else {
            json!({selected:row})
        })
    }
}

fn request_scope(native: &Json, root: bool) -> TestResult<&str> {
    if root {
        return native
            .get("Scope")
            .and_then(Json::as_str)
            .ok_or("root Scope missing".into());
    }
    if let Some(name) = native.get("Name").and_then(Json::as_str) {
        return Ok(name);
    }
    let resource = native
        .get("ResourceArn")
        .and_then(Json::as_str)
        .ok_or("child owner missing")?;
    Ok(if resource.contains(":global/") {
        "CLOUDFRONT"
    } else {
        "REGIONAL"
    })
}

fn assert_endpoint(wire: &http::Request<bytes::Bytes>, scope: &str) {
    assert_eq!(wire.method(), http::Method::POST);
    let host = match scope {
        "REGIONAL" => "wafv2.eu-west-1.amazonaws.com",
        "CLOUDFRONT" => "wafv2.us-east-1.amazonaws.com",
        _ => "unreviewed-scope",
    };
    assert_eq!(wire.uri().host(), Some(host));
}

fn assert_child(native: &Json, scope: &str, root: bool, resource: &str) {
    if root {
        assert!(matches!(scope, "REGIONAL" | "CLOUDFRONT"));
    } else if resource == "logging" {
        assert_eq!(
            native.get("ResourceArn"),
            Some(&json!(arn(scope, "webacl")))
        );
        assert_eq!(native.get("Scope"), None);
    } else {
        assert_eq!(
            native.get("Scope"),
            Some(&json!(scope)),
            "wrong-scope child request"
        );
        assert_eq!(
            native.get("Id"),
            Some(&json!("11111111-1111-4111-8111-111111111111"))
        );
        assert_eq!(native.get("Name"), Some(&json!(scope)));
    }
}

fn resource_shape(operation: &str) -> TestResult<(&'static str, &'static str, &'static str)> {
    Ok(match operation {
        "ListWebACLs" | "GetWebACL" => ("WebACLs", "WebACL", "webacl"),
        "ListIPSets" | "GetIPSet" => ("IPSets", "IPSet", "ipset"),
        "ListRegexPatternSets" | "GetRegexPatternSet" => {
            ("RegexPatternSets", "RegexPatternSet", "regexpatternset")
        }
        "ListRuleGroups" | "GetRuleGroup" => ("RuleGroups", "RuleGroup", "rulegroup"),
        "ListManagedRuleSets" | "GetManagedRuleSet" => {
            ("ManagedRuleSets", "ManagedRuleSet", "managedruleset")
        }
        "ListLoggingConfigurations" | "GetLoggingConfiguration" => {
            ("LoggingConfigurations", "LoggingConfiguration", "logging")
        }
        _ => return Err(format!("unexpected WAFv2 request: {operation}").into()),
    })
}

fn response_row(scope: &str, resource: &str) -> Json {
    if resource == "logging" {
        return json!({"ResourceArn":arn(scope,"webacl"),"LogDestinationConfigs":[]});
    }
    json!({
        "Name":scope,"Id":"11111111-1111-4111-8111-111111111111","ARN":arn(scope,resource),
        "DefaultAction":{"Allow":{}},"Rules":[],"Capacity":1,
        "VisibilityConfig":{"SampledRequestsEnabled":false,"CloudWatchMetricsEnabled":false,"MetricName":"inventory"},
        "IPAddressVersion":"IPV4","Addresses":[],"RegularExpressionList":[]
    })
}

pub(super) fn arn(scope: &str, resource: &str) -> String {
    let (region, segment) = if scope == "CLOUDFRONT" {
        ("us-east-1", "global")
    } else {
        ("eu-west-1", "regional")
    };
    format!(
        "arn:aws:wafv2:{region}:123456789012:{segment}/{resource}/{scope}/11111111-1111-4111-8111-111111111111"
    )
}
