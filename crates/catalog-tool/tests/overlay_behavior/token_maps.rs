use super::*;

#[test]
fn map_token_paths_remain_typed_and_callable() -> Result<(), Box<dyn std::error::Error>> {
    let model = r#"{"smithy":"2.0","shapes":{
"n#Service":{"type":"service","operations":[{"target":"n#ListThings"}]},
"n#ListThings":{"type":"operation","input":{"target":"n#Input"},"output":{"target":"n#Output"}},
"n#Input":{"type":"structure","members":{"Next":{"target":"n#TokenMap"}}},
"n#Output":{"type":"structure","members":{"Next":{"target":"n#TokenMap"},"Items":{"target":"n#Things"}}},
"n#TokenMap":{"type":"map","key":{"target":"smithy.api#String"},"value":{"target":"smithy.api#String"}},
"n#Things":{"type":"list","member":{"target":"n#Thing"}},
"n#Thing":{"type":"structure","members":{"Id":{"target":"smithy.api#String"}}}
}}"#;
    let root = fixture(model, "")?;
    let mut plan = complete()?;
    plan.paginator = Some(token("Next", "Next")?);
    let checked = checked(&root, vec![("ListThings", plan)])?;
    let service = checked.services().first().ok_or("missing service")?;
    let callable = service
        .callable()
        .first()
        .ok_or("map token plan not callable")?;
    let Some(overlay::Paginator::Token(pair)) = &callable.plan().paginator else {
        return Err("expected token paginator".into());
    };
    assert_eq!(pair.input, overlay::MemberPath::new("Next")?);
    assert_eq!(pair.output, overlay::MemberPath::new("Next")?);
    Ok(())
}

#[test]
fn item_identity_paths_traverse_nested_lists_and_map_values()
-> Result<(), Box<dyn std::error::Error>> {
    let model = r#"{"smithy":"2.0","shapes":{
"n#Service":{"type":"service","operations":[{"target":"n#ListThings"}]},
"n#ListThings":{"type":"operation","output":{"target":"n#Output"}},
"n#Output":{"type":"structure","members":{"Items":{"target":"n#Outer"}}},
"n#Outer":{"type":"list","member":{"target":"n#Index"}},
"n#Index":{"type":"map","key":{"target":"smithy.api#String"},"value":{"target":"n#Inner"}},
"n#Inner":{"type":"list","member":{"target":"n#Thing"}},
"n#Thing":{"type":"structure","members":{"Id":{"target":"smithy.api#String"}}}
}}"#;
    let root = fixture(model, "")?;
    let mut plan = complete()?;
    plan.exclude_from_hash = vec![overlay::MemberPath::new("Id")?];
    plan.queryable = vec![overlay::MemberPath::new("Id")?];
    plan.tags = Some(overlay::MemberPath::new("Id")?);
    let checked = checked(&root, vec![("ListThings", plan)])?;
    let callable = checked
        .services()
        .first()
        .ok_or("missing service")?
        .callable()
        .first()
        .ok_or("nested aggregate plan not callable")?;
    assert_eq!(callable.operation(), "ListThings");
    Ok(())
}
