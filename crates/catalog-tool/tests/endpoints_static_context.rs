use catalog_tool::endpoints::{DerivedEndpoint, ServiceClass, verified_endpoints};
use serde_json::{Map, Value, json};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn endpoint(agent: bool, region: &str) -> Value {
    let (host, name, signing_region) = if agent {
        ("research", "agent-signer", "us-east-1")
    } else {
        ("control", "service-signing", region)
    };
    json!({"url":format!("https://{host}.{region}.example.test"),
        "properties":{"authSchemes":[{"name":"sigv4","signingName":name,
            "signingRegion":signing_region}]}})
}

fn rules() -> Value {
    let agent = endpoint(true, "{Region}");
    let control = endpoint(false, "{Region}");
    json!({"version":"1.0","parameters":{
        "Region":{"type":"string"},
        "UseFIPS":{"type":"boolean","required":true,"default":false},
        "UseDualStack":{"type":"boolean","required":true,"default":false},
        "SubServiceType":{"type":"string"}},
        "rules":[
            {"type":"endpoint","conditions":[
                {"fn":"isSet","argv":[{"ref":"SubServiceType"}]},
                {"fn":"stringEquals","argv":[{"ref":"SubServiceType"},"AGENT"]}],
                "endpoint":agent},
            {"type":"endpoint","conditions":[],"endpoint":control}]})
}

fn cases() -> Value {
    json!([
        {"params":{"Region":"us-west-2"},"expect":{"endpoint":endpoint(false,"us-west-2")}},
        {"params":{"Region":"eu-west-1"},"expect":{"endpoint":endpoint(false,"eu-west-1")}},
        {"params":{"Region":"us-west-2","SubServiceType":"AGENT"},
            "expect":{"endpoint":endpoint(true,"us-west-2")}},
        {"params":{"Region":"us-east-1","SubServiceType":"AGENT"},
            "expect":{"endpoint":endpoint(true,"us-east-1")}}
    ])
}

fn fixture(rules: Value, cases: Value, operations: Value) -> Result<tempfile::TempDir, Box<dyn std::error::Error>> {
    let mut shapes = Map::new();
    for (name, context) in operations.as_object().ok_or("expected operation map")? {
        shapes.insert(format!("example#{name}"), json!({"type":"operation","traits":{
            "smithy.rules#staticContextParams":context}}));
    }
    model(rules, cases, shapes)
}

fn model(rules: Value, cases: Value, mut shapes: Map<String, Value>) -> Result<tempfile::TempDir, Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir()?;
    let directory = dir.path().join("models/mixed/service/2026-01-01");
    std::fs::create_dir_all(&directory)?;
    shapes.insert("example#Service".into(), json!({"type":"service","traits":{
        "aws.auth#sigv4":{"name":"service-signing"},
        "smithy.rules#endpointRuleSet":rules,
        "smithy.rules#endpointTests":{"testCases":cases}}}));
    std::fs::write(directory.join("model.json"), serde_json::to_vec(&json!({
        "smithy":"2.0","shapes":shapes
    }))?)?;
    Ok(dir)
}

fn agent() -> Value { json!({"SubServiceType":{"value":"AGENT"}}) }

fn resolved(host: &str, name: &str, signing_region: &str) -> DerivedEndpoint {
    DerivedEndpoint::Resolved {
        class: ServiceClass::Regional,
        url_template: format!("https://{host}.{{region}}.example.test"),
        signing_name: name.into(),
        signing_region: signing_region.into(),
    }
}

#[test]
fn modeled_operation_constants_select_their_route_and_independent_signing_context() -> TestResult {
    // Given a service with two modeled routes and distinct SigV4 overrides,
    let dir = fixture(rules(), cases(), json!({"ListProfiles":agent(),"ListGoals":agent()}))?;
    // When endpoint evidence is compiled for the individual operations,
    let endpoints = verified_endpoints(dir.path())?;
    let endpoint = endpoints.get("mixed").ok_or("missing service evidence")?;
    let agent_route = resolved("research", "agent-signer", "us-east-1");
    // Then both static-context operations use the modeled agent route/signature,
    // while the ordinary service operation remains regional and unchanged.
    assert_eq!(endpoint.operation_endpoint("ListProfiles"), Some(&agent_route));
    assert_eq!(endpoint.operation_endpoint("ListGoals"), Some(&agent_route));
    let control = resolved("control", "service-signing", "{region}");
    assert_eq!(endpoint.endpoint(), &control);
    assert_eq!(endpoint.operation_endpoint("ListOrdinary"), Some(&control));
    Ok(())
}

