use catalog_tool::overlay;
use inventory_core::{
    catalog::runtime::{RuntimeCatalog, RuntimeCatalogError},
    record::{FieldId, Key, MapRef, Record, Value, ValueRef},
};
#[path = "../../core/tests/support/runtime_provenance.rs"]
mod runtime_provenance;
#[path = "support/member_exclusion.rs"]
mod support;
use support::{TestResult, checked, compiled};

#[test]
fn given_reviewed_nested_member_when_compiled_and_loaded_then_only_its_name_is_excluded()
-> TestResult {
    let bytes = compiled(&["Items.Secret"])?;
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    let plan = runtime
        .lookup("things", "ListThings")
        .ok_or("missing plan")?;
    assert!(plan.excluded_members().contains(FieldId::new(4)?));
    for id in 1..=3 {
        assert!(!plan.excluded_members().contains(FieldId::new(id)?));
    }
    Ok(())
}

#[test]
fn given_empty_policy_when_compiled_then_existing_catalog_behavior_is_preserved() -> TestResult {
    let bytes = compiled(&[])?;
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    let plan = runtime
        .lookup("things", "ListThings")
        .ok_or("missing plan")?;
    assert!(!plan.excluded_members().contains(FieldId::new(4)?));
    Ok(())
}

#[test]
fn given_unmodeled_or_nonroot_path_when_checking_then_no_callable_can_be_published() -> TestResult {
    for path in [
        "Secret",
        "Items.Absent",
        "Items.Secret.Child",
        "Items.Attributes.arbitrary",
    ] {
        let error = checked(&[path]).err().ok_or("invalid policy accepted")?;
        assert!(
            error.to_string().contains("exclude_response_members"),
            "{path}: {error}"
        );
    }
    Ok(())
}

#[test]
fn given_more_than_eight_exclusions_when_checking_then_policy_is_bounded() -> TestResult {
    let error = checked(&["Items.Secret"; 9])
        .err()
        .ok_or("oversized policy accepted")?;
    assert!(error.to_string().contains("exclude_response_members"));
    Ok(())
}

#[test]
fn given_reused_output_shape_when_excluding_its_member_then_all_sites_are_detected() -> TestResult {
    let mut model = support::model()?;
    let members = model
        .pointer_mut("/shapes/n#Output/members")
        .and_then(serde_json::Value::as_object_mut)
        .ok_or("missing output")?;
    members.insert("Other".into(), serde_json::json!({"target":"n#Thing"}));
    let error = support::checked_model(&["Items.Secret"], &model)
        .err()
        .ok_or("ambiguous exclusion accepted")?;
    assert!(
        error.to_string().contains("multiple output sites"),
        "{error}"
    );
    Ok(())
}

#[test]
fn given_union_variant_when_excluding_it_then_policy_rejects_decode_failure() -> TestResult {
    let mut model = support::model()?;
    let shape = model
        .pointer_mut("/shapes/n#Attributes")
        .ok_or("missing shape")?;
    *shape = serde_json::json!({
        "type":"union","members":{"Hidden":{"target":"smithy.api#String"}}
    });
    let error = support::checked_model(&["Items.Attributes.Hidden"], &model)
        .err()
        .ok_or("union exclusion accepted")?;
    assert!(error.to_string().contains("union parent"), "{error}");
    Ok(())
}

#[test]
fn given_path_through_map_when_checking_then_policy_rejects_map_traversal() -> TestResult {
    let mut model = support::model()?;
    let shapes = model
        .pointer_mut("/shapes")
        .and_then(serde_json::Value::as_object_mut)
        .ok_or("missing shapes")?;
    shapes.insert(
        "n#Inner".to_owned(),
        serde_json::json!({"type":"structure","members":{"Leaf":{"target":"smithy.api#String"}}}),
    );
    shapes.insert(
        "n#ThingMap".to_owned(),
        serde_json::json!({"type":"map","key":{"target":"smithy.api#String"},"value":{"target":"n#Inner"}}),
    );
    let thing = model
        .pointer_mut("/shapes/n#Thing/members")
        .and_then(serde_json::Value::as_object_mut)
        .ok_or("missing thing members")?;
    thing.insert(
        "Config".to_owned(),
        serde_json::json!({"target":"n#ThingMap"}),
    );
    let error = support::checked_model(&["Items.Config.Leaf"], &model)
        .err()
        .ok_or("map traversal accepted")?;
    assert!(
        error.to_string().contains("path traverses a map"),
        "{error}"
    );
    Ok(())
}

