use super::{Result, given_model};
use crate::{
    classes::DenyList,
    overlay::{Binding, Identity, ItemSelector, MemberPath, OverlayClass, Scope},
    plans::{Model, Vocabulary},
};
use serde_json::{Value, json};

pub(super) fn given_child(modeled: bool) -> Result<Value> {
    let mut value = given_model();
    let shapes = value
        .get_mut("shapes")
        .and_then(Value::as_object_mut)
        .ok_or("shapes missing")?;
    shapes.insert("n#GetThing".into(), json!({"type":"operation","input":{"target":"n#ChildInput"},"output":{"target":"n#Thing"},"traits":{"smithy.api#readonly":{}}}));
    shapes.insert("n#ChildInput".into(), json!({"type":"structure","members":{"Arn":{"target":"smithy.api#String","traits":{"smithy.api#required":{}}}}}));
    if modeled {
        shapes.insert("n#Resource".into(), json!({"type":"resource","identifiers":{"Arn":{"target":"smithy.api#String"}},"list":{"target":"n#ListThings"},"read":{"target":"n#GetThing"}}));
    }
    Ok(value)
}

fn generated(value: &Value) -> Result<crate::overlay::ServiceOverlay> {
    let model = Model::parse(value, "sample")?;
    Ok(crate::overlay::generate::service::generate(
        "sample",
        &model,
        Some(Scope::Region),
        &Vocabulary::default(),
        &DenyList::default(),
        None,
    )?
    .policy)
}

fn child(value: &Value) -> Result<crate::overlay::OperationOverlay> {
    generated(value)?
        .operations()
        .iter()
        .find(|(name, _)| name.as_str() == "GetThing")
        .map(|(_, entry)| entry.clone())
        .ok_or_else(|| "child missing".into())
}

#[test]
fn given_modeled_identifier_when_generated_then_parent_and_root_are_callable() -> Result {
    let entry = child(&given_child(true)?)?;
    assert_eq!(entry.class, Some(OverlayClass::Expand));
    assert_eq!(entry.items, Some(ItemSelector::Root));
    assert!(entry.bind.is_empty());
    Ok(())
}

#[test]
fn given_unique_identity_when_generated_then_explicit_binding_is_written() -> Result {
    let entry = child(&given_child(false)?)?;
    assert_eq!(entry.class, Some(OverlayClass::Expand));
    assert!(
        matches!(entry.bind.values().next(), Some(Binding::From {operation, path}) if operation.as_str() == "ListThings" && path.as_str() == "Arn")
    );
    let rendered = crate::overlay::generate::render::render(&generated(&given_child(false)?)?)?;
    let parsed = crate::overlay::ServiceOverlay::parse(
        "sample",
        &rendered,
        crate::overlay::Origin::Generated,
    )?;
    assert_eq!(
        parsed.operations(),
        generated(&given_child(false)?)?.operations()
    );
    Ok(())
}

fn modify_member(value: &mut Value, shape: &str, name: &str, member: Value) -> Result {
    value
        .pointer_mut(&format!("/shapes/{shape}/members"))
        .and_then(Value::as_object_mut)
        .ok_or("members missing")?
        .insert(name.to_owned(), member);
    Ok(())
}

#[test]
fn given_deferred_parent_when_generated_then_modeled_child_stays_deferred() -> Result {
    let mut value = given_child(true)?;
    modify_member(
        &mut value,
        "n#Input",
        "Owners",
        json!({"target":"smithy.api#String"}),
    )?;
    assert_eq!(
        child(&value)?.defer_reason.as_deref(),
        Some("expand: parent enumeration is deferred")
    );
    Ok(())
}

#[test]
fn given_ambiguous_identity_parents_when_generated_then_child_stays_deferred() -> Result {
    let mut value = given_child(false)?;
    value
        .get_mut("shapes")
        .and_then(Value::as_object_mut)
        .ok_or("shapes missing")?
        .insert(
            "n#ListOtherThings".into(),
            json!({"type":"operation","input":{"target":"n#Input"},"output":{"target":"n#Output"}}),
        );
    assert_eq!(
        child(&value)?.defer_reason.as_deref(),
        Some("expand: ambiguous identity sources")
    );
    Ok(())
}

#[test]
fn given_incompatible_identity_shape_when_generated_then_child_stays_deferred() -> Result {
    let mut value = given_child(false)?;
    modify_member(
        &mut value,
        "n#ChildInput",
        "Arn",
        json!({"target":"smithy.api#Integer","traits":{"smithy.api#required":{}}}),
    )?;
    assert_eq!(
        child(&value)?.defer_reason.as_deref(),
        Some("expand: no unique compatible identity source")
    );
    Ok(())
}

