use super::*;
use overlay::{
    ExecutionBranch, FixedValue, MemberPath, Scope, TagEndpoint, TagEndpointParent,
    TagEndpointRequest, TagEndpointSelector,
};
type TestResult = Result<(), Box<dyn std::error::Error>>;

const MODEL: &str = r#"{"smithy":"2.0","shapes":{
"n#Service":{"type":"service","operations":[{"target":"n#ListThings"},{"target":"n#ListTags"},{"target":"n#DescribeEndpoints"}]},
"n#ListThings":{"type":"operation","input":{"target":"n#Input"},"output":{"target":"n#Output"}},
"n#Input":{"type":"structure","members":{"Scope":{"target":"n#Scope","traits":{"smithy.api#required":{}}}}},
"n#Scope":{"type":"enum","members":{"Regional":{"target":"smithy.api#Unit","traits":{"smithy.api#enumValue":"REGIONAL"}},"Global":{"target":"smithy.api#Unit","traits":{"smithy.api#enumValue":"GLOBAL"}}}},
"n#Output":{"type":"structure","members":{"Items":{"target":"n#Things"}}},
"n#Things":{"type":"list","member":{"target":"n#Thing"}},
"n#Thing":{"type":"structure","members":{"Id":{"target":"smithy.api#String"},"associationData":{"target":"n#Association"}}},
"n#Association":{"type":"union","members":{"knowledgeBaseAssociation":{"target":"n#KnowledgeBase"},"other":{"target":"smithy.api#String"}}},
"n#KnowledgeBase":{"type":"structure","members":{"knowledgeBaseArn":{"target":"smithy.api#String"}}},
"n#ListTags":{"type":"operation","input":{"target":"n#TagInput"},"output":{"target":"n#TagOutput"}},
"n#TagInput":{"type":"structure","members":{"ResourceArn":{"target":"smithy.api#String","traits":{"smithy.api#required":{}}}}},
"n#TagOutput":{"type":"structure","members":{"Tags":{"target":"n#Tags"}}},
"n#Tags":{"type":"map","key":{"target":"smithy.api#String"},"value":{"target":"smithy.api#String"}},
"n#DescribeEndpoints":{"type":"operation","output":{"target":"n#EndpointOutput"}},
"n#EndpointOutput":{"type":"structure","members":{"Endpoints":{"target":"n#Endpoints"}}},
"n#Endpoints":{"type":"list","member":{"target":"n#Endpoint"}},
"n#Endpoint":{"type":"structure","members":{"Address":{"target":"smithy.api#String"},"CachePeriodInMinutes":{"target":"smithy.api#Long"}}}
}}"#;

fn reviewed_operations(
    mut operations: Vec<(&str, OperationOverlay)>,
) -> Vec<(&str, OperationOverlay)> {
    for operation in ["ListThings", "ListTags", "DescribeEndpoints"] {
        if !operations.iter().any(|(name, _)| *name == operation) {
            operations.push((operation, deferred()));
        }
    }
    operations
}

fn checked(
    root: &Fixture<'_>,
    operations: Vec<(&str, OperationOverlay)>,
) -> Result<overlay::CheckedOverlays, CheckError> {
    super::checked(root, reviewed_operations(operations))
}

fn violations(
    root: &Fixture<'_>,
    operations: Vec<(&str, OperationOverlay)>,
) -> Result<Vec<Violation>, Box<dyn std::error::Error>> {
    super::violations(root, reviewed_operations(operations))
}

fn branch(name: &str, value: Option<&str>) -> Result<ExecutionBranch, overlay::NameError> {
    Ok(ExecutionBranch {
        discriminator: name.into(),
        scope: Scope::Region,
        request_region: None,
        bind: Default::default(),
        emission: None,
        row_policy: None,
        fixed: value
            .map(|value| {
                Ok::<_, overlay::NameError>((
                    MemberPath::new("Scope")?,
                    FixedValue::String(value.into()),
                ))
            })
            .transpose()?
            .into_iter()
            .collect(),
    })
}

fn parent() -> Result<OperationOverlay, overlay::NameError> {
    let mut plan = complete()?;
    plan.branches = vec![
        branch("regional", Some("REGIONAL"))?,
        branch("global", Some("GLOBAL"))?,
    ];
    Ok(plan)
}

