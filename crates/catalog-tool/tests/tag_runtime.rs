use catalog_tool::{
    classes::DenyList, endpoints, kind_map, names, overlay, runtime_catalog, smithy,
};
use inventory_core::{
    catalog::runtime::{RuntimeCatalog, RuntimeCatalogError, TagPlan},
    record::{FieldId, Key, MapRef, Record, Value, ValueRef},
};
use serde_json::json;
use std::{collections::BTreeMap, fs, path::Path};

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
    // A multi-valued TagValues member is not a scalar tag value: no plan may be emitted.
    let plural = ITEM.replace("Key", "TagKey").replace("Value", "TagValues");
    assert!(compiled("Items", Some("Tags"), LIST, &plural).is_err());
    // Two key spellings on one element stay ambiguous and fail closed.
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
    // A response-root decoy Tags map must not rescue the selected item's invalid Tags list.
    assert!(compiled("Items", Some("Tags"), LIST, &missing).is_err());
    assert!(compiled("Items", Some("Items.Tags"), LIST, ITEM).is_err());
    assert!(compiled("Items", Some("Id.Tags"), LIST, ITEM).is_err());
    Ok(())
}

fn rewrite(
    bytes: &[u8],
    change: impl FnOnce(&mut BTreeMap<Key, Value>) -> TestResult,
) -> TestResult<Vec<u8>> {
    let mut value = ValueRef::Map(MapRef::from_bytes_bounded(bytes)?).to_value()?;
    let Value::Map(root) = &mut value else {
        return Err("root is not map".into());
    };
    let Some(Value::Map(extension)) = root.get_mut(&Key::Field(FieldId::new(60_000)?)) else {
        return Err("extension absent".into());
    };
    change(extension)?;
    Ok(Record::encode(&value)?.into_bytes())
}

#[test]
fn given_old_extension_schema_when_loaded_then_reader_rejects_without_compatibility_shim()
-> TestResult {
    let bytes = compiled("Items", Some("Tags"), LIST, ITEM)?;
    for version in [1, 3] {
        let bytes = rewrite(&bytes, |extension| {
            extension.insert(Key::Field(FieldId::new(1)?), Value::Int(version));
            Ok(())
        })?;
        assert!(
            matches!(RuntimeCatalog::from_bytes(&bytes), Err(RuntimeCatalogError::ExtensionVersion(actual)) if actual == version)
        );
    }
    Ok(())
}

#[test]
fn given_tampered_numeric_tag_shape_when_loaded_then_model_mismatch_is_rejected() -> TestResult {
    let bytes = compiled("Items", Some("Tags"), LIST, ITEM)?;
    for (field, replacement) in [
        (1, Value::Int(1)),
        (2, Value::List(vec![Value::Int(2)])),
        (3, Value::Int(2)),
        (4, Value::Int(4)),
    ] {
        let bytes = rewrite(&bytes, |extension| {
            let Some(Value::List(plans)) = extension.get_mut(&Key::Field(FieldId::new(2)?)) else {
                return Err("plans absent".into());
            };
            let Some(Value::Map(plan)) = plans.first_mut() else {
                return Err("plan absent".into());
            };
            let Some(Value::Map(tags)) = plan.get_mut(&Key::Field(FieldId::new(14)?)) else {
                return Err("tags absent".into());
            };
            tags.insert(Key::Field(FieldId::new(field)?), replacement);
            Ok(())
        })?;
        assert!(matches!(
            RuntimeCatalog::from_bytes(&bytes),
            Err(RuntimeCatalogError::Capability)
        ));
    }
    Ok(())
}

#[test]
fn given_nonmap_tag_metadata_when_loaded_then_invalid_type_is_not_treated_as_unobserved()
-> TestResult {
    let bytes = compiled("Items", Some("Tags"), LIST, ITEM)?;
    for replacement in [Value::Null, Value::Int(0), Value::List(Vec::new())] {
        let bytes = rewrite(&bytes, |extension| {
            let Some(Value::List(plans)) = extension.get_mut(&Key::Field(FieldId::new(2)?)) else {
                return Err("plans absent".into());
            };
            let Some(Value::Map(plan)) = plans.first_mut() else {
                return Err("plan absent".into());
            };
            plan.insert(Key::Field(FieldId::new(14)?), replacement);
            Ok(())
        })?;
        assert!(matches!(
            RuntimeCatalog::from_bytes(&bytes),
            Err(RuntimeCatalogError::Capability)
        ));
    }
    Ok(())
}

#[test]
fn given_map_tag_metadata_pointing_to_list_shape_when_loaded_then_shape_kind_mismatch_is_rejected()
-> TestResult {
    let bytes = compiled("Items", Some("Tags"), LIST, ITEM)?;
    let bytes = rewrite(&bytes, |extension| {
        let Some(Value::List(plans)) = extension.get_mut(&Key::Field(FieldId::new(2)?)) else {
            return Err("plans absent".into());
        };
        let Some(Value::Map(plan)) = plans.first_mut() else {
            return Err("plan absent".into());
        };
        let Some(Value::Map(tags)) = plan.get_mut(&Key::Field(FieldId::new(14)?)) else {
            return Err("tags absent".into());
        };
        tags.insert(Key::Field(FieldId::new(1)?), Value::Int(1));
        tags.remove(&Key::Field(FieldId::new(3)?));
        tags.remove(&Key::Field(FieldId::new(4)?));
        Ok(())
    })?;
    assert!(matches!(
        RuntimeCatalog::from_bytes(&bytes),
        Err(RuntimeCatalogError::Capability)
    ));
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

fn child<'a>(value: &'a mut serde_json::Value, key: &str) -> TestResult<&'a mut serde_json::Value> {
    value
        .get_mut(key)
        .ok_or_else(|| format!("missing member {key}").into())
}

