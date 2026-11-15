use super::*;

#[test]
fn acyclic_binding_from_callable_parent_is_materialized() -> Result<(), Box<dyn std::error::Error>>
{
    let model = BASIC
        .replace(
            "\"n#ListThings\"}]",
            "\"n#ListThings\"},{\"target\":\"n#GetThing\"}]",
        )
        .replace(
            "\"n#ListThings\":{\"type\":\"operation\"",
            "\"n#GetThing\":{\"type\":\"operation\",\"input\":{\"target\":\"n#Input\"},\"output\":{\"target\":\"n#Output\"}},\"n#Input\":{\"type\":\"structure\",\"members\":{\"Id\":{\"target\":\"smithy.api#String\",\"traits\":{\"smithy.api#required\":{}}}}},\"n#ListThings\":{\"type\":\"operation\"",
        );
    let root = fixture(&model, "")?;
    let mut plan = complete()?;
    plan.class = Some(OverlayClass::Expand);
    plan.bind.insert(overlay::MemberPath::new("Id")?, binding("ListThings", "Id")?);
    let checked = checked(&root, vec![("ListThings", complete()?), ("GetThing", plan)])?;
    let child = checked
        .services()
        .first()
        .ok_or("missing service")?
        .callable()
        .iter()
        .find(|callable| callable.operation() == "GetThing")
        .ok_or("missing child call")?;
    assert_eq!(child.plan().bind.len(), 1);
    assert_eq!(child.class(), EffectiveClass::Expand);
    Ok(())
}
