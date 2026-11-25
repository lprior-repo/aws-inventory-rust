use super::*;

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
                Some(runtime_catalog::RuntimeCatalogCompileError::CapabilityContext { service, operation, path })
                    if service == "things" && operation == "ListFirst" && path == "Envelope.Id"
            ),
            "{error}"
        );
    }
    Ok(())
}