#[test]
fn unknown_mistyped_reserved_or_unevidenced_constants_never_use_the_service_route() -> TestResult {
    let dir = fixture(rules(), cases(), json!({
        "Unknown":{"NotAParameter":{"value":"AGENT"}},
        "WrongType":{"SubServiceType":{"value":true}},
        "Reserved":{"Region":{"value":"us-east-1"}},
        "Unobserved":{"SubServiceType":{"value":"OTHER"}},
        "Verified":{"SubServiceType":{"value":"AGENT"}}
    }))?;
    let endpoints = verified_endpoints(dir.path())?;
    let endpoint = endpoints.get("mixed").ok_or("missing service evidence")?;
    for operation in ["Unknown", "WrongType", "Reserved", "Unobserved"] {
        assert_eq!(endpoint.operation_endpoint(operation), None, "{operation}");
    }
    assert!(endpoint.operation_endpoint("Verified").is_some());
    assert!(endpoint.operation_endpoint("ListOrdinary").is_some());
    Ok(())
}

#[test]
fn failed_agent_endpoint_evidence_does_not_invalidate_or_leak_the_control_route() -> TestResult {
    let mut evidence = cases();
    let agent_case = evidence.as_array_mut().and_then(|cases| cases.get_mut(2))
        .ok_or("missing agent case")?;
    let expected = agent_case.get_mut("expect").and_then(|expect| expect.get_mut("endpoint"))
        .and_then(Value::as_object_mut).ok_or("missing expected endpoint")?;
    expected.insert("url".into(), json!("https://wrong.example.test"));
    let dir = fixture(rules(), evidence, json!({"ListProfiles":agent()}))?;
    let endpoints = verified_endpoints(dir.path())?;
    let endpoint = endpoints.get("mixed").ok_or("missing control service evidence")?;
    assert_eq!(endpoint.operation_endpoint("ListProfiles"), None);
    assert_eq!(endpoint.operation_endpoint("ListOrdinary"), Some(endpoint.endpoint()));
    Ok(())
}

#[test]
fn builtin_parameters_cannot_be_overridden_by_operation_static_context() -> TestResult {
    let mut document = rules();
    document.get_mut("parameters").and_then(|parameters| parameters.get_mut("SubServiceType"))
        .and_then(Value::as_object_mut).ok_or("missing parameter")?
        .insert("builtIn".into(), json!("SDK::Example"));
    let dir = fixture(document, cases(), json!({"ListProfiles":agent()}))?;
    let endpoints = verified_endpoints(dir.path())?;
    let endpoint = endpoints.get("mixed").ok_or("missing control service evidence")?;
    assert_eq!(endpoint.operation_endpoint("ListProfiles"), None);
    assert!(endpoint.operation_endpoint("ListOrdinary").is_some());
    Ok(())
}

#[test]
fn omitted_case_parameters_use_only_the_original_modeled_default_as_evidence() -> TestResult {
    let mut document = rules();
    document.get_mut("parameters").and_then(|parameters| parameters.get_mut("SubServiceType"))
        .and_then(Value::as_object_mut).ok_or("missing parameter")?
        .extend([("required".into(), json!(true)), ("default".into(), json!("AGENT"))]);
    let evidence = json!([
        {"params":{"Region":"us-west-2"},"expect":{"endpoint":endpoint(true,"us-west-2")}},
        {"params":{"Region":"us-east-1"},"expect":{"endpoint":endpoint(true,"us-east-1")}}
    ]);
    let dir = fixture(document, evidence, json!({
        "ListProfiles":agent(),"Unevidenced":{"SubServiceType":{"value":"OTHER"}}
    }))?;
    let endpoints = verified_endpoints(dir.path())?;
    let endpoint = endpoints.get("mixed").ok_or("missing service evidence")?;
    assert_eq!(endpoint.operation_endpoint("ListProfiles"), Some(&resolved("research", "agent-signer", "us-east-1")));
    assert_eq!(endpoint.operation_endpoint("Unevidenced"), None);
    Ok(())
}

