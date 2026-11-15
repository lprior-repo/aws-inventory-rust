use super::*;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn scalar_model(shape: &str) -> String {
    BASIC.replace("\"target\":\"n#Thing\"", &format!("\"target\":\"smithy.api#{shape}\""))
}

fn scalar_overlay(identity: overlay::Identity) -> Result<OperationOverlay, overlay::NameError> {
    let mut plan = complete()?;
    plan.identity = Some(identity);
    Ok(plan)
}

fn scalar_key() -> Result<OperationOverlay, overlay::NameError> {
    scalar_overlay(overlay::Identity::Key(vec![overlay::MemberPath::selected("$")?]))
}

fn scalar_arn() -> Result<OperationOverlay, overlay::NameError> {
    scalar_overlay(overlay::Identity::Arn(overlay::MemberPath::selected("$")?))
}

#[test]
fn given_scalar_items_when_current_item_is_the_key_then_all_integer_widths_are_reviewable() -> TestResult {
    ["String", "Boolean", "Byte", "Short", "Integer", "Long"].into_iter()
        .try_for_each(|shape| {
            let given = fixture(&scalar_model(shape), "")?;
            let when = checked(&given, vec![("ListThings", scalar_key()?)])?;
            let then = when.services().first().ok_or("service missing")?
                .callable().first().ok_or("callable missing")?;
            assert!(matches!(then.plan().identity.as_ref(), Some(overlay::Identity::Key(paths)) if paths.len() == 1 && paths.first().is_some_and(|path| path.as_str() == "$")));
            Ok(())
        })
}

#[test]
fn given_string_items_when_current_item_is_the_arn_then_review_preserves_the_selector() -> TestResult {
    let given = fixture(&scalar_model("String"), "")?;
    let when = checked(&given, vec![("ListThings", scalar_arn()?)])?;
    let then = when.services().first().ok_or("service missing")?
        .callable().first().ok_or("callable missing")?;
    assert!(matches!(then.plan().identity.as_ref(), Some(overlay::Identity::Arn(path)) if path.as_str() == "$"));
    Ok(())
}

#[test]
fn given_structured_items_when_self_identity_is_requested_then_review_rejects_it() -> TestResult {
    let given = fixture(BASIC, "")?;
    let when = violations(&given, vec![("ListThings", scalar_key()?)])?;
    assert!(when.iter().any(|violation| violation.rule() == ViolationRule::InvalidPath && violation.key() == "identity"));
    Ok(())
}

#[test]
fn given_root_items_when_keyed_by_member_then_root_meaning_is_unchanged() -> TestResult {
    let given = fixture(BASIC, "")?;
    let mut plan = complete()?;
    plan.items = Some(ItemSelector::Root);
    let when = checked(&given, vec![("ListThings", plan)])?;
    let then = when.services().first().ok_or("service missing")?
        .callable().first().ok_or("callable missing")?;
    assert_eq!(then.plan().items, Some(ItemSelector::Root));
    Ok(())
}


#[test]
fn given_scalar_items_when_self_is_queryable_then_checker_uses_scalar_selection_semantics() -> TestResult {
    let given = fixture(&scalar_model("String"), "")?;
    let mut plan = scalar_key()?;
    plan.queryable = vec![overlay::MemberPath::selected("$")?];
    let when = checked(&given, vec![("ListThings", plan)])?;
    assert_eq!(when.services().first().ok_or("service")?.callable().first().ok_or("callable")?
        .plan().queryable, vec![overlay::MemberPath::selected("$")?]);
    Ok(())
}
const PARENT_MODEL: &str = r#"{"smithy":"2.0","shapes":{
"n#Service":{"type":"service","operations":[{"target":"n#ListThings"},{"target":"n#GetThing"}]},
"n#ListThings":{"type":"operation","output":{"target":"n#Output"}},
"n#Output":{"type":"structure","members":{"Items":{"target":"n#Items"}}},
"n#Items":{"type":"list","member":{"target":"smithy.api#SCALAR"}},
"n#GetThing":{"type":"operation","input":{"target":"n#Input"},"output":{"target":"n#Thing"}},
"n#Input":{"type":"structure","members":{"Id":{"target":"smithy.api#SCALAR","traits":{"smithy.api#required":{}}}}},
"n#Thing":{"type":"structure","members":{"Id":{"target":"smithy.api#SCALAR"}}}
}}"#;

fn child_overlay() -> Result<OperationOverlay, overlay::NameError> {
    let mut plan = complete()?;
    plan.class = Some(OverlayClass::Expand);
    plan.items = Some(ItemSelector::Root);
    plan.bind.insert(overlay::MemberPath::new("Id")?, binding("ListThings", "$")?);
    Ok(plan)
}

#[test]
fn given_scalar_parents_when_bound_to_matching_child_input_then_review_accepts_exact_type() -> TestResult {
    ["String", "Boolean", "Byte", "Short", "Integer", "Long"].into_iter()
        .try_for_each(|shape| {
            let given = fixture(&PARENT_MODEL.replace("SCALAR", shape), "")?;
            let when = checked(&given, vec![
                ("ListThings", scalar_key()?),
                ("GetThing", child_overlay()?),
            ])?;
            assert_eq!(when.services().first().ok_or("service missing")?.callable().len(), 2);
            Ok(())
        })
}

#[test]
fn given_structured_parent_when_self_binding_is_requested_then_review_rejects_it() -> TestResult {
    let model = PARENT_MODEL.replace("SCALAR", "String")
        .replace("\"n#Items\":{\"type\":\"list\",\"member\":{\"target\":\"smithy.api#String\"}}", "\"n#Items\":{\"type\":\"list\",\"member\":{\"target\":\"n#Thing\"}}");
    let given = fixture(&model, "")?;
    assert!(violations(&given, vec![
        ("ListThings", complete()?),
        ("GetThing", child_overlay()?),
    ])?.iter().any(|violation| {
        violation.rule() == ViolationRule::InvalidValue && violation.key() == "bind"
    }));
    Ok(())
}

#[test]
fn given_numeric_items_when_current_item_is_an_arn_then_review_rejects_nontext_identity() -> TestResult {
    let given = fixture(&scalar_model("Long"), "")?;
    assert!(violations(&given, vec![("ListThings", scalar_arn()?)])?.iter().any(|violation| {
        violation.rule() == ViolationRule::InvalidPath && violation.key() == "identity"
    }));
    Ok(())
}
