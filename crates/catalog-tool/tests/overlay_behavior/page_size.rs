use super::*;

const PAGE_SIZE_MODEL: &str = r#"{"smithy":"2.0","shapes":{
"n#Service":{"type":"service","operations":[{"target":"n#ListThings"}]},
"n#ListThings":{"type":"operation","input":{"target":"n#Input"},"output":{"target":"n#Output"},"traits":{"smithy.api#paginated":{"inputToken":"Next","outputToken":"Next","items":"Items","pageSize":"Limit"}}},
"n#Input":{"type":"structure","members":{"Next":{"target":"smithy.api#String"},"Limit":{"target":"n#Size","traits":{"smithy.api#range":{"min":1,"max":10}}}}},
"n#Output":{"type":"structure","members":{"Next":{"target":"smithy.api#String"},"Items":{"target":"n#Things"}}},
"n#Size":{"type":"integer","traits":{"smithy.api#range":{"min":1,"max":100}}},
"n#Things":{"type":"list","member":{"target":"n#Thing"}},
"n#Thing":{"type":"structure","members":{"Id":{"target":"smithy.api#String"}}}}}"#;

#[test]
fn modeled_page_size_obeys_member_range_at_and_inside_its_boundary()
-> Result<(), Box<dyn std::error::Error>> {
    let model = PAGE_SIZE_MODEL;
    [("1", true), ("5", true), ("10", true), ("11", false)]
        .into_iter()
        .try_for_each(|(size, valid)| -> Result<(), Box<dyn std::error::Error>> {
            let root = fixture(model, "")?;
            let mut plan = inferred()?;
            plan.page_size = Some(size.parse::<i64>()?);
            if valid {
                let checked = checked(&root, vec![("ListThings", plan)])?;
                let callable = checked.services().first().ok_or("missing service")?.callable().first().ok_or("missing callable")?;
                assert_eq!(callable.plan().page_size, Some(size.parse::<i64>()?));
                assert_eq!(callable.plan().items, Some(ItemSelector::Member(overlay::MemberPath::new("Items")?)));
                let Some(overlay::Paginator::Token(pair)) = &callable.plan().paginator else {
                    return Err("missing inferred paginator".into());
                };
                assert_eq!(pair.input, overlay::MemberPath::new("Next")?);
                assert_eq!(pair.output, overlay::MemberPath::new("Next")?);
            } else {
                let errors = violations(&root, vec![("ListThings", plan)])?;
                assert!(errors.iter().any(|error| error.key() == "page_size"
                    && error.value() == size
                    && error.rule() == ViolationRule::InvalidValue));
            }
            Ok(())
        })
}

#[test]
fn wider_member_range_precedes_narrower_target_range() -> Result<(), Box<dyn std::error::Error>> {
    let mut model: serde_json::Value = serde_json::from_str(PAGE_SIZE_MODEL)?;
    let member_max = model
        .pointer_mut("/shapes/n#Input/members/Limit/traits/smithy.api#range/max")
        .ok_or("missing member range maximum")?;
    *member_max = serde_json::json!(1000);
    let model = serde_json::to_string(&model)?;
    for (page_size, expected) in [(Some(500), 500), (None, 1000)] {
        let root = fixture(&model, "")?;
        let mut plan = inferred()?;
        plan.page_size = page_size;
        let checked = checked(&root, vec![("ListThings", plan)])?;
        let callable = checked
            .services()
            .first()
            .ok_or("missing service")?
            .callable()
            .first()
            .ok_or("missing callable")?;
        assert_eq!(callable.plan().page_size, Some(expected));
    }
    Ok(())
}

#[test]
fn inferred_page_size_satisfies_required_input_without_override()
-> Result<(), Box<dyn std::error::Error>> {
    let model = PAGE_SIZE_MODEL.replace(
        "\"Limit\":{\"target\":\"n#Size\",\"traits\":{\"smithy.api#range\":{\"min\":1,\"max\":10}}}",
        "\"Limit\":{\"target\":\"n#Size\",\"traits\":{\"smithy.api#required\":{},\"smithy.api#range\":{\"min\":1,\"max\":10}}}",
    );
    let root = fixture(&model, "")?;
    let checked = checked(&root, vec![("ListThings", inferred()?)])?;
    let callable = checked
        .services()
        .first()
        .ok_or("missing service")?
        .callable()
        .first()
        .ok_or("missing callable")?;
    assert_eq!(callable.plan().page_size, Some(10));
    Ok(())
}

#[test]
fn given_explicit_fixed_page_limit_when_model_infers_default_then_only_explicit_value_is_written()
-> Result<(), Box<dyn std::error::Error>> {
    let root = fixture(PAGE_SIZE_MODEL, "")?;
    let mut plan = inferred()?;
    plan.fixed.insert(overlay::MemberPath::new("Limit")?, overlay::FixedValue::Integer(5));
    let reviewed = checked(&root, vec![("ListThings", plan)])?;
    let plan = reviewed.services().first().ok_or("service")?.callable().first().ok_or("callable")?.plan();
    assert_eq!(plan.page_size, None);
    assert_eq!(plan.fixed.get(&overlay::MemberPath::new("Limit")?), Some(&overlay::FixedValue::Integer(5)));
    Ok(())
}

#[test]
fn given_modeled_string_page_limit_when_explicit_then_decimal_text_constraints_and_positive_bounds_apply()
-> Result<(), Box<dyn std::error::Error>> {
    let model = PAGE_SIZE_MODEL.replace("\"type\":\"integer\"", "\"type\":\"string\"");
    for value in [0, 1, 10, 11, 100_000, 100_001, i64::MAX] {
        let root = fixture(&model, "")?;
        let mut plan = inferred()?;
        plan.page_size = Some(value);
        match (value, checked(&root, vec![("ListThings", plan)])) {
            (1 | 10, Ok(when)) => {
                let plan = when.services().first().ok_or("service")?
                    .callable().first().ok_or("callable")?.plan();
                assert_eq!(plan.page_size, Some(value));
            }
            (0 | 100_001 | i64::MAX, Err(CheckError::Policy(error))) =>
                assert_eq!(error.value, "page_size: outside 1..=100000"),
            (11 | 100_000, Err(CheckError::Violations(errors))) =>
                assert!(errors.iter().any(|error| error.key() == "page_size"
                    && error.rule() == ViolationRule::InvalidValue)),
            (_, result) => return Err(format!("unexpected page limit result for {value}: {result:?}").into()),
        }
    }
    Ok(())
}