#[test]
fn inputs_of_the_operation_itself_are_route_evidence_and_another_operations_are_not() -> TestResult {
    // A case's parameters are its complete rule inputs, so the request members
    // its own operation lists add nothing. Another operation's requests are not
    // requests the compiled operation makes, even when the constants match.
    let own = json!([{"operationName":"ListProfiles","operationParams":{"Profile":"p"}}]);
    let other = json!([{"operationName":"ListGoals"}]);
    for (inputs, expected) in [
        ([own, other.clone()], Some(resolved("research", "agent-signer", "us-east-1"))),
        ([other.clone(), other], None),
    ] {
        let mut evidence = cases();
        for (case, inputs) in evidence.as_array_mut().ok_or("expected cases")?.iter_mut().skip(2).zip(inputs) {
            case.as_object_mut().ok_or("expected case")?
                .insert("operationInputs".into(), inputs);
        }
        let dir = fixture(rules(), evidence, json!({"ListProfiles":agent()}))?;
        let endpoints = verified_endpoints(dir.path())?;
        let endpoint = endpoints.get("mixed").ok_or("missing control service evidence")?;
        assert_eq!(endpoint.operation_endpoint("ListProfiles"), expected.as_ref());
        assert!(endpoint.operation_endpoint("ListOrdinary").is_some());
    }
    Ok(())
}

#[test]
fn boolean_and_string_array_constants_use_the_modeled_parameter_types() -> TestResult {
    let mut document = rules();
    document.get_mut("parameters").and_then(Value::as_object_mut)
        .ok_or("missing parameters")?.extend([
            ("Enabled".into(), json!({"type":"boolean"})),
            ("Labels".into(), json!({"type":"stringArray"})),
        ]);
    let mut evidence = cases();
    for case in evidence.as_array_mut().ok_or("expected cases")?.iter_mut().skip(2) {
        case.get_mut("params").and_then(Value::as_object_mut).ok_or("missing case parameters")?
            .extend([("Enabled".into(), json!(true)), ("Labels".into(), json!(["one", "two"]))]);
    }
    let dir = fixture(document, evidence, json!({
        "Typed":{"SubServiceType":{"value":"AGENT"},"Enabled":{"value":true},
            "Labels":{"value":["one","two"]}},
        "WrongLabels":{"SubServiceType":{"value":"AGENT"},"Enabled":{"value":true},
            "Labels":{"value":["one",2]}}
    }))?;
    let endpoints = verified_endpoints(dir.path())?;
    let endpoint = endpoints.get("mixed").ok_or("missing service evidence")?;
    assert_eq!(endpoint.operation_endpoint("Typed"), Some(&resolved("research", "agent-signer", "us-east-1")));
    assert_eq!(endpoint.operation_endpoint("WrongLabels"), None);
    Ok(())
}

#[test]
fn only_accountless_non_required_cases_are_evidence_for_an_operation_route() -> TestResult {
    // Given a route whose only modeled cases also pin the account-endpoint mode,
    let mut document = rules();
    document.get_mut("parameters").and_then(Value::as_object_mut).ok_or("missing parameters")?
        .insert("AccountIdEndpointMode".into(),
            json!({"type":"string","builtIn":"AWS::Auth::AccountIdEndpointMode"}));
    let evidence = |mode: &str, account: Option<&str>| -> Result<Value, Box<dyn std::error::Error>> {
        let mut evidence = cases();
        for case in evidence.as_array_mut().ok_or("expected cases")?.iter_mut().skip(2) {
            let params = case.get_mut("params").and_then(Value::as_object_mut)
                .ok_or("missing case parameters")?;
            params.insert("AccountIdEndpointMode".into(), json!(mode));
            if let Some(account) = account {
                params.insert("AccountId".into(), json!(account));
            }
        }
        Ok(evidence)
    };
    let route = |evidence: Value| -> Result<Option<DerivedEndpoint>, Box<dyn std::error::Error>> {
        let dir = fixture(document.clone(), evidence, json!({"ListProfiles":agent()}))?;
        let endpoints = verified_endpoints(dir.path())?;
        let endpoint = endpoints.get("mixed").ok_or("missing service evidence")?;
        Ok(endpoint.operation_endpoint("ListProfiles").cloned())
    };
    // When the case describes the runtime, which holds no account: `disabled`,
    // or the SDK-default `preferred` that falls back to the regional route,
    // Then the cases verify the operation route; `required` or a supplied
    // account describes requests the runtime never sends and proves nothing.
    assert_eq!(route(evidence("disabled", None)?)?, Some(resolved("research", "agent-signer", "us-east-1")));
    assert_eq!(route(evidence("preferred", None)?)?, Some(resolved("research", "agent-signer", "us-east-1")));
    assert_eq!(route(evidence("required", None)?)?, None);
    assert_eq!(route(evidence("disabled", Some("123456789012"))?)?, None);
    assert_eq!(route(evidence("preferred", Some("123456789012"))?)?, None);
    Ok(())
}

