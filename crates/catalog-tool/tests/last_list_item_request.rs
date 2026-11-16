use catalog_tool::{endpoints, kind_map, names, overlay, runtime_catalog, smithy};
use inventory_core::catalog::runtime::{RuntimeCatalog, TokenPlacement, TokenSelection, TokenType};
use serde_json::json;
use std::{collections::BTreeMap, fs, path::Path};

type TestResult = Result<(), Box<dyn std::error::Error>>;

const MODEL: &str = r#"{"smithy":"2.0","shapes":{
"n#Service":{"type":"service","version":"1","operations":[{"target":"n#ListThings"}],"traits":{"aws.api#service":{"sdkId":"Things","endpointPrefix":"things","arnNamespace":"things"},"aws.auth#sigv4":{"name":"things"},"aws.protocols#restJson1":{}}},
"n#ListThings":{"type":"operation","input":{"target":"n#Input"},"output":{"target":"n#Output"},"traits":{"smithy.api#http":{"method":"POST","uri":"/things"}}},
"n#Input":{"type":"structure","members":{"Cursor":{"target":"smithy.api#String"},"Parent":{"target":"n#Parent"},"Parents":{"target":"n#Parents"},"ParentByName":{"target":"n#ParentByName"}}},
"n#Parent":{"type":"structure","members":{"Cursor":{"target":"smithy.api#String"}}},
"n#Parents":{"type":"list","member":{"target":"n#Parent"}},
"n#ParentByName":{"type":"map","key":{"target":"smithy.api#String"},"value":{"target":"n#Parent"}},
"n#Output":{"type":"structure","members":{"Items":{"target":"n#Items"},"Names":{"target":"n#Names"},"Truncated":{"target":"smithy.api#Boolean"}}},
"n#Items":{"type":"list","member":{"target":"n#Thing"}},
"n#Thing":{"type":"structure","members":{"Id":{"target":"smithy.api#String"}}},
"n#Names":{"type":"list","member":{"target":"smithy.api#String"}}
}}"#;
const REGISTRY: &str = "Cursor\nId\nItems\nNames\nParent\nParentByName\nParents\nTruncated\n";
const KINDS: kind_map::MappingSource<'static> = kind_map::MappingSource {
    version: 1,
    reviewed: "2026-11-15",
    reviewer: "Paginator request path fixture",
    default_unmapped_reason: None,
    cfn: &[], relations: &[], parent_relations: &[], compound_relations: &[],
    additional_emissions: &[], relation_targets: &[], bootstraps: &[],
    api_only: &[kind_map::ApiOnlyMapping {
        kind: "AWS::ApiOnly::Thing", reason: "Paginator request path fixture",
    }],
    operations: &[kind_map::OperationMapping {
        service: "things", operation: "ListThings", kind: "AWS::ApiOnly::Thing",
    }],
};

fn endpoint() -> Result<endpoints::VerifiedEndpoint, Box<dyn std::error::Error>> {
    let properties = json!({"authSchemes":[{"name":"sigv4","signingName":"things","signingRegion":"us-east-1"}]});
    let document = json!({"version":"1.0","parameters":{"Region":{"type":"string"}},"rules":[{"type":"endpoint","conditions":[],"endpoint":{"url":"https://things.test","properties":properties}}]});
    let rules = endpoints::RuleSet::parse(Path::new("paginator-endpoint.json"), &document)?;
    let cases: Vec<endpoints::Case> = serde_json::from_value(json!([{"params":{"Region":"us-east-1"},"expect":{"endpoint":{"url":"https://things.test","properties":properties}}}]))?;
    rules.run_cases(&cases)?.verified_endpoint(Some("things"))
        .map_err(|_| "fixture endpoint evidence failed".into())
}

