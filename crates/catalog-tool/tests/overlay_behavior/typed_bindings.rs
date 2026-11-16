use super::*;
use overlay::{Binding, BindingAlternative, ContextSource, MemberPath, OperationName};
type TestResult = Result<(), Box<dyn std::error::Error>>;

const MODEL: &str = r#"{"smithy":"2.0","shapes":{
"n#Service":{"type":"service","operations":[{"target":"n#ListA"},{"target":"n#ListB"},{"target":"n#ListChildren"}]},
"n#ListA":{"type":"operation","output":{"target":"n#Output"}},
"n#ListB":{"type":"operation","output":{"target":"n#Output"}},
"n#ListChildren":{"type":"operation","input":{"target":"n#Input"},"output":{"target":"n#Output"}},
"n#Input":{"type":"structure","members":{"Id":{"target":"smithy.api#String","traits":{"smithy.api#required":{}}},"OtherId":{"target":"smithy.api#String"},"Regions":{"target":"n#Strings"}}},
"n#Output":{"type":"structure","members":{"Items":{"target":"n#Things"}}},
"n#Things":{"type":"list","member":{"target":"n#Thing"}},
"n#Thing":{"type":"structure","members":{"Id":{"target":"smithy.api#String"},"Names":{"target":"n#Strings"},"Count":{"target":"smithy.api#Integer"},"Enabled":{"target":"smithy.api#Boolean"}}},
"n#Strings":{"type":"list","member":{"target":"smithy.api#String"}}
}}"#;

fn alternative(operation: &str, path: &str) -> Result<BindingAlternative, overlay::NameError> {
    Ok(BindingAlternative { operation: OperationName::new(operation)?, source: ContextSource::SelectedItem,
        path: MemberPath::new(path)? })
}
fn child(alternatives: Vec<BindingAlternative>) -> Result<OperationOverlay, overlay::NameError> {
    let mut plan = complete()?;
    plan.class = Some(OverlayClass::Expand);
    plan.bind.insert(MemberPath::new("Id")?, Binding::FromAny(alternatives));
    Ok(plan)
}
fn check_child(plan: OperationOverlay) -> Result<overlay::CheckedOverlays, Box<dyn std::error::Error>> {
    let given = fixture(MODEL, "")?;
    Ok(checked(&given, vec![("ListA", complete()?), ("ListB", complete()?), ("ListChildren", plan)])?)
}

#[test]
fn given_disjoint_producers_when_each_alternative_resolves_then_one_child_binding_is_accepted() -> TestResult {
    let plan = child(vec![alternative("ListA", "Id")?, alternative("ListB", "Id")?])?;
    let when = check_child(plan)?;
    assert_eq!(when.services().first().ok_or("service")?.callable().len(), 3);
    Ok(())
}

#[test]
fn given_alternative_sources_when_empty_duplicate_unmodeled_or_incompatible_then_no_plan_is_accepted() -> TestResult {
    for alternatives in [vec![], vec![alternative("ListA", "Id")?, alternative("ListA", "Id")?],
        vec![alternative("ListA", "Id")?, alternative("ListB", "Missing")?],
        vec![alternative("ListA", "Id")?, alternative("ListB", "Count")?]] {
        assert!(check_child(child(alternatives)?).is_err());
    }
    Ok(())
}

#[test]
fn given_multiple_alternative_inputs_when_producer_sets_differ_then_cross_product_cannot_be_invented() -> TestResult {
    let mut plan = child(vec![alternative("ListA", "Id")?, alternative("ListB", "Id")?])?;
    plan.bind.insert(MemberPath::new("OtherId")?, Binding::FromAny(vec![alternative("ListA", "Id")?]));
    assert!(check_child(plan).is_err());
    Ok(())
}

#[test]
fn given_region_list_binding_when_target_is_list_of_text_then_scalar_coercion_is_rejected() -> TestResult {
    let mut plan = child(vec![alternative("ListA", "Id")?])?;
    plan.bind.insert(MemberPath::new("Regions")?, Binding::RegionList);
    check_child(plan)?;
    let mut invalid = child(vec![alternative("ListA", "Id")?])?;
    invalid.bind.insert(MemberPath::new("OtherId")?, Binding::RegionList);
    assert!(check_child(invalid).is_err());
    Ok(())
}

