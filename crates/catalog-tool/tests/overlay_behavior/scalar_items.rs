use super::*;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn scalar_model(shape: &str) -> String {
    BASIC.replace("\"target\":\"n#Thing\"", &format!("\"target\":\"smithy.api#{shape}\""))
}

fn scalar_overlay(identity: &str) -> String {
    format!("{HEADER}{}", COMPLETE.replace("key=['Id']", identity))
}

#[test]
fn given_scalar_items_when_current_item_is_the_key_then_all_integer_widths_are_reviewable() -> TestResult {
    ["String", "Boolean", "Byte", "Short", "Integer", "Long"].into_iter()
        .try_for_each(|shape| {
            let given = fixture(&scalar_model(shape), "")?;
            overlay(given.path(), &scalar_overlay("key=['$']"))?;
            let when = overlay::check(given.path(), &source(given.path()))?;
            let then = when.services().first().ok_or("service missing")?
                .callable().first().ok_or("callable missing")?;
            assert!(matches!(then.plan().identity.as_ref(), Some(overlay::Identity::Key(paths)) if paths.len() == 1 && paths.first().is_some_and(|path| path.as_str() == "$")));
            Ok(())
        })
}

#[test]
fn given_string_items_when_current_item_is_the_arn_then_review_preserves_the_selector() -> TestResult {
    let given = fixture(&scalar_model("String"), "")?;
    overlay(given.path(), &scalar_overlay("arn='$'"))?;
    let when = overlay::check(given.path(), &source(given.path()))?;
    let then = when.services().first().ok_or("service missing")?
        .callable().first().ok_or("callable missing")?;
    assert!(matches!(then.plan().identity.as_ref(), Some(overlay::Identity::Arn(path)) if path.as_str() == "$"));
    Ok(())
}

#[test]
fn given_structured_items_when_self_identity_is_requested_then_review_rejects_it() -> TestResult {
    let given = fixture(BASIC, "")?;
    overlay(given.path(), &scalar_overlay("key=['$']"))?;
    let when = violations(given.path())?;
    assert!(when.iter().any(|violation| violation.rule == ViolationRule::InvalidPath && violation.key == "identity"));
    Ok(())
}

#[test]
fn given_root_items_when_keyed_by_member_then_root_meaning_is_unchanged() -> TestResult {
    let given = fixture(BASIC, "")?;
    overlay(given.path(), &format!("{HEADER}{}", COMPLETE.replace("items='Items'", "items='$'")))?;
    let when = overlay::check(given.path(), &source(given.path()))?;
    let then = when.services().first().ok_or("service missing")?
        .callable().first().ok_or("callable missing")?;
    assert_eq!(then.plan().items, Some(ItemSelector::Root));
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

const CHILD_OVERLAY: &str = "[operation.GetThing]\nclass='expand'\nitems='$'\nbind={Id={from='ListThings',path='$'}}\n[operation.GetThing.paginator]\nkind='none'\n[operation.GetThing.identity]\nkey=['Id']\n";

#[test]
fn given_scalar_parents_when_bound_to_matching_child_input_then_review_accepts_exact_type() -> TestResult {
    ["String", "Boolean", "Byte", "Short", "Integer", "Long"].into_iter()
        .try_for_each(|shape| {
            let given = fixture(&PARENT_MODEL.replace("SCALAR", shape), "")?;
            overlay(given.path(), &format!("{}{CHILD_OVERLAY}", scalar_overlay("key=['$']")))?;
            let when = overlay::check(given.path(), &source(given.path()))?;
            assert_eq!(when.services().first().ok_or("service missing")?.callable().len(), 2);
            Ok(())
        })
}

#[test]
fn given_structured_parent_when_self_binding_is_requested_then_review_rejects_it() -> TestResult {
    let model = PARENT_MODEL.replace("SCALAR", "String")
        .replace("\"n#Items\":{\"type\":\"list\",\"member\":{\"target\":\"smithy.api#String\"}}", "\"n#Items\":{\"type\":\"list\",\"member\":{\"target\":\"n#Thing\"}}");
    let given = fixture(&model, "")?;
    overlay(given.path(), &format!("{HEADER}{COMPLETE}{CHILD_OVERLAY}"))?;
    assert!(violations(given.path())?.iter().any(|violation| {
        violation.rule == ViolationRule::InvalidValue && violation.key == "bind"
    }));
    Ok(())
}

#[test]
fn given_numeric_items_when_current_item_is_an_arn_then_review_rejects_nontext_identity() -> TestResult {
    let given = fixture(&scalar_model("Long"), "")?;
    overlay(given.path(), &scalar_overlay("arn='$'"))?;
    assert!(violations(given.path())?.iter().any(|violation| {
        violation.rule == ViolationRule::InvalidPath && violation.key == "identity"
    }));
    Ok(())
}