#[test]
fn operation_constants_route_a_service_whose_own_route_cannot_resolve() -> TestResult {
    // Given a rule set whose required selector has no default, so only an
    // operation's modeled constant can complete it,
    let mut document = rules();
    document.get_mut("parameters").and_then(|parameters| parameters.get_mut("SubServiceType"))
        .and_then(Value::as_object_mut).ok_or("missing parameter")?
        .insert("required".into(), json!(true));
    let mut evidence = cases();
    for case in evidence.as_array_mut().ok_or("expected cases")?.iter_mut().take(2) {
        case.get_mut("params").and_then(Value::as_object_mut).ok_or("missing case parameters")?
            .insert("SubServiceType".into(), json!("CONTROL"));
    }
    let control = json!({"SubServiceType":{"value":"CONTROL"}});
    let dir = fixture(document.clone(), evidence.clone(),
        json!({"ListProfiles":agent(),"ListControl":control}))?;
    // When endpoint evidence is compiled,
    let endpoints = verified_endpoints(dir.path())?;
    let endpoint = endpoints.get("mixed").ok_or("missing operation routes")?;
    // Then each operation uses the route its own constant selects, and an
    // operation without one inherits nothing callable from the service.
    assert_eq!(endpoint.operation_endpoint("ListProfiles"),
        Some(&resolved("research", "agent-signer", "us-east-1")));
    assert_eq!(endpoint.operation_endpoint("ListControl"),
        Some(&resolved("control", "service-signing", "{region}")));
    assert!(!matches!(endpoint.operation_endpoint("ListOrdinary"),
        Some(DerivedEndpoint::Resolved { .. })));
    // And a service with no verified operation route is not published at all.
    let dir = fixture(document, evidence, json!({"Unobserved":{"SubServiceType":{"value":"OTHER"}}}))?;
    assert!(!verified_endpoints(dir.path())?.contains_key("mixed"));
    Ok(())
}

fn express_rules() -> Value {
    json!({"version":"1.0","parameters":{
        "Region":{"type":"string"},
        "UseFIPS":{"type":"boolean","required":true,"default":false},
        "UseDualStack":{"type":"boolean","required":true,"default":false},
        "Accelerate":{"type":"boolean","required":true,"default":false},
        "Bucket":{"type":"string"},
        "UseControl":{"type":"boolean"},
        "DisableSessionAuth":{"type":"boolean"}},
        "rules":[
            {"type":"endpoint","conditions":[
                {"fn":"isSet","argv":[{"ref":"UseControl"}]},
                {"fn":"booleanEquals","argv":[{"ref":"UseControl"},true]},
                {"fn":"isSet","argv":[{"ref":"DisableSessionAuth"}]}],
                "endpoint":endpoint(true,"{Region}")},
            {"type":"endpoint","conditions":[
                {"fn":"isSet","argv":[{"ref":"UseControl"}]},
                {"fn":"booleanEquals","argv":[{"ref":"UseControl"},true]}],
                "endpoint":endpoint(true,"{Region}")},
            {"type":"endpoint","conditions":[],"endpoint":endpoint(false,"{Region}")}]})
}

fn express_case(region: &str, inputs: Value, extra: Value) -> Value {
    let mut params = json!({"Region":region,"UseFIPS":false,"UseDualStack":false,
        "Accelerate":false,"UseControl":true});
    if let (Some(params), Some(extra)) = (params.as_object_mut(), extra.as_object()) {
        params.extend(extra.clone());
    }
    json!({"params":params,"operationInputs":inputs,
        "expect":{"endpoint":endpoint(true,region)}})
}

fn express_route(cases: Value, rules: Value) -> Result<Option<DerivedEndpoint>, Box<dyn std::error::Error>> {
    let mut all = json!([
        {"params":{"Region":"us-west-2"},"expect":{"endpoint":endpoint(false,"us-west-2")}}
    ]);
    all.as_array_mut().ok_or("expected cases")?
        .extend(cases.as_array().ok_or("expected cases")?.iter().cloned());
    let dir = fixture(rules, all, json!({"ListDirectories":{"UseControl":{"value":true}}}))?;
    let endpoints = verified_endpoints(dir.path())?;
    let endpoint = endpoints.get("mixed").ok_or("missing service evidence")?;
    Ok(endpoint.operation_endpoint("ListDirectories").cloned())
}

