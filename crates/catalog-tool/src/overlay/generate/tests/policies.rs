use super::{children::given_child, *};
use crate::overlay::{OperationOverlay, OverlayClass, ServiceOverlay};

fn generated(value: &Value, reviewed: Option<&ServiceOverlay>) -> Result<ServiceOverlay> {
    let model = Model::parse(value, "sample")?;
    Ok(super::super::service::generate(
        "sample",
        &model,
        Some(Scope::Region),
        &Vocabulary::default(),
        &DenyList::default(),
        reviewed,
    )?
    .policy)
}

#[test]
fn given_reviewed_parent_deny_when_generated_then_child_cannot_bypass_denial() -> Result {
    let reviewed = crate::overlay::builtin::service(
        "sample",
        vec![(
            "ListThings",
            OperationOverlay {
                class: Some(OverlayClass::Deny(crate::classes::DenyReason::Mutating)),
                ..OperationOverlay::default()
            },
        )],
    )?;
    let policy = generated(&given_child(true)?, Some(&reviewed))?;
    assert!(
        !policy
            .operations()
            .keys()
            .any(|name| name.as_str() == "ListThings")
    );
    assert!(
        policy
            .operations()
            .values()
            .all(|entry| entry.class == Some(OverlayClass::Defer))
    );
    Ok(())
}

#[test]
fn given_reviewed_child_deny_when_generated_then_child_is_omitted() -> Result {
    let reviewed = crate::overlay::builtin::service(
        "sample",
        vec![(
            "GetThing",
            OperationOverlay {
                class: Some(OverlayClass::Deny(crate::classes::DenyReason::Mutating)),
                ..OperationOverlay::default()
            },
        )],
    )?;
    let policy = generated(&given_child(false)?, Some(&reviewed))?;
    assert!(
        !policy
            .operations()
            .keys()
            .any(|name| name.as_str() == "GetThing")
    );
    assert!(
        policy
            .operations()
            .values()
            .any(|entry| entry.class == Some(OverlayClass::Enumerate))
    );
    Ok(())
}

#[test]
fn given_context_without_identity_binding_when_generated_then_no_arbitrary_parent_is_chosen()
-> Result {
    let mut value = given_child(false)?;
    set(
        &mut value,
        "/shapes/n#ChildInput",
        "members",
        json!({"AccountId":{"target":"smithy.api#String","traits":{"smithy.api#required":{}}}}),
    )?;
    let policy = generated(&value, None)?;
    let entry = policy
        .operations()
        .iter()
        .find(|(name, _)| name.as_str() == "GetThing")
        .map(|(_, entry)| entry)
        .ok_or("child missing")?;
    assert_eq!(entry.class, Some(OverlayClass::Defer));
    assert_eq!(
        entry.defer_reason.as_deref(),
        Some("expand: no unique compatible identity source")
    );
    Ok(())
}

#[test]
fn given_recursive_output_when_generated_then_traversal_terminates_without_losing_secrets() -> Result
{
    let mut value = given_child(true)?;
    set(
        &mut value,
        "/shapes/n#Thing/members",
        "Nested",
        json!({"target":"n#Thing"}),
    )?;
    assert!(
        generated(&value, None)?
            .operations()
            .values()
            .all(|entry| entry.class != Some(OverlayClass::Defer))
    );
    set(
        &mut value,
        "/shapes/n#Thing/members",
        "Password",
        json!({"target":"smithy.api#String"}),
    )?;
    assert!(
        generated(&value, None)?
            .operations()
            .values()
            .all(|entry| entry.class == Some(OverlayClass::Defer))
    );
    Ok(())
}

#[test]
fn given_specific_parent_defer_when_generated_then_parent_and_child_remain_deferred() -> Result {
    let reviewed = crate::overlay::builtin::service(
        "sample",
        vec![(
            "ListThings",
            OperationOverlay {
                class: Some(OverlayClass::Defer),
                defer_reason: Some("owner filter must be fixed".to_owned()),
                ..OperationOverlay::default()
            },
        )],
    )?;
    let policy = generated(&given_child(true)?, Some(&reviewed))?;
    let child = policy.operations().get("GetThing").ok_or("child missing")?;
    assert_eq!(child.class, Some(OverlayClass::Defer));
    assert_eq!(
        child.defer_reason.as_deref(),
        Some("expand: parent enumeration is deferred")
    );
    assert_eq!(
        policy
            .operations()
            .get("ListThings")
            .and_then(|entry| entry.class),
        Some(OverlayClass::Defer)
    );
    Ok(())
}

#[test]
fn given_only_self_identity_source_when_generated_then_child_cannot_bind_itself() -> Result {
    use crate::overlay::{Identity, ItemSelector, MemberPath, Paginator};
    let reviewed = crate::overlay::builtin::service(
        "sample",
        vec![
            (
                "ListThings",
                OperationOverlay {
                    class: Some(OverlayClass::Defer),
                    defer_reason: Some("restricted root".to_owned()),
                    ..OperationOverlay::default()
                },
            ),
            (
                "GetThing",
                OperationOverlay {
                    class: Some(OverlayClass::Enumerate),
                    scope: Some(Scope::Region),
                    items: Some(ItemSelector::Root),
                    identity: Some(Identity::Arn(MemberPath::new("Arn")?)),
                    paginator: Some(Paginator::None),
                    ..OperationOverlay::default()
                },
            ),
        ],
    )?;
    let policy = generated(&given_child(false)?, Some(&reviewed))?;
    let child = policy.operations().get("GetThing").ok_or("child missing")?;
    assert_eq!(child.class, Some(OverlayClass::Defer));
    assert_eq!(
        child.defer_reason.as_deref(),
        Some("expand: no unique compatible identity source")
    );
    Ok(())
}

fn reviewed_child(marker: Option<crate::overlay::GenerationBlocker>) -> Result<ServiceOverlay> {
    Ok(crate::overlay::builtin::service(
        "sample",
        vec![(
            "GetThing",
            OperationOverlay {
                class: Some(OverlayClass::Defer),
                generation_blocker: marker,
                defer_reason: Some("reviewed request binding blocker".to_owned()),
                ..OperationOverlay::default()
            },
        )],
    )?)
}

#[test]
fn given_specific_child_defer_when_generated_then_callable_proposal_is_not_published() -> Result {
    let reviewed = reviewed_child(None)?;
    let policy = generated(&given_child(false)?, Some(&reviewed))?;
    let child = policy.operations().get("GetThing").ok_or("child missing")?;
    assert_eq!(child.class, Some(OverlayClass::Defer));
    assert_eq!(
        child.defer_reason.as_deref(),
        Some("reviewed request binding blocker")
    );
    Ok(())
}

#[test]
fn given_typed_request_blocker_when_generated_then_validated_child_is_callable() -> Result {
    let reviewed = reviewed_child(Some(crate::overlay::GenerationBlocker::RequiredBindings))?;
    let policy = generated(&given_child(false)?, Some(&reviewed))?;
    assert_eq!(
        policy
            .operations()
            .get("GetThing")
            .and_then(|entry| entry.class),
        Some(OverlayClass::Expand)
    );
    Ok(())
}