fn compile(input: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let root = tempfile::tempdir()?;
    let source = root.path().join("source");
    let model_dir = source.join("models/things/service/2020-01-01");
    fs::create_dir_all(&model_dir)?;
    fs::write(model_dir.join("model.json"), MODEL)?;
    let mut plan = overlay::OperationOverlay::default();
    plan.class = Some(overlay::OverlayClass::Enumerate);
    plan.scope = Some(overlay::Scope::Region);
    plan.items = Some(overlay::ItemSelector::parse("Items")?);
    plan.identity = Some(overlay::Identity::Key(vec![overlay::MemberPath::new("Id")?]));
    plan.paginator = Some(overlay::Paginator::LastListItem {
        input: overlay::MemberPath::new(input)?,
        output: overlay::MemberPath::new("Names")?,
        truncated: overlay::MemberPath::new("Truncated")?,
    });
    let policies = [overlay::ServiceOverlay::new("things", vec![("ListThings", plan)])?];
    let checked = overlay::check_policies(&source, &policies, &catalog_tool::classes::DenyList::builtin())?;
    let fields = smithy::Registry::parse(REGISTRY)?;
    let base = smithy::build_catalog(&source, &fields, "paginator-request-fixture")?;
    let kinds = KINDS.compile(&names::Exceptions::none())?;
    let endpoints = BTreeMap::from([("things".to_owned(), endpoint()?)]);
    Ok(runtime_catalog::augment(&base, &checked, &kinds, &endpoints)?)
}

#[test]
fn last_list_item_checker_rejects_projected_list_and_map_request_paths_before_compilation() -> TestResult {
    // Given collection-valued request ancestors whose elements have a text Cursor.
    ["Parents.Cursor", "ParentByName.Cursor"].into_iter().try_for_each(|input| -> TestResult {
        // When using the public check-to-native-compile pipeline.
        let failure = match compile(input) {
            Err(failure) => failure,
            Ok(_) => return Err("collection-projected request cursor was authorized".into()),
        };
        // Then rejection happens in the checker, not later in the native compiler.
        let errors = match failure.downcast_ref::<overlay::CheckError>() {
            Some(overlay::CheckError::Violations(errors)) => errors,
            _ => return Err(format!("expected checker violations for {input}, got {failure}").into()),
        };
        assert!(errors.iter().any(|error| error.key() == "paginator.token"
            && error.value() == input && error.rule() == overlay::ViolationRule::InvalidValue),
            "{input}: {errors:?}");
        Ok(())
    })
}

#[test]
fn last_list_item_checker_preserves_scalar_request_cursors_through_native_compilation() -> TestResult {
    // Given direct and structure-nested text request cursors, never collection projections.
    let fields = smithy::Registry::parse(REGISTRY)?;
    ["Cursor", "Parent.Cursor"].into_iter().try_for_each(|input| -> TestResult {
        // When public review is followed by native compilation and catalog loading.
        let bytes = compile(input)?;
        let catalog = RuntimeCatalog::from_bytes(&bytes)?;
        let plan = catalog.lookup("things", "ListThings").ok_or("callable plan missing")?;
        // Then exact request destinations and last-list-item semantics survive all boundaries.
        let request = input.split('.').map(|part| fields.id(part).ok_or("request field missing"))
            .collect::<Result<Vec<_>, _>>()?;
        let names = fields.id("Names").ok_or("response field missing")?;
        let truncated = fields.id("Truncated").ok_or("truncation field missing")?;
        assert_eq!(plan.pagination().tokens.len(), 1);
        let token = plan.pagination().tokens.first().ok_or("cursor missing")?;
        assert_eq!(token.request_path.as_ref(), request.as_slice());
        assert_eq!(token.response_path.as_ref(), &[names]);
        assert_eq!(token.request_placement, TokenPlacement::Body(request.into_boxed_slice()));
        assert_eq!(token.response_placement, TokenPlacement::Body(vec![names].into_boxed_slice()));
        assert_eq!(token.selection, TokenSelection::LastListItem);
        assert_eq!(token.value_type, TokenType::String);
        assert_eq!(token.optional, false);
        assert_eq!(plan.pagination().truncated_path.as_deref(), Some([truncated].as_slice()));
        Ok(())
    })
}
