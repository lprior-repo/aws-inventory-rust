use super::*;

#[test]
fn optional_primitive_boolean_input_does_not_block_a_callable_plan()
-> Result<(), Box<dyn std::error::Error>> {
    let model = BASIC
        .replace(
            "\"n#ListThings\":{\"type\":\"operation\",\"output\":",
            "\"n#ListThings\":{\"type\":\"operation\",\"input\":{\"target\":\"n#Input\"},\"output\":",
        )
        .replace(
            "\"n#Output\":",
            "\"n#Input\":{\"type\":\"structure\",\"members\":{\"IncludeAll\":{\"target\":\"smithy.api#PrimitiveBoolean\"}}},\"n#Output\":",
        );
    let root = fixture(&model, "")?;
    let checked = checked(&root, vec![("ListThings", complete()?)])?;
    let callable = checked
        .services()
        .first()
        .ok_or("missing service")?
        .callable()
        .first()
        .ok_or("missing callable")?;
    assert_eq!(callable.operation(), "ListThings");
    Ok(())
}
