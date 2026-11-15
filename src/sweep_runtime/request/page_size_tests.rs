use super::encode;
use crate::sweep_runtime::input::{InputField, root_bindings};
use catalog_tool::{endpoints, kind_map, names, overlay, runtime_catalog, smithy};
use inventory_core::{catalog::runtime::{PageSizeType, RuntimeCatalog}, record::Value};
use serde_json::json;
use std::{collections::BTreeMap, path::Path};
type TestResult = Result<(), Box<dyn std::error::Error>>;

const MODEL: &str = r#"{"smithy":"2.0","shapes":{
"n.page#Service":{"type":"service","version":"1","operations":[{"target":"n.page#ListThings"}],"traits":{"aws.api#service":{"sdkId":"Page","endpointPrefix":"page","arnNamespace":"page"},"aws.auth#sigv4":{"name":"page"},"PROTOCOL":{}}},
"n.page#ListThings":{"type":"operation","input":{"target":"n.page#Input"},"output":{"target":"n.page#Output"},"traits":{"smithy.api#http":{"method":"GET","uri":"/things"},"smithy.api#paginated":{"inputToken":"Next","outputToken":"Next","items":"Items","pageSize":"MaxItems"}}},
"n.page#Input":{"type":"structure","members":{"Next":{"target":"smithy.api#String","traits":{"smithy.api#httpQuery":"marker"}},"MaxItems":{"target":"n.page#Size","traits":{"smithy.api#httpQuery":"max-items"}}}},
"n.page#Size":{"type":"TYPE","traits":{"smithy.api#range":{"min":1,"max":100}}},
"n.page#Output":{"type":"structure","members":{"Next":{"target":"smithy.api#String"},"Items":{"target":"n.page#Items"}}},
"n.page#Items":{"type":"list","member":{"target":"n.page#Thing"}},
"n.page#Thing":{"type":"structure","members":{"Id":{"target":"smithy.api#String"}}}
}}"#;

fn compiled(protocol: &str, size_type: &str, maximum: i64) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let model = directory.path().join("models/page/service/2020-01-01");
    std::fs::create_dir_all(&model)?;
    std::fs::write(model.join("model.json"), MODEL.replace("PROTOCOL", protocol).replace("TYPE", size_type))?;
    let mut plan = overlay::OperationOverlay::default();
    plan.class = Some(overlay::OverlayClass::Enumerate);
    plan.scope = Some(overlay::Scope::Region);
    plan.identity = Some(overlay::Identity::Key(vec![overlay::MemberPath::new("Id")?]));
    plan.page_size = Some(maximum);
    let policies = [overlay::ServiceOverlay::new("page", vec![("ListThings", plan)])?];
    let checked = overlay::check_policies(directory.path(), &policies, &catalog_tool::classes::DenyList::builtin())?;
    let registry = smithy::Registry::parse("Id\nItems\nMaxItems\nNext\n")?;
    let base = smithy::build_catalog(directory.path(), &registry, "typed-page-size-fixture")?;
    let properties = json!({"authSchemes":[{"name":"sigv4","signingName":"page","signingRegion":"us-east-1"}]});
    let endpoint = json!({"version":"1.0","parameters":{"Region":{"type":"string"}},"rules":[{"type":"endpoint","conditions":[],"endpoint":{"url":"https://page.test","properties":properties}}]});
    let rules = endpoints::RuleSet::parse(Path::new("page-endpoint.json"), &endpoint)?;
    let cases: Vec<endpoints::Case> = serde_json::from_value(json!([{"params":{"Region":"us-east-1"},"expect":{"endpoint":{"url":"https://page.test","properties":properties}}}]))?;
    let verified = rules.run_cases(&cases)?.verified_endpoint(Some("page"))?;
    let source = kind_map::MappingSource {
        reviewer: "Typed page size fixture", reviewed: "2026-11-15",
        api_only: &[kind_map::ApiOnlyMapping { kind: "AWS::ApiOnly::PageThing", reason: "Synthetic wire fixture" }],
        operations: &[kind_map::OperationMapping { service: "page", operation: "ListThings", kind: "AWS::ApiOnly::PageThing" }],
        ..kind_map::MappingSource::default()
    };
    let kinds = source.compile(&names::Exceptions::none())?;
    Ok(runtime_catalog::augment(&base, &checked, &kinds, &BTreeMap::from([("page".to_owned(), verified)]))?)
}

#[test]
fn given_modeled_page_size_when_compiled_and_json_encoded_then_string_and_long_remain_distinct() -> TestResult {
    for (kind, expected_type, expected_value) in [
        ("string", PageSizeType::String, json!("100")),
        ("long", PageSizeType::Integer, json!(100)),
    ] {
        let bytes = compiled("aws.protocols#awsJson1_1", kind, 100)?;
        let catalog = RuntimeCatalog::from_bytes(&bytes)?;
        let plan = catalog.lookup("page", "ListThings").ok_or("missing plan")?;
        let size = plan.pagination().page_size.as_ref().ok_or("page size")?;
        assert_eq!(size.value_type, expected_type);
        let base = root_bindings(plan, None, None)?;
        let request = encode(plan, "https://page.test", &base,
            &[InputField { path: size.path.clone(), value: size.value() }])?;
        let body: serde_json::Value = serde_json::from_slice(request.body())?;
        assert_eq!(body.get("MaxItems"), Some(&expected_value));
    }
    Ok(())
}

#[test]
fn given_cloudfront_shaped_string_page_size_when_rest_xml_encoded_then_query_limit_is_decimal_text() -> TestResult {
    let bytes = compiled("aws.protocols#restXml", "string", 100)?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let plan = catalog.lookup("page", "ListThings").ok_or("missing plan")?;
    let size = plan.pagination().page_size.as_ref().ok_or("page size")?;
    assert_eq!(size.value(), Value::Text("100".into()));
    let base = root_bindings(plan, None, None)?;
    let request = encode(plan, "https://page.test", &base,
        &[InputField { path: size.path.clone(), value: size.value() }])?;
    assert_eq!(request.uri().query(), Some("max-items=100"));
    Ok(())
}

#[test]
fn given_modeled_page_limit_bounds_when_compiling_then_zero_negative_and_above_model_max_are_rejected() -> TestResult {
    for kind in ["string", "long"] {
        for maximum in [-1, 0, 101, i64::MAX] {
            assert!(compiled("aws.protocols#awsJson1_1", kind, maximum).is_err());
        }
    }
    Ok(())
}
