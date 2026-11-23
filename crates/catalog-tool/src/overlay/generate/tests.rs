mod children;
mod edge;
mod identities;
mod policies;
mod root_identity;
use super::{Derived, derive};
use crate::{
    classes::DenyList,
    overlay::{Identity, Paginator, Scope},
    plans::{Model, Vocabulary},
};
use serde_json::{Value, json};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

pub(super) fn given_model() -> Value {
    json!({"smithy":"2.0","shapes":{
        "n#Service":{"type":"service","operations":[{"target":"n#ListThings"}]},
        "n#ListThings":{"type":"operation","input":{"target":"n#Input"},"output":{"target":"n#Output"}},
        "n#Input":{"type":"structure","members":{}},
        "n#Output":{"type":"structure","members":{"Items":{"target":"n#Items"}}},
        "n#Items":{"type":"list","member":{"target":"n#Thing"}},
        "n#Thing":{"type":"structure","members":{"Arn":{"target":"smithy.api#String"}}}
    }})
}

fn set(value: &mut Value, path: &str, name: &str, entry: Value) -> Result {
    value
        .pointer_mut(path)
        .and_then(Value::as_object_mut)
        .ok_or("missing object")?
        .insert(name.to_owned(), entry);
    Ok(())
}

fn derived(value: &Value, vocabulary: &Vocabulary, deny: &DenyList<'_>) -> Result<Derived> {
    let model = Model::parse(value, "sample")?;
    Ok(derive::enumerate(
        &model,
        "ListThings",
        Scope::Region,
        vocabulary,
        deny,
    ))
}

fn then_callable(result: Derived) -> Result<crate::overlay::OperationOverlay> {
    match result {
        Ok(entry) => Ok(entry),
        Err(reason) => Err(reason.into()),
    }
}

fn then_deferred(result: Derived, expected: &str) -> Result {
    match result {
        Err(reason) => {
            assert_eq!(reason, expected);
            Ok(())
        }
        Ok(_) => Err("unexpected callable policy".into()),
    }
}

#[test]
fn given_single_list_when_derived_then_arn_and_explicit_unpaginated_are_written() -> Result {
    let entry = then_callable(derived(
        &given_model(),
        &Vocabulary::default(),
        &DenyList::builtin(),
    )?)?;
    assert!(matches!(entry.identity, Some(Identity::Arn(path)) if path.as_str() == "Arn"));
    assert_eq!(entry.paginator, Some(Paginator::None));
    assert!(entry.page_size.is_none());
    Ok(())
}

#[test]
fn given_modeled_pagination_when_derived_then_inferred_fields_stay_unset() -> Result {
    let mut value = given_model();
    add_tokens(&mut value)?;
    modeled(&mut value)?;
    let entry = then_callable(derived(
        &value,
        &Vocabulary::default(),
        &DenyList::builtin(),
    )?)?;
    assert!(entry.items.is_none());
    assert!(entry.paginator.is_none());
    assert!(entry.identity.is_some());
    Ok(())
}

fn add_tokens(value: &mut Value) -> Result {
    for path in ["/shapes/n#Input/members", "/shapes/n#Output/members"] {
        set(
            value,
            path,
            "NextToken",
            json!({"target":"smithy.api#String"}),
        )?;
    }
    Ok(())
}

fn modeled(value: &mut Value) -> Result {
    set(
        value,
        "/shapes/n#ListThings",
        "traits",
        json!({"smithy.api#paginated":{
            "inputToken":"NextToken","outputToken":"NextToken","items":"Items"
        }}),
    )
}

fn vocabulary() -> Result<Vocabulary> {
    let mut value = given_model();
    add_tokens(&mut value)?;
    modeled(&mut value)?;
    let mut vocabulary = Vocabulary::default();
    vocabulary.record(&Model::parse(&value, "sample")?)?;
    Ok(vocabulary)
}

#[test]
fn given_two_lists_when_derived_then_ambiguous_selection_is_deferred() -> Result {
    let mut value = given_model();
    set(
        &mut value,
        "/shapes/n#Output/members",
        "Other",
        json!({"target":"n#Items"}),
    )?;
    then_deferred(
        derived(&value, &Vocabulary::default(), &DenyList::builtin())?,
        "items: no single list member",
    )
}

#[test]
fn given_known_token_pair_when_derived_then_tokens_are_explicit() -> Result {
    let mut value = given_model();
    add_tokens(&mut value)?;
    let entry = then_callable(derived(&value, &vocabulary()?, &DenyList::builtin())?)?;
    assert!(matches!(entry.paginator, Some(Paginator::Token(pair))
        if pair.input.as_str() == "NextToken" && pair.output.as_str() == "NextToken"));
    Ok(())
}

#[test]
fn given_truncation_flag_when_derived_then_missing_token_policy_is_deferred() -> Result {
    let mut value = given_model();
    set(
        &mut value,
        "/shapes/n#Output/members",
        "IsTruncated",
        json!({"target":"smithy.api#Boolean"}),
    )?;
    then_deferred(
        derived(&value, &Vocabulary::default(), &DenyList::builtin())?,
        "paginator: TruncationFlag",
    )
}

#[test]
fn given_scalar_string_items_when_derived_then_root_key_is_used() -> Result {
    let mut value = given_model();
    set(
        &mut value,
        "/shapes/n#Items",
        "member",
        json!({"target":"smithy.api#String"}),
    )?;
    let entry = then_callable(derived(
        &value,
        &Vocabulary::default(),
        &DenyList::builtin(),
    )?)?;
    assert!(matches!(entry.identity, Some(Identity::Key(paths))
        if paths.first().is_some_and(|path| path.as_str() == "$") && paths.len() == 1));
    Ok(())
}

#[test]
fn given_owner_filter_when_derived_then_reviewed_fixed_input_is_required() -> Result {
    for name in include_str!("../../../../../catalog/overlay/defer-inputs.txt").lines() {
        let mut value = given_model();
        set(
            &mut value,
            "/shapes/n#Input/members",
            name,
            json!({"target":"smithy.api#String"}),
        )?;
        then_deferred(
            derived(&value, &Vocabulary::default(), &DenyList::builtin())?,
            &format!("input {name} needs a reviewed fixed value"),
        )?;
    }
    Ok(())
}

#[test]
fn given_new_deny_rule_when_derived_then_candidate_cannot_be_opened() -> Result {
    let deny = DenyList::parse("sample/ListThings\tsecret\n")?;
    then_deferred(
        derived(&given_model(), &Vocabulary::default(), &deny)?,
        "not an enumerate candidate",
    )
}

#[test]
fn given_required_input_when_derived_then_expand_is_not_enumerated() -> Result {
    let mut value = given_model();
    set(
        &mut value,
        "/shapes/n#Input/members",
        "Id",
        json!({"target":"smithy.api#String",
        "traits":{"smithy.api#required":{}}}),
    )?;
    then_deferred(
        derived(&value, &Vocabulary::default(), &DenyList::builtin())?,
        "not an enumerate candidate",
    )
}