fn set(value: &mut serde_json::Value, key: &str, new: serde_json::Value) -> TestResult {
    value
        .as_object_mut()
        .ok_or("object expected")?
        .insert(key.to_owned(), new);
    Ok(())
}

fn relation_fixture(detail_callable: bool, same_kind: bool) -> TestResult<Vec<u8>> {
    let mut model: serde_json::Value =
        serde_json::from_str(&MODEL.replace("TAG_SHAPE", MAP).replace("TAG_ITEM", ITEM))?;
    let shapes = child(&mut model, "shapes")?;
    set(
        child(shapes, "n#Service")?,
        "operations",
        json!([{"target":"n#ListFirst"},{"target":"n#ListThings"}]),
    )?;
    set(
        shapes,
        "n#ListFirst",
        json!({"type":"operation","output":{"target":"n#SummaryOutput"},"traits":{"smithy.api#http":{"method":"GET","uri":"/summary"}}}),
    )?;
    set(
        shapes,
        "n#SummaryOutput",
        json!({"type":"structure","members":{"Items":{"target":"n#Summaries"}}}),
    )?;
    set(
        shapes,
        "n#Summaries",
        json!({"type":"list","member":{"target":"n#Summary"}}),
    )?;
    set(
        shapes,
        "n#Summary",
        json!({"type":"structure","members":{"Id":{"target":"smithy.api#String"}}}),
    )?;
    set(
        child(child(shapes, "n#Container")?, "members")?,
        "Id",
        json!({"target":"smithy.api#String"}),
    )?;
    let mut first = overlay::OperationOverlay::default();
    first.class = Some(overlay::OverlayClass::Enumerate);
    first.scope = Some(overlay::Scope::Region);
    first.items = Some(overlay::ItemSelector::parse("Items")?);
    first.identity = Some(overlay::Identity::Key(vec![overlay::MemberPath::new(
        "Id",
    )?]));
    first.paginator = Some(overlay::Paginator::None);
    let detail = if detail_callable {
        first.clone()
    } else {
        let mut deferred = overlay::OperationOverlay::default();
        deferred.class = Some(overlay::OverlayClass::Defer);
        deferred.defer_reason = Some("Fixture explicitly excludes the detail operation".to_owned());
        deferred
    };
    let policies = [overlay::ServiceOverlay::new(
        "things",
        vec![("ListFirst", first), ("ListThings", detail)],
    )?];
    let mappings = kind_map::MappingSource {
        api_only: &[
            kind_map::ApiOnlyMapping {
                kind: "AWS::ApiOnly::TagFixture",
                reason: "Detail path fixture",
            },
            kind_map::ApiOnlyMapping {
                kind: "AWS::ApiOnly::Other",
                reason: "Unrelated-kind isolation fixture",
            },
        ],
        operations: &[
            kind_map::OperationMapping {
                service: "things",
                operation: "ListFirst",
                kind: "AWS::ApiOnly::TagFixture",
            },
            kind_map::OperationMapping {
                service: "things",
                operation: "ListThings",
                kind: if same_kind {
                    "AWS::ApiOnly::TagFixture"
                } else {
                    "AWS::ApiOnly::Other"
                },
            },
        ],
        relations: &[kind_map::RelationMapping {
            source_kind: "AWS::ApiOnly::TagFixture",
            source: kind_map::RelationSource::SelectedOutput(kind_map::RelationPath::Fields(
                "Envelope.Id",
            )),
            target_kind: "AWS::ApiOnly::TagFixture",
            target: kind_map::RelationTarget::SelectedOutput(kind_map::RelationPath::Fields("Id")),
            relation: "LINKS_TO",
            scope: "account_region",
            guard: None,
        }],
        ..kind_map::MappingSource::default()
    };
    compiled_with(&serde_json::to_string(&model)?, &policies, &mappings)
}

#[test]
fn given_same_kind_summary_and_detail_when_relation_compiled_then_any_reviewed_detail_model_can_supply_path()
-> TestResult {
    let bytes = relation_fixture(true, true)?;
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    let relation = runtime.relationships().first().ok_or("relation missing")?;
    assert_eq!(relation.source_path, path(&[1, 2])?);
    assert_eq!(relation.target_path, path(&[2])?);
    Ok(())
}

#[test]
fn given_only_unrelated_or_noncallable_detail_paths_when_relation_compiled_then_metadata_is_rejected()
-> TestResult {
    for (callable, same_kind) in [(false, true), (true, false)] {
        let error = relation_fixture(callable, same_kind)
            .err()
            .ok_or("invalid relation compiled")?;
        assert!(
            matches!(
                error.downcast_ref::<runtime_catalog::RuntimeCatalogCompileError>(),
                Some(runtime_catalog::RuntimeCatalogCompileError::Capability)
            ),
            "{error}"
        );
    }
    Ok(())
}