#[test]
fn given_path_through_nine_nested_lists_when_checking_then_policy_rejects_depth() -> TestResult {
    let mut model = support::model()?;
    let shapes = model
        .pointer_mut("/shapes")
        .and_then(serde_json::Value::as_object_mut)
        .ok_or("missing shapes")?;
    shapes.insert(
        "n#Deepest".to_owned(),
        serde_json::json!({"type":"structure","members":{"Secret":{"target":"smithy.api#String"}}}),
    );
    let chain = [
        "n#L1",
        "n#L2",
        "n#L3",
        "n#L4",
        "n#L5",
        "n#L6",
        "n#L7",
        "n#L8",
        "n#L9",
        "n#Deepest",
    ];
    for pair in chain.windows(2) {
        let [name, element] = pair else {
            return Err("list chain pair is malformed".into());
        };
        shapes.insert(
            (*name).to_owned(),
            serde_json::json!({"type":"list","member":{"target":element}}),
        );
    }
    let thing = model
        .pointer_mut("/shapes/n#Thing/members")
        .and_then(serde_json::Value::as_object_mut)
        .ok_or("missing thing members")?;
    thing.insert("Deep".to_owned(), serde_json::json!({"target":"n#L1"}));
    let error = support::checked_model(&["Items.Deep.Secret"], &model)
        .err()
        .ok_or("9-list path accepted")?;
    assert!(
        error.to_string().contains("path nests more than 8 lists"),
        "{error}"
    );
    Ok(())
}

fn field(id: u32) -> TestResult<Key> {
    Ok(Key::Field(FieldId::new(id)?))
}

fn rewritten(bytes: &[u8], replacement: Value) -> TestResult<Vec<u8>> {
    let mut root = ValueRef::Map(MapRef::from_bytes_bounded(bytes)?).to_value()?;
    let Value::Map(fields) = &mut root else {
        return Err("root not map".into());
    };
    let Some(Value::Map(extension)) = fields.get_mut(&field(60_000)?) else {
        return Err("extension missing".into());
    };
    let Some(Value::List(plans)) = extension.get_mut(&field(2)?) else {
        return Err("plans missing".into());
    };
    let Some(Value::Map(plan)) = plans.first_mut() else {
        return Err("plan missing".into());
    };
    plan.insert(field(24)?, replacement);
    runtime_provenance::reseal(&mut root, bytes)?;
    Ok(Record::encode(&root)?.into_bytes())
}

#[test]
fn given_honestly_resealed_bad_exclusion_when_loading_then_consumer_rejects_it() -> TestResult {
    let bytes = compiled(&["Items.Secret"])?;
    for value in [
        Value::Int(1),
        Value::List(vec![Value::List(vec![])]),
        Value::List(vec![Value::List(vec![Value::Int(999)])]),
    ] {
        let changed = rewritten(&bytes, value)?;
        assert!(matches!(
            RuntimeCatalog::from_bytes(&changed),
            Err(RuntimeCatalogError::Capability
                | RuntimeCatalogError::Catalog(_)
                | RuntimeCatalogError::Bounds)
        ));
    }
    Ok(())
}

const CASES: &[(&str, &str, &[&str])] = &[
    (
        "iam",
        "ListInstanceProfiles",
        &["InstanceProfiles.Roles.AssumeRolePolicyDocument"],
    ),
    (
        "iam",
        "GetInstanceProfile",
        &["InstanceProfile.Roles.AssumeRolePolicyDocument"],
    ),
    (
        "iam",
        "ListInstanceProfilesForRole",
        &["InstanceProfiles.Roles.AssumeRolePolicyDocument"],
    ),
    (
        "lambda",
        "ListFunctions",
        &["Functions.Environment", "Functions.ImageConfigResponse"],
    ),
    (
        "lambda",
        "GetFunctionConfiguration",
        &["Environment", "ImageConfigResponse"],
    ),
    (
        "sns",
        "ListPlatformApplications",
        &["PlatformApplications.Attributes"],
    ),
];

#[test]
fn given_reviewed_builtin_unlocks_when_loading_policy_then_required_omissions_remain() -> TestResult
{
    let policies = overlay::embedded()?;
    for (service, operation, paths) in CASES {
        let policy = policies
            .iter()
            .find(|policy| policy.service() == *service)
            .ok_or("missing service")?;
        let plan = policy
            .operations()
            .get(*operation)
            .ok_or("missing operation")?;
        assert!(matches!(
            plan.class,
            Some(overlay::OverlayClass::Enumerate | overlay::OverlayClass::Expand)
        ));
        assert!(
            plan.exclude_response_members
                .iter()
                .map(|path| path.as_str())
                .eq(paths.iter().copied())
        );
    }
    Ok(())
}
