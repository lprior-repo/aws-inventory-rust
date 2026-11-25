use catalog_tool::{
    classes::DenyList, endpoints, kind_map, names, overlay, runtime_catalog, smithy,
};
use inventory_core::{
    catalog::runtime::{RuntimeCatalog, RuntimeCatalogError, TagPlan},
    record::{FieldId, Key, MapRef, Record, Value, ValueRef},
};
use serde_json::json;
use std::{collections::BTreeMap, fs, path::Path};

#[path = "tag_runtime/decoder.rs"]
mod decoder;
#[path = "tag_runtime/ec2.rs"]
mod ec2;
#[path = "tag_runtime/relations.rs"]
mod relations;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const MODEL: &str = r#"{"smithy":"2.0","shapes":{
"n#Service":{"type":"service","version":"1","operations":[{"target":"n#ListThings"}],"traits":{"aws.api#service":{"sdkId":"Things","endpointPrefix":"things","arnNamespace":"things"},"aws.auth#sigv4":{"name":"things"},"aws.protocols#restJson1":{}}},
"n#ListThings":{"type":"operation","output":{"target":"n#Output"},"traits":{"smithy.api#http":{"method":"GET","uri":"/things"}}},
"n#Output":{"type":"structure","members":{"Items":{"target":"n#Items"},"Envelope":{"target":"n#Thing"},"Id":{"target":"smithy.api#String"},"Tags":{"target":"n#Decoy"}}},
"n#Items":{"type":"list","member":{"target":"n#Thing"}},
"n#Thing":{"type":"structure","members":{"Id":{"target":"smithy.api#String"},"Tags":{"target":"n#Tags"},"Envelope":{"target":"n#Container"}}},
"n#Container":{"type":"structure","members":{"Tags":{"target":"n#Tags"}}},
"n#Decoy":{"type":"map","key":{"target":"smithy.api#String"},"value":{"target":"smithy.api#String"}},
"n#Tags":TAG_SHAPE,
"n#Tag":TAG_ITEM
}}"#;
const LIST: &str = r#"{"type":"list","member":{"target":"n#Tag"}}"#;
const MAP: &str =
    r#"{"type":"map","key":{"target":"smithy.api#String"},"value":{"target":"smithy.api#String"}}"#;
const ITEM: &str = r#"{"type":"structure","members":{"Key":{"target":"smithy.api#String"},"Value":{"target":"smithy.api#String"}}}"#;

fn endpoint() -> TestResult<endpoints::VerifiedEndpoint> {
    let properties = json!({"authSchemes":[{"name":"sigv4","signingName":"things","signingRegion":"us-east-1"}]});
    let document = json!({"version":"1.0","parameters":{"Region":{"type":"string"}},"rules":[{"type":"endpoint","conditions":[],"endpoint":{"url":"https://things.test","properties":properties}}]});
    let rules = endpoints::RuleSet::parse(Path::new("tag-endpoint.json"), &document)?;
    let cases: Vec<endpoints::Case> = serde_json::from_value(
        json!([{"params":{"Region":"us-east-1"},"expect":{"endpoint":{"url":"https://things.test","properties":properties}}}]),
    )?;
    rules
        .run_cases(&cases)?
        .verified_endpoint(Some("things"))
        .map_err(|failure| format!("endpoint evidence failed: {failure:?}").into())
}

fn compiled(selector: &str, tags: Option<&str>, shape: &str, item: &str) -> TestResult<Vec<u8>> {
    let mut operation = overlay::OperationOverlay::default();
    operation.class = Some(overlay::OverlayClass::Enumerate);
    operation.scope = Some(overlay::Scope::Region);
    operation.items = Some(overlay::ItemSelector::parse(selector)?);
    operation.identity = Some(overlay::Identity::Key(vec![overlay::MemberPath::new(
        "Id",
    )?]));
    operation.paginator = Some(overlay::Paginator::None);
    operation.tags = tags.map(overlay::MemberPath::selected).transpose()?;
    let policies = [overlay::ServiceOverlay::new(
        "things",
        vec![("ListThings", operation)],
    )?];
    let mappings = kind_map::MappingSource {
        api_only: &[kind_map::ApiOnlyMapping {
            kind: "AWS::ApiOnly::TagFixture",
            reason: "Numeric tag compiler fixture",
        }],
        operations: &[kind_map::OperationMapping {
            service: "things",
            operation: "ListThings",
            kind: "AWS::ApiOnly::TagFixture",
        }],
        ..kind_map::MappingSource::default()
    };
    compiled_with(
        &MODEL.replace("TAG_SHAPE", shape).replace("TAG_ITEM", item),
        &policies,
        &mappings,
    )
}

fn compiled_with(
    model: &str,
    policies: &[overlay::ServiceOverlay],
    mappings: &kind_map::MappingSource<'_>,
) -> TestResult<Vec<u8>> {
    let root = tempfile::tempdir()?;
    let source = root.path().join("source");
    let models = source.join("models/things/service/2020-01-01");
    fs::create_dir_all(&models)?;
    fs::write(models.join("model.json"), model)?;
    let checked = overlay::check_policies(&source, policies, &DenyList::builtin())?;
    let registry = smithy::Registry::parse(
        "Envelope\nId\nItems\nKey\nTags\nValue\nkey\nvalue\nTagKey\nTagValue\nTagValues\n",
    )?;
    let base = smithy::build_catalog(&source, &registry, "tag-fixture")?;
    let kinds = mappings.compile(&names::Exceptions::none())?;
    Ok(runtime_catalog::augment(
        &base,
        &checked,
        &kinds,
        &BTreeMap::from([("things".to_owned(), endpoint()?)]),
    )?)
}

