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
    let dir = tempfile::tempdir()?;
    let directory = dir.path().join("models/mixed/service/2026-01-01");
    std::fs::create_dir_all(&directory)?;
    let mut shapes = Map::new();
    shapes.insert("example#Service".into(), json!({"type":"service","traits":{
        "aws.auth#sigv4":{"name":"service-signing"},
        "smithy.rules#endpointRuleSet":rules,
        "smithy.rules#endpointTests":{"testCases":cases}}}));
    for (name, context) in operations.as_object().ok_or("expected operation map")? {
        shapes.insert(format!("example#{name}"), json!({"type":"operation","traits":{
            "smithy.rules#staticContextParams":context}}));
    }
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
fn operation_input_cases_do_not_fabricate_static_route_evidence() -> TestResult {
    let mut evidence = cases();
    for case in evidence.as_array_mut().ok_or("expected cases")?.iter_mut().skip(2) {
        case.as_object_mut().ok_or("expected case")?
            .insert("operationInputs".into(), json!([{"operationName":"ListProfiles"}]));
    }
    let dir = fixture(rules(), evidence, json!({"ListProfiles":agent()}))?;
    let endpoints = verified_endpoints(dir.path())?;
    let endpoint = endpoints.get("mixed").ok_or("missing control service evidence")?;
    assert_eq!(endpoint.operation_endpoint("ListProfiles"), None);
    assert!(endpoint.operation_endpoint("ListOrdinary").is_some());
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
