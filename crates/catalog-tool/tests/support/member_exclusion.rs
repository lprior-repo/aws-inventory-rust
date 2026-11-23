use catalog_tool::{
    classes::DenyList, endpoints, kind_map, names, overlay, runtime_catalog, smithy,
};
use serde_json::json;
use std::{collections::BTreeMap, fs, path::Path};
pub type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const MODEL: &str = r#"{"smithy":"2.0","shapes":{
"n#Service":{"type":"service","version":"1","operations":[{"target":"n#ListThings"}],"traits":{"aws.api#service":{"sdkId":"Things","endpointPrefix":"things","arnNamespace":"things"},"aws.auth#sigv4":{"name":"things"},"aws.protocols#restJson1":{}}},
"n#ListThings":{"type":"operation","output":{"target":"n#Output"},"traits":{"smithy.api#http":{"method":"GET","uri":"/things"}}},
"n#Output":{"type":"structure","members":{"Items":{"target":"n#Items"}}},
"n#Items":{"type":"list","member":{"target":"n#Thing"}},
"n#Thing":{"type":"structure","members":{"Id":{"target":"smithy.api#String"},"Secret":{"target":"smithy.api#String"},"Attributes":{"target":"n#Attributes"}}},
"n#Attributes":{"type":"map","key":{"target":"smithy.api#String"},"value":{"target":"smithy.api#String"}}
}}"#;

fn policy(paths: &[&str]) -> TestResult<overlay::ServiceOverlay> {
    let mut plan = overlay::OperationOverlay::default();
    plan.class = Some(overlay::OverlayClass::Enumerate);
    plan.scope = Some(overlay::Scope::Region);
    plan.items = Some(overlay::ItemSelector::parse("Items")?);
    plan.identity = Some(overlay::Identity::Key(vec![overlay::MemberPath::new(
        "Id",
    )?]));
    plan.paginator = Some(overlay::Paginator::None);
    plan.exclude_response_members = paths
        .iter()
        .map(|path| overlay::MemberPath::new(*path))
        .collect::<Result<_, _>>()?;
    Ok(overlay::ServiceOverlay::new(
        "things",
        vec![("ListThings", plan)],
    )?)
}

fn source() -> TestResult<tempfile::TempDir> {
    let root = tempfile::tempdir()?;
    let directory = root.path().join("models/things/service/2020-01-01");
    fs::create_dir_all(&directory)?;
    fs::write(directory.join("model.json"), MODEL)?;
    Ok(root)
}

pub fn checked(paths: &[&str]) -> TestResult<overlay::CheckedOverlays> {
    let source = source()?;
    Ok(overlay::check_policies(
        source.path(),
        &[policy(paths)?],
        &DenyList::builtin(),
    )?)
}

pub fn compiled(paths: &[&str]) -> TestResult<Vec<u8>> {
    let source = source()?;
    let checked = overlay::check_policies(source.path(), &[policy(paths)?], &DenyList::builtin())?;
    let base = smithy::build_catalog(
        source.path(),
        &smithy::Registry::parse("Attributes\nId\nItems\nSecret\n")?,
        "member-exclusion-fixture",
    )?;
    Ok(runtime_catalog::augment(
        &base,
        &checked,
        &kinds()?,
        &BTreeMap::from([("things".to_owned(), endpoint()?)]),
    )?)
}

fn kinds() -> TestResult<kind_map::KindMap> {
    Ok(kind_map::MappingSource {
        api_only: &[kind_map::ApiOnlyMapping {
            kind: "AWS::ApiOnly::MemberExclusion",
            reason: "Modeled member omission fixture",
        }],
        operations: &[kind_map::OperationMapping {
            service: "things",
            operation: "ListThings",
            kind: "AWS::ApiOnly::MemberExclusion",
        }],
        ..kind_map::MappingSource::default()
    }
    .compile(&names::Exceptions::none())?)
}

fn endpoint() -> TestResult<endpoints::VerifiedEndpoint> {
    let properties = json!({"authSchemes":[{"name":"sigv4","signingName":"things","signingRegion":"us-east-1"}]});
    let document = json!({"version":"1.0","parameters":{"Region":{"type":"string"}},"rules":[{"type":"endpoint","conditions":[],"endpoint":{"url":"https://things.test","properties":properties}}]});
    let rules = endpoints::RuleSet::parse(Path::new("member-exclusion-endpoint.json"), &document)?;
    let cases = serde_json::from_value::<Vec<endpoints::Case>>(
        json!([{"params":{"Region":"us-east-1"},"expect":{"endpoint":{"url":"https://things.test","properties":properties}}}]),
    )?;
    rules
        .run_cases(&cases)?
        .verified_endpoint(Some("things"))
        .map_err(|error| format!("fixture endpoint evidence failed: {error:?}").into())
}