fn tag_plan(bytes: &[u8]) -> TestResult<Option<TagPlan>> {
    let runtime = RuntimeCatalog::from_bytes(bytes)?;
    let plan = runtime
        .lookup("things", "ListThings")
        .ok_or("missing callable plan")?;
    assert_eq!(
        plan.hash_exclusions(),
        &inventory_core::record::Exclusions::none()
    );
    Ok(plan.tags().cloned())
}

fn path(ids: &[u32]) -> TestResult<Box<[FieldId]>> {
    Ok(ids
        .iter()
        .map(|id| FieldId::new(*id))
        .collect::<Result<Vec<_>, _>>()?
        .into_boxed_slice())
}

#[test]
fn given_list_tags_when_compiled_then_exact_numeric_item_relative_plan_roundtrips() -> TestResult {
    for selector in ["Items", "Envelope"] {
        let bytes = compiled(selector, Some("Tags"), LIST, ITEM)?;
        assert_eq!(
            tag_plan(&bytes)?,
            Some(TagPlan::List {
                path: path(&[5])?,
                key: FieldId::new(4)?,
                value: FieldId::new(6)?
            })
        );
    }
    let lower = ITEM.replace("Key", "key").replace("Value", "value");
    let bytes = compiled("Items", Some("Envelope.Tags"), LIST, &lower)?;
    assert_eq!(
        tag_plan(&bytes)?,
        Some(TagPlan::List {
            path: path(&[1, 5])?,
            key: FieldId::new(7)?,
            value: FieldId::new(8)?
        })
    );
    Ok(())
}

#[test]
fn given_prefixed_tag_members_when_compiled_then_key_and_value_fields_are_bound() -> TestResult {
    let prefixed = ITEM.replace("Key", "TagKey").replace("Value", "TagValue");
    let bytes = compiled("Items", Some("Tags"), LIST, &prefixed)?;
    assert_eq!(
        tag_plan(&bytes)?,
        Some(TagPlan::List {
            path: path(&[5])?,
            key: FieldId::new(9)?,
            value: FieldId::new(10)?
        })
    );
    let plural = ITEM.replace("Key", "TagKey").replace("Value", "TagValues");
    assert!(compiled("Items", Some("Tags"), LIST, &plural).is_err());
    let ambiguous = prefixed.replace(
        "\"TagKey\":",
        "\"Key\":{\"target\":\"smithy.api#String\"},\"TagKey\":",
    );
    assert!(compiled("Items", Some("Tags"), LIST, &ambiguous).is_err());
    Ok(())
}

#[test]
fn given_map_or_absent_tags_when_compiled_then_root_nested_and_unobserved_stay_distinct()
-> TestResult {
    for (selector, tags, expected) in [
        ("$", Some("Tags"), Some(path(&[5])?)),
        ("Items", Some("Envelope.Tags"), Some(path(&[1, 5])?)),
        ("Items", None, None),
    ] {
        let bytes = compiled(selector, tags, MAP, ITEM)?;
        assert_eq!(
            tag_plan(&bytes)?,
            expected.map(|path| TagPlan::Map { path })
        );
    }
    let root = compiled("$", Some("Tags"), LIST, ITEM)?;
    assert_eq!(tag_plan(&root)?, Some(TagPlan::Map { path: path(&[5])? }));
    Ok(())
}

#[test]
fn given_invalid_tag_models_when_compiled_then_no_executable_plan_is_emitted() -> TestResult {
    let missing = ITEM.replace("\"Key\":{\"target\":\"smithy.api#String\"},", "");
    let nonstring = ITEM.replace("smithy.api#String", "smithy.api#Boolean");
    let ambiguous = ITEM.replace(
        "\"Key\":",
        "\"key\":{\"target\":\"smithy.api#String\"},\"Key\":",
    );
    for (shape, item) in [
        (LIST.to_owned(), missing.clone()),
        (LIST.to_owned(), nonstring),
        (LIST.to_owned(), ambiguous),
        (LIST.to_owned(), r#"{"type":"string"}"#.to_owned()),
        (
            MAP.replace(
                "\"value\":{\"target\":\"smithy.api#String\"}",
                "\"value\":{\"target\":\"smithy.api#Boolean\"}",
            ),
            ITEM.to_owned(),
        ),
        (r#"{"type":"string"}"#.to_owned(), ITEM.to_owned()),
    ] {
        let error = compiled("Items", Some("Tags"), &shape, &item)
            .err()
            .ok_or("invalid tag model compiled")?;
        assert!(
            matches!(
                error.downcast_ref::<runtime_catalog::RuntimeCatalogCompileError>(),
                Some(runtime_catalog::RuntimeCatalogCompileError::CapabilityContext { .. })
            ),
            "{error}"
        );
    }
    assert!(compiled("Items", Some("Tags"), LIST, &missing).is_err());
    assert!(compiled("Items", Some("Items.Tags"), LIST, ITEM).is_err());
    assert!(compiled("Items", Some("Id.Tags"), LIST, ITEM).is_err());
    Ok(())
}

#[test]
fn given_self_tag_path_on_structured_items_when_checked_then_invalid_tag_selection_is_rejected()
-> TestResult {
    for selector in ["$", "Items", "Envelope"] {
        let error = compiled(selector, Some("$"), MAP, ITEM)
            .err()
            .ok_or("structured self tags accepted")?;
        let Some(overlay::CheckError::Violations(violations)) =
            error.downcast_ref::<overlay::CheckError>()
        else {
            return Err(format!("expected checker rejection, got {error}").into());
        };
        assert!(violations.iter().any(|violation| violation.key() == "tags"
            && violation.rule() == overlay::ViolationRule::InvalidPath));
    }
    Ok(())
}