#[test]
fn given_typed_identity_fields_when_shapes_and_bounds_match_then_checker_preserves_the_exact_contract() -> TestResult {
    let given = fixture(MODEL, "")?;
    let mut plan = complete()?;
    let fields = vec![overlay::IdentityField::SelectedInteger { path: MemberPath::new("Count")?, minimum: 0, maximum: 100 },
        overlay::IdentityField::SelectedStringList { path: MemberPath::new("Names")?, max_items: 8, max_item_bytes: 128 },
        overlay::IdentityField::SelectedDigest(MemberPath::selected("$")?)];
    plan.identity = Some(overlay::Identity::CorrelatedKey(fields.clone()));
    let when = checked(&given, vec![("ListA", plan), ("ListB", deferred()), ("ListChildren", deferred())])?;
    assert_eq!(when.services().first().ok_or("service")?.callable().first().ok_or("callable")?.plan().identity,
        Some(overlay::Identity::CorrelatedKey(fields)));
    Ok(())
}

#[test]
fn given_typed_identity_fields_when_shape_or_bounds_are_invalid_then_plan_is_rejected() -> TestResult {
    let given = fixture(MODEL, "")?;
    for field in [
        (overlay::IdentityField::SelectedInteger { path: MemberPath::new("Id")?, minimum: 0, maximum: 10 }, None),
        (overlay::IdentityField::SelectedInteger { path: MemberPath::new("Count")?, minimum: 11, maximum: 10 }, Some("identity integer bounds")),
        (overlay::IdentityField::SelectedInteger { path: MemberPath::new("Count")?, minimum: 0, maximum: i64::MAX }, None),
        (overlay::IdentityField::SelectedStringList { path: MemberPath::new("Names")?, max_items: 0, max_item_bytes: 1 }, Some("identity string list bounds")),
        (overlay::IdentityField::SelectedStringList { path: MemberPath::new("Names")?, max_items: 65, max_item_bytes: 128 }, Some("identity string list bounds")),
        (overlay::IdentityField::SelectedStringList { path: MemberPath::new("Id")?, max_items: 1, max_item_bytes: 1 }, None),
        (overlay::IdentityField::SelectedDigest(MemberPath::new("Missing")?), None),
    ] {
        let (field, policy_error) = field;
        let mut plan = complete()?;
        plan.identity = Some(overlay::Identity::CorrelatedKey(vec![field]));
        match (checked(&given, vec![("ListA", plan), ("ListB", deferred()), ("ListChildren", deferred())]), policy_error) {
            (Err(CheckError::Policy(error)), Some(expected)) => assert_eq!(error.value, expected),
            (Err(CheckError::Violations(errors)), None) => assert!(errors.iter()
                .any(|error| error.operation() == "ListA" && error.key() == "identity")),
            (result, expected) => return Err(format!("unexpected identity result {result:?}, expected {expected:?}").into()),
        }
    }
    Ok(())
}

#[test]
fn given_scalar_presence_predicates_when_checked_then_numeric_and_boolean_presence_are_not_text_comparisons() -> TestResult {
    let given = fixture(MODEL, "")?;
    for (path, present, accepted) in [
        ("Id", true, true), ("Count", true, true), ("Enabled", true, true),
        ("Id", false, true), ("Count", false, false), ("Enabled", false, false),
        ("Names", true, false), ("Missing", true, false),
    ] {
        let predicate = if present {
            overlay::IdentityPredicate::Present(MemberPath::new(path)?)
        } else {
            overlay::IdentityPredicate::TextEquals { path: MemberPath::new(path)?, value: "value".into() }
        };
        let identity = overlay::Identity::Choice(overlay::IdentityChoice::Discriminated(vec![
            overlay::IdentityBranch::new(predicate, overlay::IdentityLeaf::Key(vec![MemberPath::new("Id")?]))?,
        ]));
        let mut plan = complete()?;
        plan.identity = Some(identity.clone());
        let result = checked(&given, vec![("ListA", plan), ("ListB", deferred()), ("ListChildren", deferred())]);
        match (result, accepted) {
            (Ok(when), true) => assert_eq!(when.services().first().ok_or("service")?
                .callable().first().ok_or("callable")?.plan().identity, Some(identity)),
            (Err(CheckError::Violations(errors)), false) => assert!(errors.iter()
                .any(|error| error.operation() == "ListA" && error.key() == "identity")),
            (result, _) => return Err(format!("unexpected predicate result for {path}: {result:?}").into()),
        }
    }
    Ok(())
}