fn tags(path: &str) -> Result<OperationOverlay, overlay::NameError> {
    TagEndpoint {
        parents: vec![TagEndpointParent {
            operation: overlay::OperationName::new("ListThings")?,
            request: TagEndpointRequest::Member {
                input: MemberPath::new("ResourceArn")?,
                parent: MemberPath::new(path)?,
            },
        }],
        selector: TagEndpointSelector::Map {
            path: MemberPath::new("Tags")?,
        },
    }
    .operation(Scope::Region, overlay::Paginator::None)
}

#[test]
fn given_required_branch_input_when_every_branch_supplies_valid_value_then_plan_is_callable()
-> TestResult {
    let given = fixture(MODEL, "")?;
    let when = checked(&given, vec![("ListThings", parent()?)])?;
    let plan = when
        .services()
        .first()
        .ok_or("service")?
        .callable()
        .first()
        .ok_or("callable")?
        .plan();
    assert_eq!(plan.branches.len(), 2);
    assert_eq!(
        plan.branches
            .first()
            .ok_or("branch")?
            .fixed
            .get(&MemberPath::new("Scope")?),
        Some(&FixedValue::String("REGIONAL".into()))
    );
    Ok(())
}

#[test]
fn given_required_branch_input_when_one_branch_omits_or_forges_it_then_plan_is_rejected()
-> TestResult {
    for value in [None, Some("UNKNOWN")] {
        let given = fixture(MODEL, "")?;
        let mut plan = parent()?;
        plan.branches = vec![
            branch("regional", Some("REGIONAL"))?,
            branch("broken", value)?,
        ];
        assert!(
            violations(&given, vec![("ListThings", plan)])?
                .iter()
                .any(|error| error.key() == "required" && error.value() == "Scope")
        );
    }
    Ok(())
}

#[test]
fn given_tag_endpoint_parent_request_when_valid_then_required_input_is_supplied() -> TestResult {
    let given = fixture(MODEL, "")?;
    let when = checked(
        &given,
        vec![("ListThings", parent()?), ("ListTags", tags("Id")?)],
    )?;
    assert_eq!(
        when.services().first().ok_or("service")?.callable().len(),
        2
    );
    let errors = violations(
        &given,
        vec![("ListThings", parent()?), ("ListTags", tags("Missing")?)],
    )?;
    assert!(
        errors
            .iter()
            .any(|error| error.key() == "tag_endpoint.parent")
    );
    assert!(
        errors
            .iter()
            .any(|error| error.key() == "required" && error.value() == "ResourceArn")
    );
    Ok(())
}

#[test]
fn given_multiple_tag_parents_when_one_omits_required_request_then_coverage_is_not_unioned()
-> TestResult {
    let given = fixture(MODEL, "")?;
    let mut tag = tags("Id")?;
    tag.tag_endpoint
        .as_mut()
        .ok_or("endpoint")?
        .parents
        .push(TagEndpointParent {
            operation: overlay::OperationName::new("DescribeEndpoints")?,
            request: TagEndpointRequest::None,
        });
    let errors = violations(&given, vec![("ListThings", parent()?), ("ListTags", tag)])?;
    assert!(
        errors
            .iter()
            .any(|error| error.key() == "required" && error.value() == "ResourceArn")
    );
    Ok(())
}

#[test]
fn given_endpoint_discovery_without_resource_selector_when_checked_then_root_and_no_paging_are_explicit()
-> TestResult {
    let given = fixture(MODEL, "")?;
    let mut discovery = OperationOverlay::default();
    discovery.class = Some(OverlayClass::Enumerate);
    discovery.scope = Some(Scope::Region);
    discovery.endpoint_discovery = true;
    let when = checked(&given, vec![("DescribeEndpoints", discovery)])?;
    let plan = when
        .services()
        .first()
        .ok_or("service")?
        .callable()
        .first()
        .ok_or("callable")?
        .plan();
    assert_eq!(plan.items, Some(ItemSelector::Root));
    assert_eq!(plan.paginator, Some(overlay::Paginator::None));
    assert!(plan.endpoint_discovery);
    Ok(())
}

