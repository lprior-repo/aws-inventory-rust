use catalog_tool::{endpoints, kind_map, names, overlay, runtime_catalog, smithy};
use serde_json::json;
use std::{collections::BTreeMap, path::Path};

type FixtureResult<T> = Result<T, Box<dyn std::error::Error>>;

const MODEL: &str = r#"{"smithy":"2.0","shapes":{
"n.page#Service":{"type":"service","version":"1","operations":[{"target":"n.page#ListThings"}],"traits":{"aws.api#service":{"sdkId":"Page","endpointPrefix":"page","arnNamespace":"page"},"aws.auth#sigv4":{"name":"page"},"PROTOCOL":{}}},
"n.page#ListThings":{"type":"operation","input":{"target":"n.page#Input"},"output":{"target":"n.page#Output"},"traits":{"smithy.api#readonly":{},"smithy.api#http":{"method":"METHOD","uri":"/things"}}},
"n.page#Input":{"type":"structure","members":{}},
"n.page#Output":{"type":"structure","members":{"Thing":{"target":"n.page#Thing"}}},
"n.page#Thing":{"type":"structure","members":{"Id":{"target":"smithy.api#String"}}}
}}"#;

pub(super) fn compiled(protocol: &str, method: &str) -> FixtureResult<Vec<u8>> {
    let directory = tempfile::tempdir()?;
    let model = directory.path().join("models/page/service/2020-01-01");
    std::fs::create_dir_all(&model)?;
    std::fs::write(
        model.join("model.json"),
        MODEL
            .replace("PROTOCOL", protocol)
            .replace("METHOD", method),
    )?;
    let policies = [policy()?];
    let checked = overlay::check_policies(
        directory.path(),
        &policies,
        &catalog_tool::classes::DenyList::builtin(),
    )?;
    let registry = smithy::Registry::parse("Id\nThing\n")?;
    let base = smithy::build_catalog(directory.path(), &registry, "read-only-request-fixture")?;
    Ok(runtime_catalog::augment(
        &base,
        &checked,
        &kinds()?,
        &BTreeMap::from([("page".to_owned(), endpoint()?)]),
    )?)
}

fn policy() -> FixtureResult<overlay::ServiceOverlay> {
    let mut plan = overlay::OperationOverlay::default();
    plan.class = Some(overlay::OverlayClass::Enumerate);
    plan.scope = Some(overlay::Scope::Region);
    plan.items = Some(overlay::ItemSelector::parse("Thing")?);
    plan.paginator = Some(overlay::Paginator::None);
    plan.identity = Some(overlay::Identity::Key(vec![overlay::MemberPath::new(
        "Id",
    )?]));
    Ok(overlay::ServiceOverlay::new(
        "page",
        vec![("ListThings", plan)],
    )?)
}

fn kinds() -> FixtureResult<kind_map::KindMap> {
    let source = kind_map::MappingSource {
        reviewer: "Read-only request boundary fixture",
        reviewed: "2026-11-23",
        api_only: &[kind_map::ApiOnlyMapping {
            kind: "AWS::ApiOnly::PageThing",
            reason: "Reviewed read-only request fixture",
        }],
        operations: &[kind_map::OperationMapping {
            service: "page",
            operation: "ListThings",
            kind: "AWS::ApiOnly::PageThing",
        }],
        ..kind_map::MappingSource::default()
    };
    Ok(source.compile(&names::Exceptions::none())?)
}

fn endpoint() -> FixtureResult<endpoints::VerifiedEndpoint> {
    let properties =
        json!({"authSchemes":[{"name":"sigv4","signingName":"page","signingRegion":"us-east-1"}]});
    let value = json!({"version":"1.0","parameters":{"Region":{"type":"string"}},"rules":[{"type":"endpoint","conditions":[],"endpoint":{"url":"https://page.test","properties":properties}}]});
    let rules = endpoints::RuleSet::parse(Path::new("page-endpoint.json"), &value)?;
    let cases: Vec<endpoints::Case> = serde_json::from_value(
        json!([{"params":{"Region":"us-east-1"},"expect":{"endpoint":{"url":"https://page.test","properties":properties}}}]),
    )?;
    Ok(rules
        .run_cases(&cases)?
        .verified_endpoint(Some("page"))
        .map_err(|error| format!("read-only request fixture endpoint failed: {error:?}"))?)
}
