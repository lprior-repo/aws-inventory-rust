use catalog_tool::{endpoints, kind_map, names, overlay, runtime_catalog, smithy};
use serde_json::json;
use std::{collections::BTreeMap, fs, path::Path};

pub type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const MODEL: &str = r#"{"smithy":"2.0","shapes":{
"n#Service":{"type":"service","version":"1","operations":[{"target":"n#ListThings"}],"traits":{"aws.api#service":{"sdkId":"Things","endpointPrefix":"things","arnNamespace":"things"},"aws.auth#sigv4":{"name":"things"},"aws.protocols#restJson1":{}}},
"n#ListThings":{"type":"operation","input":{"target":"n#Input"},"output":{"target":"n#Output"},"traits":{"smithy.api#http":{"method":"GET","uri":"/things"}}},
"n#Input":{"type":"structure","members":{"Mode":{"target":"n#Choice","traits":{"smithy.api#required":{},"smithy.api#httpQuery":"mode"}},"Modes":{"target":"n#Choices"}}},
"n#Output":{"type":"structure","members":{"Items":{"target":"n#Items"},"Mode":{"target":"n#Choice"}}},
"n#Items":{"type":"list","member":{"target":"ITEM_TARGET"}},
"n#Thing":{"type":"structure","members":{"Name":{"target":"n#Choice"},"OtherName":{"target":"smithy.api#String"},"Count":{"target":"smithy.api#Long"},"Flag":{"target":"smithy.api#Boolean"}}},
"n#Choice":{"type":"enum","members":{"ACTIVE":{"target":"smithy.api#Unit","traits":{"smithy.api#enumValue":"active-wire"}},"SECOND":{"target":"smithy.api#Unit","traits":{"smithy.api#enumValue":"second-wire"}}}},
"n#Choices":{"type":"list","member":{"target":"n#Choice"}}
}}"#;

pub fn key(path: &str) -> Result<overlay::Identity, overlay::NameError> {
    Ok(overlay::Identity::Key(vec![overlay::MemberPath::selected(
        path,
    )?]))
}

pub fn arn(path: &str) -> Result<overlay::Identity, overlay::NameError> {
    Ok(overlay::Identity::Arn(overlay::MemberPath::selected(path)?))
}

pub fn compiled(identity: overlay::Identity, fixed: &str, scalar: bool) -> TestResult<Vec<u8>> {
    compile_fixture(identity, fixed, None, scalar, false, None)
}

pub fn compiled_singleton(identity: overlay::Identity) -> TestResult<Vec<u8>> {
    compile_fixture(identity, "active-wire", None, false, true, None)
}

pub fn compiled_fixed(
    identity: overlay::Identity,
    mode: &str,
    modes: &[&str],
    scalar: bool,
) -> TestResult<Vec<u8>> {
    compile_fixture(identity, mode, Some(modes), scalar, false, None)
}

pub fn compiled_self_binding(source: overlay::ContextSource) -> TestResult<Vec<u8>> {
    compile_fixture(
        key("Name")?,
        "active-wire",
        None,
        false,
        false,
        Some(source),
    )
}

fn compile_fixture(
    identity: overlay::Identity,
    mode: &str,
    modes: Option<&[&str]>,
    scalar: bool,
    singleton: bool,
    self_binding: Option<overlay::ContextSource>,
) -> TestResult<Vec<u8>> {
    let root = tempfile::tempdir()?;
    let source = root.path().join("source");
    let model_dir = source.join("models/things/service/2020-01-01");
    fs::create_dir_all(&model_dir)?;
    let target = if scalar { "n#Choice" } else { "n#Thing" };
    let model = MODEL.replace("ITEM_TARGET", target);
    let model = if singleton {
        model.replace(
            "\"Items\":{\"target\":\"n#Items\"}",
            "\"Items\":{\"target\":\"n#Thing\"}",
        )
    } else {
        model
    };
    fs::write(model_dir.join("model.json"), model)?;
    let mut plan = overlay::OperationOverlay::default();
    plan.class = Some(overlay::OverlayClass::Enumerate);
    plan.scope = Some(overlay::Scope::Region);
    plan.items = Some(overlay::ItemSelector::parse("Items")?);
    plan.paginator = Some(overlay::Paginator::None);
    plan.identity = Some(identity);
    let mode_path = overlay::MemberPath::new("Mode")?;
    if let Some(source) = self_binding {
        plan.bind.insert(
            mode_path,
            overlay::Binding::Context {
                operation: overlay::OperationName::new("ListThings")?,
                source,
                path: overlay::MemberPath::new("Mode")?,
            },
        );
    } else {
        plan.fixed
            .insert(mode_path, overlay::FixedValue::String(mode.to_owned()));
    }
    if let Some(modes) = modes {
        plan.fixed.insert(
            overlay::MemberPath::new("Modes")?,
            overlay::FixedValue::Strings(modes.iter().map(|mode| (*mode).to_owned()).collect()),
        );
    }
    let policies = [overlay::ServiceOverlay::new(
        "things",
        vec![("ListThings", plan)],
    )?];
    let checked = overlay::check_policies(
        &source,
        &policies,
        &catalog_tool::classes::DenyList::builtin(),
    )?;
    let fields = smithy::Registry::parse("Count\nFlag\nItems\nMode\nModes\nName\nOtherName\n")?;
    let base = smithy::build_catalog(&source, &fields, "identity-fixture")?;
    let kinds = kind_map::MappingSource {
        api_only: &[kind_map::ApiOnlyMapping {
            kind: "AWS::ApiOnly::Identity",
            reason: "Identity fixture",
        }],
        operations: &[kind_map::OperationMapping {
            service: "things",
            operation: "ListThings",
            kind: "AWS::ApiOnly::Identity",
        }],
        ..kind_map::MappingSource::default()
    }
    .compile(&names::Exceptions::none())?;
    let endpoints = BTreeMap::from([("things".to_owned(), endpoint()?)]);
    Ok(runtime_catalog::augment(
        &base, &checked, &kinds, &endpoints,
    )?)
}

fn endpoint() -> TestResult<endpoints::VerifiedEndpoint> {
    let properties = json!({"authSchemes":[{"name":"sigv4","signingName":"things","signingRegion":"us-east-1"}]});
    let document = json!({"version":"1.0","parameters":{"Region":{"type":"string"}},"rules":[{"type":"endpoint","conditions":[],"endpoint":{"url":"https://things.test","properties":properties}}]});
    let rules = endpoints::RuleSet::parse(Path::new("identity-endpoint.json"), &document)?;
    let cases: Vec<endpoints::Case> = serde_json::from_value(
        json!([{"params":{"Region":"us-east-1"},"expect":{"endpoint":{"url":"https://things.test","properties":properties}}}]),
    )?;
    rules
        .run_cases(&cases)?
        .verified_endpoint(Some("things"))
        .map_err(|failure| format!("fixture endpoint evidence failed: {failure:?}").into())
}