#[test]
fn given_modeled_union_arm_when_queryable_then_exact_path_survives_and_invented_arm_is_rejected()
-> TestResult {
    let given = fixture(MODEL, "")?;
    let mut plan = parent()?;
    let path = MemberPath::new("associationData.knowledgeBaseAssociation.knowledgeBaseArn")?;
    plan.queryable.push(path.clone());
    let when = checked(&given, vec![("ListThings", plan)])?;
    assert_eq!(
        when.services()
            .first()
            .ok_or("service")?
            .callable()
            .first()
            .ok_or("callable")?
            .plan()
            .queryable,
        vec![path]
    );
    let mut invalid = parent()?;
    invalid.queryable.push(MemberPath::new(
        "associationData.invented.knowledgeBaseArn",
    )?);
    assert!(
        violations(&given, vec![("ListThings", invalid)])?
            .iter()
            .any(|error| error.key() == "queryable")
    );
    Ok(())
}

fn branch_consumer(binding: overlay::Binding) -> Result<OperationOverlay, overlay::NameError> {
    let mut plan = OperationOverlay::default();
    plan.class = Some(OverlayClass::Expand);
    plan.scope = Some(Scope::Region);
    plan.items = Some(ItemSelector::Root);
    plan.paginator = Some(overlay::Paginator::None);
    plan.emission = overlay::Emission::DiscoveryOnly;
    plan.fixed.insert(
        MemberPath::new("ResourceArn")?,
        FixedValue::String("base-value".into()),
    );
    let mut child = branch("child", None)?;
    child.bind.insert(MemberPath::new("ResourceArn")?, binding);
    plan.branches.push(child);
    Ok(plan)
}

#[test]
fn given_branch_qualified_binding_when_source_exists_then_exact_base_override_remains_callable()
-> TestResult {
    for binding in [
        overlay::Binding::FromBranch {
            operation: overlay::OperationName::new("ListThings")?,
            branch: "regional".into(),
            path: MemberPath::new("Id")?,
        },
        overlay::Binding::ContextFromBranch {
            operation: overlay::OperationName::new("ListThings")?,
            branch: "regional".into(),
            source: overlay::ContextSource::SelectedItem,
            path: MemberPath::new("Id")?,
        },
    ] {
        let given = fixture(MODEL, "")?;
        let when = checked(
            &given,
            vec![
                ("ListThings", parent()?),
                ("ListTags", branch_consumer(binding)?),
            ],
        )?;
        assert_eq!(
            when.services().first().ok_or("service")?.callable().len(),
            2
        );
    }
    Ok(())
}

#[test]
fn given_branch_bindings_when_target_collides_or_source_is_unmodeled_then_plan_is_rejected()
-> TestResult {
    for (path, collision) in [("Missing", false), ("Id", true)] {
        let given = fixture(MODEL, "")?;
        let mut plan = branch_consumer(overlay::Binding::FromBranch {
            operation: overlay::OperationName::new("ListThings")?,
            branch: "regional".into(),
            path: MemberPath::new(path)?,
        })?;
        if collision {
            plan.branches.first_mut().ok_or("branch")?.fixed.insert(
                MemberPath::new("ResourceArn")?,
                FixedValue::String("collision".into()),
            );
        }
        let errors = violations(&given, vec![("ListThings", parent()?), ("ListTags", plan)])?;
        assert!(
            errors
                .iter()
                .any(|error| error.key() == if collision { "input" } else { "bind" })
        );
    }
    Ok(())
}

#[test]
fn given_discovery_only_base_when_one_branch_emits_resources_then_identity_is_still_required()
-> TestResult {
    let given = fixture(MODEL, "")?;
    let mut plan = branch_consumer(overlay::Binding::FromBranch {
        operation: overlay::OperationName::new("ListThings")?,
        branch: "regional".into(),
        path: MemberPath::new("Id")?,
    })?;
    plan.branches.first_mut().ok_or("branch")?.emission = Some(overlay::Emission::EmitMetadata);
    assert!(
        violations(&given, vec![("ListThings", parent()?), ("ListTags", plan)])?
            .iter()
            .any(|error| error.key() == "identity")
    );
    Ok(())
}