#[test]
fn a_published_case_for_the_operation_itself_with_default_and_inert_flags_is_route_evidence() -> TestResult {
    // Given the only control-plane evidence names the operation and spells out
    // a default and an inert optional flag, whether it lists client built-ins
    // or request members that bind no endpoint parameter,
    let own = json!([{"operationName":"ListDirectories","builtInParams":{"AWS::Region":"us-east-1"}}]);
    let bound = json!([{"operationName":"ListDirectories","operationParams":{"Bucket":"b"}}]);
    for inputs in [own, bound] {
        let cases = json!([express_case("us-east-1", inputs, json!({"DisableSessionAuth":false}))]);
        // When the operation route is compiled, Then that case verifies it.
        assert_eq!(express_route(cases, express_rules())?,
            Some(resolved("research", "agent-signer", "us-east-1")));
    }
    Ok(())
}

#[test]
fn cases_for_other_operations_request_values_or_meaningful_flags_are_not_route_evidence() -> TestResult {
    let own = json!([{"operationName":"ListDirectories","builtInParams":{"AWS::Region":"us-east-1"}}]);
    let other = json!([{"operationName":"CreateDirectory","builtInParams":{"AWS::Region":"us-east-1"}}]);
    for (name, case) in [
        ("another operation", express_case("us-east-1", other, json!({}))),
        ("enabled flag", express_case("us-east-1", own.clone(), json!({"DisableSessionAuth":true}))),
        ("request value", express_case("us-east-1", own.clone(), json!({"Bucket":"b"}))),
    ] {
        assert_eq!(express_route(json!([case]), express_rules())?, None, "{name}");
    }
    // A false flag the rules do distinguish from the unset flag stays a parameter.
    let mut rules = express_rules();
    let flagged = rules.get_mut("rules").and_then(Value::as_array_mut)
        .and_then(|rules| rules.first_mut()).ok_or("missing flagged rule")?;
    flagged.as_object_mut().ok_or("expected rule")?
        .insert("endpoint".into(), endpoint(false, "{Region}"));
    let mut case = express_case("us-east-1", own, json!({"DisableSessionAuth":false}));
    case.as_object_mut().ok_or("expected case")?
        .insert("expect".into(), json!({"endpoint":endpoint(false,"us-east-1")}));
    assert_eq!(express_route(json!([case]), rules)?, None, "distinguished flag");
    Ok(())
}

fn label_rules(account_host: &str) -> Value {
    let enabled = |name: &str| json!([
        {"fn":"isSet","argv":[{"ref":name}]},
        {"fn":"booleanEquals","argv":[{"ref":name},true]}]);
    let mut account = enabled("RequiresAccountId");
    if let Some(conditions) = account.as_array_mut() {
        conditions.push(json!({"fn":"isSet","argv":[{"ref":"AccountId"}]}));
    }
    let mut account_endpoint = endpoint(false, "{Region}");
    account_endpoint["url"] = json!(format!("https://{account_host}.{{Region}}.example.test"));
    json!({"version":"1.0","parameters":{
        "Region":{"type":"string"},
        "UseFIPS":{"type":"boolean","required":true,"default":false},
        "UseDualStack":{"type":"boolean","required":true,"default":false},
        "AccountId":{"type":"string"},
        "RequiresAccountId":{"type":"boolean"},
        "UseExpress":{"type":"boolean"}},
        "rules":[
            {"type":"error","conditions":[
                {"fn":"isSet","argv":[{"ref":"AccountId"}]},
                {"fn":"not","argv":[{"fn":"isValidHostLabel","argv":[{"ref":"AccountId"},false]}]}],
                "error":"invalid account"},
            {"type":"endpoint","conditions":enabled("UseExpress"),"endpoint":endpoint(true,"{Region}")},
            {"type":"endpoint","conditions":account,"endpoint":account_endpoint},
            {"type":"error","conditions":enabled("RequiresAccountId"),"error":"account required"},
            {"type":"endpoint","conditions":[],"endpoint":endpoint(false,"{Region}")}]})
}

fn account_case(region: &str, account: Option<&str>, flag: &str, expect: Value) -> Value {
    let mut params = json!({"Region":region,"UseFIPS":false,flag:true});
    if let Some(account) = account {
        params["AccountId"] = json!(account);
    }
    json!({"params":params,"expect":expect})
}

