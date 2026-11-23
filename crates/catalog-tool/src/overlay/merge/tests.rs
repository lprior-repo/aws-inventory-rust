use super::*;
use crate::overlay::{GenerationBlocker, OperationOverlay, OverlayClass};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn merged(hand: OperationOverlay, generated: OverlayClass) -> Result<ServiceOverlay> {
    let mut hand = crate::overlay::builtin::service("sample", vec![("GetThing", hand)])?;
    let generated = crate::overlay::builtin::service(
        "sample",
        vec![(
            "GetThing",
            OperationOverlay {
                class: Some(generated),
                defer_reason: (generated == OverlayClass::Defer)
                    .then(|| "generated blocker".to_owned()),
                ..OperationOverlay::default()
            },
        )],
    )?;
    add_generated(&mut hand, generated)?;
    Ok(hand)
}

fn marked(class: OverlayClass) -> OperationOverlay {
    OperationOverlay {
        class: Some(class),
        defer_reason: (class == OverlayClass::Defer)
            .then(|| "missing required bindings".to_owned()),
        generation_blocker: Some(GenerationBlocker::RequiredBindings),
        ..OperationOverlay::default()
    }
}

fn entry(policy: &ServiceOverlay) -> Result<&OperationOverlay> {
    policy
        .operations
        .values()
        .next()
        .ok_or_else(|| "entry missing".into())
}

#[test]
fn given_binding_blocker_when_generated_child_merges_then_typed_marker_allows_replacement() -> Result
{
    let policy = merged(marked(OverlayClass::Defer), OverlayClass::Expand)?;
    assert_eq!(entry(&policy)?.class, Some(OverlayClass::Expand));
    assert_eq!(policy.origin.values().next(), Some(&Origin::Generated));
    Ok(())
}

#[test]
fn given_specific_review_when_generated_child_merges_then_review_remains_absolute() -> Result {
    let hand = OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("parent needs filtering".to_owned()),
        ..OperationOverlay::default()
    };
    let policy = merged(hand.clone(), OverlayClass::Expand)?;
    assert_eq!(entry(&policy)?, &hand);
    Ok(())
}

#[test]
fn given_deny_even_with_marker_when_generated_child_merges_then_deny_wins() -> Result {
    let hand = marked(OverlayClass::Deny(crate::classes::DenyReason::Mutating));
    assert_eq!(entry(&merged(hand.clone(), OverlayClass::Expand)?)?, &hand);
    Ok(())
}

#[test]
fn given_marked_blocker_when_nonchild_merges_then_blocker_is_not_erased() -> Result {
    for class in [OverlayClass::Enumerate, OverlayClass::Defer] {
        let hand = marked(OverlayClass::Defer);
        assert_eq!(entry(&merged(hand.clone(), class)?)?, &hand);
    }
    Ok(())
}

#[test]
fn given_callable_review_with_marker_when_generated_child_merges_then_review_is_preserved() -> Result
{
    let hand = marked(OverlayClass::Enumerate);
    assert_eq!(entry(&merged(hand.clone(), OverlayClass::Expand)?)?, &hand);
    Ok(())
}