fn config_model(required: bool) -> Result<String, Box<dyn std::error::Error>> {
    let mut model: serde_json::Value = serde_json::from_str(MODEL)?;
    let shapes = model
        .get_mut("shapes")
        .and_then(serde_json::Value::as_object_mut)
        .ok_or("shapes")?;
    shapes.insert(
        "n#Config".into(),
        serde_json::json!({"type":"structure","members":{
        "Left":{"target":"smithy.api#String","traits":{"smithy.api#required":{}}},
        "Right":{"target":"smithy.api#String","traits":{"smithy.api#required":{}}}}}),
    );
    let members = shapes
        .get_mut("n#Input")
        .and_then(|shape| shape.get_mut("members"))
        .and_then(serde_json::Value::as_object_mut)
        .ok_or("input members")?;
    let mut config = serde_json::json!({"target":"n#Config"});
    if required {
        config["traits"] = serde_json::json!({"smithy.api#required":{}});
    }
    members.insert("Config".into(), config);
    Ok(serde_json::to_string(&model)?)
}

#[test]
fn given_structured_branch_fixed_input_when_all_required_children_exist_then_whole_value_covers_them()
-> TestResult {
    for required in [false, true] {
        let given = fixture(&config_model(required)?, "")?;
        let mut plan = parent()?;
        let value = FixedValue::structure([
            ("Left", FixedValue::String("left".into())),
            ("Right", FixedValue::String("right".into())),
        ])?;
        plan.branches
            .first_mut()
            .ok_or("branch")?
            .fixed
            .insert(MemberPath::new("Config")?, value.clone());
        if required {
            plan.branches
                .last_mut()
                .ok_or("branch")?
                .fixed
                .insert(MemberPath::new("Config")?, value);
        }
        checked(&given, vec![("ListThings", plan)])?;
    }
    Ok(())
}

#[test]
fn given_optional_branch_structure_when_partial_members_activate_it_then_missing_required_sibling_is_rejected()
-> TestResult {
    let given = fixture(&config_model(false)?, "")?;
    let mut plan = parent()?;
    plan.branches.first_mut().ok_or("branch")?.fixed.insert(
        MemberPath::new("Config.Left")?,
        FixedValue::String("left".into()),
    );
    assert!(
        violations(&given, vec![("ListThings", plan)])?
            .iter()
            .any(|error| error.key() == "required" && error.value() == "Config.Right")
    );
    Ok(())
}

#[test]
fn given_noncallable_common_producer_when_every_branch_overrides_then_only_effective_references_are_checked()
-> TestResult {
    // Given modeled request inputs of the same scalar type and an explicitly deferred producer.
    let mut model: serde_json::Value = serde_json::from_str(MODEL)?;
    let shapes = model
        .get_mut("shapes")
        .and_then(serde_json::Value::as_object_mut)
        .ok_or("shapes")?;
    shapes
        .get_mut("n#ListTags")
        .and_then(serde_json::Value::as_object_mut)
        .ok_or("producer")?
        .insert("input".into(), serde_json::json!({"target":"n#Input"}));
    shapes
        .get_mut("n#Input")
        .and_then(|shape| shape.get_mut("members"))
        .and_then(|members| members.get_mut("Scope"))
        .and_then(serde_json::Value::as_object_mut)
        .ok_or("scope")?
        .insert("target".into(), serde_json::json!("smithy.api#String"));
    let model = serde_json::to_string(&model)?;
    let given = fixture(&model, "")?;
    for fixed in [true, false] {
        let mut plan = parent()?;
        plan.bind.insert(
            MemberPath::new("Scope")?,
            overlay::Binding::Context {
                operation: overlay::OperationName::new("ListTags")?,
                source: overlay::ContextSource::Request,
                path: MemberPath::new("Scope")?,
            },
        );
        if !fixed {
            for branch in &mut plan.branches {
                branch.fixed.clear();
                branch
                    .bind
                    .insert(MemberPath::new("Scope")?, overlay::Binding::Region);
            }
        }
        // When all actual branches override the common destination, then the consumer is callable.
        let when = checked(
            &given,
            vec![("ListThings", plan.clone()), ("ListTags", deferred())],
        )?;
        assert!(
            when.services()
                .first()
                .ok_or("service")?
                .callable()
                .iter()
                .any(|handle| handle.operation() == "ListThings")
        );
        // Given a sibling branch inherits that destination, then the deferred producer must be rejected.
        plan.branches.push(branch("inherited", None)?);
        let errors = violations(&given, vec![("ListThings", plan), ("ListTags", deferred())])?;
        assert!(errors.iter().any(|error| error.operation() == "ListThings"
            && error.value() == "ListTags"
            && error.rule() == ViolationRule::BindingSourceNotCallable));
    }
    Ok(())
}