fn account_endpoint(host: &str, region: &str) -> Value {
    let mut expect = endpoint(false, region);
    expect["url"] = json!(format!("https://{host}.{region}.example.test"));
    json!({"endpoint":expect})
}

/// Compiles `ListJobs`, whose required `AccountId` member is a host label bound
/// to the endpoint parameter of the same name, under the given constant flag.
fn label_route(
    prefix: Option<&str>,
    flag: &str,
    rules: Value,
    cases: Vec<Value>,
) -> Result<Option<DerivedEndpoint>, Box<dyn std::error::Error>> {
    let mut all = vec![
        json!({"params":{"Region":"us-west-2"},"expect":{"endpoint":endpoint(false,"us-west-2")}}),
    ];
    all.extend(cases);
    let mut traits = json!({"smithy.rules#staticContextParams":{flag:{"value":true}}});
    if let Some(prefix) = prefix {
        traits["smithy.api#endpoint"] = json!({"hostPrefix":prefix});
    }
    let mut shapes = Map::new();
    shapes.insert("example#ListJobs".into(), json!({"type":"operation",
        "input":{"target":"example#ListJobsRequest"},"traits":traits}));
    shapes.insert("example#ListJobsRequest".into(), json!({"type":"structure","members":{
        "AccountId":{"target":"smithy.api#String","traits":{
            "smithy.api#hostLabel":{},"smithy.api#required":{},
            "smithy.rules#contextParam":{"name":"AccountId"}}},
        "MaxResults":{"target":"smithy.api#Integer"}}}));
    let dir = model(rules, Value::Array(all), shapes)?;
    let endpoints = verified_endpoints(dir.path())?;
    let endpoint = endpoints.get("mixed").ok_or("missing service evidence")?;
    Ok(endpoint.operation_endpoint("ListJobs").cloned())
}

#[test]
fn a_request_host_label_is_left_for_the_modeled_host_prefix_to_supply() -> TestResult {
    // Given every published case for the operation carries some account as the
    // leading host label, and the operation's host prefix is that label,
    let prefixed = ["us-west-2", "eu-west-1"].map(|region| account_case(
        region, Some("123456789012"), "RequiresAccountId",
        account_endpoint("123456789012.control", region)));
    // When its route is compiled, Then the route is the host the prefix extends.
    assert_eq!(
        label_route(Some("{AccountId}."), "RequiresAccountId", label_rules("{AccountId}.control"), prefixed.to_vec())?,
        Some(resolved("control", "service-signing", "{region}")));
    // Given rules that route the operation without using its label, and no host
    // prefix, Then the published route stands as it is.
    let ignored = account_case("us-east-1", Some("123456789012"), "UseExpress",
        json!({"endpoint":endpoint(true,"us-east-1")}));
    assert_eq!(
        label_route(None, "UseExpress", label_rules("{AccountId}.control"), vec![ignored])?,
        Some(resolved("research", "agent-signer", "us-east-1")));
    Ok(())
}

#[test]
fn a_host_label_the_runtime_would_misplace_or_never_send_yields_no_route() -> TestResult {
    let leading = || label_rules("{AccountId}.control");
    let account = |host: &str| account_case("us-west-2", Some("123456789012"),
        "RequiresAccountId", account_endpoint(host, "us-west-2"));
    let express = account_case("us-east-1", Some("123456789012"), "UseExpress",
        json!({"endpoint":endpoint(true,"us-east-1")}));
    for (name, prefix, flag, rules, case) in [
        ("label inside another host label", Some("{AccountId}."), "RequiresAccountId",
            label_rules("control-{AccountId}"), account("control-123456789012")),
        ("rules omit the modeled prefix", Some("{AccountId}."), "UseExpress", leading(), express),
        ("rules add a label no prefix supplies", None, "RequiresAccountId",
            leading(), account("123456789012.control")),
        ("prefix decorates the label", Some("{AccountId}-jobs."), "RequiresAccountId",
            leading(), account("123456789012.control")),
        ("case omits the required label", Some("{AccountId}."), "RequiresAccountId", leading(),
            account_case("us-west-2", None, "RequiresAccountId", json!({"error":"account required"}))),
        ("error about one label value", Some("{AccountId}."), "RequiresAccountId", leading(),
            account_case("us-west-2", Some("bad_account"), "RequiresAccountId", json!({"error":"invalid account"}))),
    ] {
        assert_eq!(label_route(prefix, flag, rules, vec![case])?, None, "{name}");
    }
    Ok(())
}
