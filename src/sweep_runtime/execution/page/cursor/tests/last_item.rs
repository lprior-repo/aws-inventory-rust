use super::*;
use catalog_tool::{endpoints, kind_map, names, overlay, runtime_catalog, smithy};
use inventory_core::catalog::runtime::RuntimeCatalog;
use serde_json::json;
use std::{collections::BTreeMap, path::Path};

const MODEL: &str = r#"{"smithy":"2.0","shapes":{
"n#Service":{"type":"service","version":"1","operations":[{"target":"n#ListItems"}],"traits":{"aws.api#service":{"sdkId":"Pagination","endpointPrefix":"pagination","arnNamespace":"pagination"},"aws.auth#sigv4":{"name":"pagination"},"aws.protocols#restJson1":{}}},
"n#ListItems":{"type":"operation","input":{"target":"n#Input"},"output":{"target":"n#Output"},"traits":{"smithy.api#readonly":{},"smithy.api#http":{"method":"GET","uri":"/items"}}},
"n#Input":{"type":"structure","members":{"Start":{"target":"smithy.api#String","traits":{"smithy.api#httpQuery":"start"}}}},
"n#Output":{"type":"structure","members":{"Items":{"target":"n#Items"},"HasMore":{"target":"smithy.api#Boolean"}}},
"n#Items":{"type":"list","member":{"target":"smithy.api#String"}}
}}"#;

#[test]
fn given_compiled_last_item_cursor_when_resuming_then_final_list_text_is_scalar_input() -> TestResult {
    // Given a reviewed list-output/scalar-input paginator compiled and decoded by production APIs.
    let bytes = compiled_catalog_bytes()?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let page = catalog.lookup("pagination", "ListItems").ok_or("missing plan")?.pagination();
    let response = record([(1, Value::Bool(true)), (2, Value::List(vec![
        Value::Text("first".into()), Value::Text("last".into()),
    ]))])?;
    // When the real page cursor advances, checkpoints and restores its request fields.
    let next = advance(page, &response)?;
    let fields = restored(&next)?.request_fields(page)?;
    let request = input::merge_input(&empty_record()?, &fields)?;
    // Then the continuation contains the final string, not the list or its first element.
    assert_eq!(request.as_bytes(), record([(3, Value::Text("last".into()))])?.as_bytes());
    Ok(())
}

#[test]
fn given_last_item_cursor_when_not_truncated_then_empty_list_finishes() -> TestResult {
    // Given a compiled paginator whose final response has no items or continuation.
    let bytes = compiled_catalog_bytes()?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let page = catalog.lookup("pagination", "ListItems").ok_or("missing plan")?.pagination();
    let response = record([(1, Value::Bool(false)), (2, Value::List(vec![]))])?;
    // When the real cursor consumes it, then no extra request is scheduled.
    assert!(Cursor::default().next(page, &response, &http::HeaderMap::new(), 10)?.is_none());
    Ok(())
}

#[test]
fn given_last_item_cursor_when_truncated_without_valid_final_text_then_continuation_is_rejected() -> TestResult {
    // Given malformed or absent last-item continuation values on truncated pages.
    let bytes = compiled_catalog_bytes()?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let page = catalog.lookup("pagination", "ListItems").ok_or("missing plan")?.pagination();
    let cases = [
        (Value::List(vec![]), "pagination response omitted required continuation"),
        (Value::List(vec![Value::Null]), "pagination response omitted required continuation"),
        (Value::List(vec![Value::Text("first".into()), Value::Text(String::new())]),
            "pagination response omitted required continuation"),
        (Value::List(vec![Value::Int(1)]), "pagination token has the wrong native type"),
        (Value::Text("not a list".into()), "pagination last-item selector requires a list"),
    ];
    for (items, message) in cases {
        let response = record([(1, Value::Bool(true)), (2, items)])?;
        // When consumed, then invalid pages cannot produce a misleading request checkpoint.
        let result = Cursor::default().next(page, &response, &http::HeaderMap::new(), 10);
        assert!(matches!(result, Err(SweepError::Operation(actual)) if actual == message), "{message}");
    }
    Ok(())
}

fn compiled_catalog_bytes() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let models = directory.path().join("models/pagination/service/2026-01-01");
    std::fs::create_dir_all(&models)?;
    std::fs::write(models.join("model.json"), MODEL)?;
    let mut operation = overlay::OperationOverlay::default();
    operation.class = Some(overlay::OverlayClass::Enumerate);
    operation.scope = Some(overlay::Scope::Account);
    operation.items = Some(overlay::ItemSelector::parse("Items")?);
    operation.identity = Some(overlay::Identity::Key(vec![overlay::MemberPath::selected("$")?]));
    operation.paginator = Some(overlay::Paginator::LastListItem {
        input: overlay::MemberPath::new("Start")?, output: overlay::MemberPath::new("Items")?,
        truncated: overlay::MemberPath::new("HasMore")?,
    });
    let policy = overlay::ServiceOverlay::new("pagination", vec![("ListItems", operation)])?;
    let checked = overlay::check_policies(directory.path(), &[policy], &catalog_tool::classes::DenyList::builtin())?;
    let registry = smithy::Registry::parse("HasMore\nItems\nStart\n")?;
    let base = smithy::build_catalog(directory.path(), &registry, "last-item-consumer")?;
    let kinds = kind_map::MappingSource {
        version: 1, reviewed: "2026-11-15", reviewer: "Cursor regression", default_unmapped_reason: None,
        cfn: &[], relations: &[], parent_relations: &[], compound_relations: &[], additional_emissions: &[], relation_targets: &[],
        api_only: &[kind_map::ApiOnlyMapping { kind: "AWS::ApiOnly::PaginationScalar", reason: "Scalar fixture" }],
        operations: &[kind_map::OperationMapping { service: "pagination", operation: "ListItems", kind: "AWS::ApiOnly::PaginationScalar" }],
        bootstraps: &[],
    }.compile(&names::Exceptions::none())?;
    let properties = json!({"authSchemes":[{"name":"sigv4","signingName":"pagination","signingRegion":"us-east-1"}]});
    let document = json!({"version":"1.0","parameters":{},"rules":[{"type":"endpoint","conditions":[],"endpoint":{"url":"https://pagination.invalid","properties":properties}}]});
    let rules = endpoints::RuleSet::parse(Path::new("pagination-endpoint.json"), &document)?;
    let cases = serde_json::from_value::<Vec<endpoints::Case>>(json!([{"params":{},"expect":{"endpoint":{"url":"https://pagination.invalid","properties":properties}}}]))?;
    let endpoint = rules.run_cases(&cases)?.verified_endpoint(Some("pagination"))
        .map_err(|error| format!("fixture endpoint derivation failed: {error:?}"))?;
    let endpoints = BTreeMap::from([("pagination".to_owned(), endpoint)]);
    runtime_catalog::augment(&base, &checked, &kinds, &endpoints).map_err(Into::into)
}
