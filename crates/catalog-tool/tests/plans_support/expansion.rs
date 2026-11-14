use super::{TestResult, binding, model, replace};
use catalog_tool::{
    classes::DenyList,
    plans::{Expansion, ExpansionReason, Model, expansion},
};
use serde_json::{Value, json};
use std::collections::BTreeMap;

fn derive<'a>(value: &'a Value, deny: &str) -> Result<Expansion<'a>, Box<dyn std::error::Error>> {
    let model = Model::parse(value, "svc")?;
    Ok(expansion(
        &model,
        "n#DescribeChild",
        &DenyList::parse(deny)?,
    )?)
}

#[test]
fn name_binding_ignores_optional_nonidentifiers_and_has_list_source() -> TestResult {
    assert_eq!(
        derive(&binding(), "")?,
        Expansion::Modeled {
            resource: "n#Resource",
            identifiers: BTreeMap::from([("Parent", "Parent")]),
            list_source: true,
            ambiguous_resource: false
        }
    );
    Ok(())
}

#[test]
fn resource_identifier_trait_binds_a_differently_named_required_member() -> TestResult {
    let mut value = binding();
    replace(
        &mut value,
        "/shapes/n#Input/members",
        json!({
        "InputParent":{"target":"smithy.api#String","traits":{
            "smithy.api#required":{},"smithy.api#resourceIdentifier":"Parent"}}}),
    )?;
    assert_eq!(
        derive(&value, "")?,
        Expansion::Modeled {
            resource: "n#Resource",
            identifiers: BTreeMap::from([("InputParent", "Parent")]),
            list_source: true,
            ambiguous_resource: false
        }
    );
    Ok(())
}

#[test]
fn required_nonidentifier_requires_overlay_even_when_resource_references_operation() -> TestResult {
    let mut value = binding();
    replace(
        &mut value,
        "/shapes/n#Input/members/Filter",
        json!({
        "target":"smithy.api#String","traits":{"smithy.api#required":{}}}),
    )?;
    assert_eq!(
        derive(&value, "")?,
        Expansion::NeedsOverlay(ExpansionReason::RequiredMemberNotIdentifier)
    );
    Ok(())
}

#[test]
fn absent_resource_reference_is_distinct_from_an_unbindable_reference() -> TestResult {
    let mut value = binding();
    replace(
        &mut value,
        "/shapes/n#Resource",
        json!({"type":"resource","identifiers":{
        "Parent":{"target":"smithy.api#String"}}}),
    )?;
    assert_eq!(
        derive(&value, "")?,
        Expansion::NeedsOverlay(ExpansionReason::NotInResource)
    );
    Ok(())
}

#[test]
fn all_four_resource_reference_locations_are_supported() -> TestResult {
    ["read", "list", "operations", "collectionOperations"]
        .into_iter()
        .try_for_each(|slot| {
            let mut value = binding();
            let reference = json!({"target":"n#DescribeChild"});
            let reference = if slot.ends_with("perations") {
                json!([reference])
            } else {
                reference
            };
            replace(
                &mut value,
                "/shapes/n#Resource",
                json!({"type":"resource",
            "identifiers":{"Parent":{"target":"smithy.api#String"}}, (slot):reference}),
            )?;
            assert!(matches!(
                derive(&value, "")?,
                Expansion::Modeled {
                    resource: "n#Resource",
                    ..
                }
            ));
            Ok::<_, Box<dyn std::error::Error>>(())
        })
}

#[test]
fn first_qualifying_resource_is_chosen_and_ambiguity_is_counted() -> TestResult {
    let mut value = binding();
    let resource = value
        .pointer("/shapes/n#Resource")
        .ok_or("resource missing")?
        .clone();
    let shapes = value
        .get_mut("shapes")
        .and_then(Value::as_object_mut)
        .ok_or("shapes missing")?;
    shapes.insert("n#AResource".to_owned(), resource);
    assert_eq!(
        derive(&value, "")?,
        Expansion::Modeled {
            resource: "n#AResource",
            identifiers: BTreeMap::from([("Parent", "Parent")]),
            list_source: true,
            ambiguous_resource: true
        }
    );
    Ok(())
}

#[test]
fn unqualified_earlier_resource_does_not_hide_a_later_binding() -> TestResult {
    let mut value = binding();
    let shapes = value
        .get_mut("shapes")
        .and_then(Value::as_object_mut)
        .ok_or("shapes missing")?;
    shapes.insert(
        "n#AResource".to_owned(),
        json!({"type":"resource","read":{"target":"n#DescribeChild"}}),
    );
    assert!(matches!(
        derive(&value, "")?,
        Expansion::Modeled {
            resource: "n#Resource",
            ambiguous_resource: false,
            ..
        }
    ));
    Ok(())
}

#[test]
fn missing_list_source_does_not_erase_the_modeled_binding() -> TestResult {
    let mut value = binding();
    replace(
        &mut value,
        "/shapes/n#Resource",
        json!({"type":"resource",
        "identifiers":{"Parent":{"target":"smithy.api#String"}},"read":{"target":"n#DescribeChild"}}),
    )?;
    assert!(matches!(
        derive(&value, "")?,
        Expansion::Modeled {
            list_source: false,
            ..
        }
    ));
    Ok(())
}

#[test]
fn denied_or_unclassified_list_source_cannot_supply_identifiers() -> TestResult {
    assert!(matches!(
        derive(&binding(), "ListRoots\tcontent\n")?,
        Expansion::Modeled {
            list_source: false,
            ..
        }
    ));
    let mut value = binding();
    replace(&mut value, "/shapes/n#ListRoots", json!({"type":"string"}))?;
    let shapes = value
        .get_mut("shapes")
        .and_then(Value::as_object_mut)
        .ok_or("shapes missing")?;
    shapes.insert("n#GetRoots".to_owned(), json!({"type":"operation"}));
    replace(
        &mut value,
        "/shapes/n#Resource/list/target",
        json!("n#GetRoots"),
    )?;
    assert!(matches!(
        derive(&value, "")?,
        Expansion::Modeled {
            list_source: false,
            ..
        }
    ));
    Ok(())
}

#[test]
fn expansion_class_list_source_is_also_a_modeled_identifier_source() -> TestResult {
    let mut value = binding();
    replace(
        &mut value,
        "/shapes/n#ListRoots",
        json!({"type":"operation","input":{"target":"n#Input"}}),
    )?;
    assert!(matches!(
        derive(&value, "")?,
        Expansion::Modeled {
            list_source: true,
            ..
        }
    ));
    Ok(())
}

#[test]
fn empty_resource_model_reports_not_in_resource() -> TestResult {
    let value = model(json!({"n#DescribeChild":{"type":"operation"}}));
    assert_eq!(
        derive(&value, "")?,
        Expansion::NeedsOverlay(ExpansionReason::NotInResource)
    );
    Ok(())
}