#[test]
fn given_owner_sensitive_child_when_generated_then_child_stays_deferred() -> Result {
    let mut value = given_child(true)?;
    modify_member(
        &mut value,
        "n#ChildInput",
        "IncludeShared",
        json!({"target":"smithy.api#Boolean"}),
    )?;
    assert_eq!(
        child(&value)?.defer_reason.as_deref(),
        Some("input IncludeShared needs a reviewed fixed value")
    );
    Ok(())
}

#[test]
fn given_nested_sensitive_output_when_generated_then_parent_and_child_are_deferred() -> Result {
    let mut value = given_child(true)?;
    modify_member(
        &mut value,
        "n#Thing",
        "Payload",
        json!({"target":"smithy.api#String","traits":{"smithy.api#sensitive":{}}}),
    )?;
    assert_eq!(
        child(&value)?.defer_reason.as_deref(),
        Some("output: secret-bearing member Payload")
    );
    assert!(
        generated(&value)?
            .operations()
            .values()
            .all(|entry| entry.class == Some(OverlayClass::Defer))
    );
    Ok(())
}

#[test]
fn given_two_child_collections_when_generated_then_ambiguity_is_not_hidden() -> Result {
    let mut value = given_child(true)?;
    modify_member(&mut value, "n#Thing", "First", json!({"target":"n#Items"}))?;
    modify_member(&mut value, "n#Thing", "Second", json!({"target":"n#Items"}))?;
    assert_eq!(
        child(&value)?.defer_reason.as_deref(),
        Some("items: no single list member")
    );
    Ok(())
}

#[test]
fn given_single_structure_child_when_generated_then_wrapper_is_selected() -> Result {
    let mut value = given_child(true)?;
    *value
        .pointer_mut("/shapes/n#GetThing/output/target")
        .ok_or("output missing")? = json!("n#Wrapper");
    value
        .get_mut("shapes")
        .and_then(Value::as_object_mut)
        .ok_or("shapes missing")?
        .insert(
            "n#Wrapper".into(),
            json!({"type":"structure","members":{"Thing":{"target":"n#Thing"}}}),
        );
    let entry = child(&value)?;
    assert_eq!(entry.class, Some(OverlayClass::Expand));
    assert_eq!(entry.items, Some(ItemSelector::parse("Thing")?));
    Ok(())
}

#[test]
fn given_context_inputs_when_generated_then_context_is_not_fabricated() -> Result {
    let mut value = given_child(false)?;
    ["AccountId", "Region"].into_iter().try_for_each(|name| {
        modify_member(
            &mut value,
            "n#ChildInput",
            name,
            json!({"target":"smithy.api#String","traits":{"smithy.api#required":{}}}),
        )
    })?;
    let entry = child(&value)?;
    assert_eq!(entry.class, Some(OverlayClass::Expand));
    assert!(
        entry
            .bind
            .values()
            .any(|binding| *binding == Binding::Account)
    );
    assert!(
        entry
            .bind
            .values()
            .any(|binding| *binding == Binding::Region)
    );
    Ok(())
}

#[test]
fn given_root_identity_with_incidental_collection_when_generated_then_resource_root_is_selected()
-> Result {
    let mut value = given_child(true)?;
    let shapes = value
        .get_mut("shapes")
        .and_then(Value::as_object_mut)
        .ok_or("shapes missing")?;
    shapes.insert("n#Tags".into(), json!({"type":"map","key":{"target":"smithy.api#String"},"value":{"target":"smithy.api#String"}}));
    modify_member(&mut value, "n#Thing", "Tags", json!({"target":"n#Tags"}))?;
    let entry = child(&value)?;
    assert_eq!(entry.class, Some(OverlayClass::Expand));
    assert_eq!(entry.items, Some(ItemSelector::Root));
    assert_eq!(entry.identity, Some(Identity::Arn(MemberPath::new("Arn")?)));
    Ok(())
}

#[test]
fn given_credential_field_variants_when_generated_then_no_root_or_child_is_callable() -> Result {
    for name in [
        "MasterUserPassword",
        "OauthAccessToken",
        "TdeCredentials",
        "EnvironmentVariables",
        "Headers",
    ] {
        let mut value = given_child(false)?;
        modify_member(
            &mut value,
            "n#Thing",
            name,
            json!({"target":"smithy.api#String"}),
        )?;
        assert!(
            generated(&value)?
                .operations()
                .values()
                .all(|entry| entry.class == Some(OverlayClass::Defer))
        );
    }
    Ok(())
}
