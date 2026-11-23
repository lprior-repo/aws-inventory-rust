use super::*;

#[test]
fn given_no_collection_when_derived_then_missing_items_are_deferred() -> Result {
    let mut value = given_model();
    set(&mut value, "/shapes/n#Output", "members", json!({}))?;
    then_deferred(
        derived(&value, &Vocabulary::default(), &DenyList::builtin())?,
        "items: no single list member",
    )
}

#[test]
fn given_map_collection_when_derived_then_values_supply_identity() -> Result {
    let mut value = given_model();
    set(&mut value, "/shapes", "n#Items", json!({"type":"map",
        "key":{"target":"smithy.api#String"},"value":{"target":"n#Thing"}}))?;
    then_callable(derived(&value, &Vocabulary::default(), &DenyList::builtin())?)?;
    Ok(())
}

#[test]
fn given_nonstring_scalar_when_derived_then_identity_is_deferred() -> Result {
    let mut value = given_model();
    set(&mut value, "/shapes/n#Items", "member", json!({"target":"smithy.api#Boolean"}))?;
    then_deferred(
        derived(&value, &Vocabulary::default(), &DenyList::builtin())?,
        "identity: no ARN member and no modeled identifiers",
    )
}

#[test]
fn given_unpaired_tokens_when_derived_then_each_suspicion_is_deferred() -> Result {
    for (side, reason) in [("Input", "TokenInputOnly"), ("Output", "TokenOutputOnly")] {
        let mut value = given_model();
        set(&mut value, &format!("/shapes/n#{side}/members"), "NextToken",
            json!({"target":"smithy.api#String"}))?;
        then_deferred(derived(&value, &vocabulary()?, &DenyList::builtin())?,
            &format!("paginator: {reason}"))?;
    }
    Ok(())
}

#[test]
fn given_missing_modeled_identifiers_when_derived_then_no_key_is_invented() -> Result {
    let mut value = given_model();
    set(&mut value, "/shapes/n#Thing", "members", json!({}))?;
    set(&mut value, "/shapes", "n#Resource", json!({"type":"resource",
        "list":{"target":"n#ListThings"},"identifiers":{"Id":{"target":"smithy.api#String"}}}))?;
    then_deferred(derived(&value, &Vocabulary::default(), &DenyList::builtin())?,
        "identity: no ARN member and no modeled identifiers")
}

#[test]
fn given_nonscalar_modeled_identifier_when_checked_then_dry_run_defers() -> Result {
    let mut value = given_model();
    set(&mut value, "/shapes/n#Thing", "members", json!({"Id":{"target":"n#Items"}}))?;
    set(&mut value, "/shapes", "n#Resource", json!({"type":"resource",
        "list":{"target":"n#ListThings"},"identifiers":{"Id":{"target":"smithy.api#String"}}}))?;
    let result = derived(&value, &Vocabulary::default(), &DenyList::builtin())?;
    assert!(result.is_err());
    assert!(result.err().is_some_and(|reason| reason.contains("identity")));
    Ok(())
}

#[test]
fn given_long_unicode_reason_when_bounded_then_utf8_remains_valid() {
    let reason = super::super::derive::bound_reason("é".repeat(600));
    assert_eq!(reason.len(), 1024);
}

#[test]
fn given_ambiguous_pairs_when_derived_then_no_pair_is_invented() -> Result {
    let mut value = given_model();
    add_tokens(&mut value)?;
    let model = Model::parse(&value, "sample")?;
    let pagination = crate::plans::Pagination::NeedsOverlay(
        crate::plans::PaginationReason::TokenBothSides,
    );
    let result = super::super::pagination::derive(
        &model, "ListThings", &pagination, &Vocabulary::default(),
    );
    assert_eq!(result, Err("paginator: ambiguous token pair".to_owned()));
    Ok(())
}

#[test]
fn given_two_known_pairs_when_derived_then_ambiguity_is_deferred() -> Result {
    let mut value = given_model();
    add_tokens(&mut value)?;
    let mut known = vocabulary()?;
    for side in ["Input", "Output"] {
        set(&mut value, &format!("/shapes/n#{side}/members"), "Marker",
            json!({"target":"smithy.api#String"}))?;
    }
    set(&mut value, "/shapes/n#ListThings", "traits", json!({"smithy.api#paginated":{
        "inputToken":"Marker","outputToken":"Marker","items":"Items"}}))?;
    known.record(&Model::parse(&value, "sample")?)?;
    set(&mut value, "/shapes/n#ListThings", "traits", json!({}))?;
    then_deferred(derived(&value, &known, &DenyList::builtin())?,
        "paginator: ambiguous token pair")
}
