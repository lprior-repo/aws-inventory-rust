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
            let text = format!(
                "{HEADER}[operation.ListThings]\nclass='enumerate'\npage_size={size}\n[operation.ListThings.identity]\nkey=['Id']\n"
            );
            overlay(root.path(), &text)?;
            if valid {
                let checked = overlay::check(root.path(), &source(root.path()))?;
                let callable = checked.services().first().ok_or("missing service")?.callable().first().ok_or("missing callable")?;
                assert_eq!(callable.plan().page_size, Some(size.parse::<i64>()?));
                assert_eq!(callable.plan().items, Some(ItemSelector::Member(overlay::MemberPath::new("Items")?)));
                let Some(overlay::Paginator::Token(pair)) = &callable.plan().paginator else {
                    return Err("missing inferred paginator".into());
                };
                assert_eq!(pair.input, overlay::MemberPath::new("Next")?);
                assert_eq!(pair.output, overlay::MemberPath::new("Next")?);
            } else {
                let errors = violations(root.path())?;
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
    for (page_size, expected) in [("page_size=500\n", 500), ("", 1000)] {
        let root = fixture(&model, "")?;
        overlay(
            root.path(),
            &format!(
                "{HEADER}[operation.ListThings]\nclass='enumerate'\n{page_size}[operation.ListThings.identity]\nkey=['Id']\n"
            ),
        )?;
        let checked = overlay::check(root.path(), &source(root.path()))?;
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
    overlay(
        root.path(),
        &format!(
            "{HEADER}[operation.ListThings]\nclass='enumerate'\n[operation.ListThings.identity]\nkey=['Id']\n"
        ),
    )?;
    let checked = overlay::check(root.path(), &source(root.path()))?;
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
