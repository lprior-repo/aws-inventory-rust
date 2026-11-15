use super::*;

#[test]
fn invalid_fixed_paths_remain_distinct_in_the_aggregate() -> Result<(), Box<dyn std::error::Error>>
{
    let model = BASIC
        .replace(
            "\"n#ListThings\":{\"type\":\"operation\",\"output\":",
            "\"n#ListThings\":{\"type\":\"operation\",\"input\":{\"target\":\"n#Input\"},\"output\":",
        )
        .replace(
            "\"n#Output\":",
            "\"n#Input\":{\"type\":\"structure\",\"members\":{\"Id\":{\"target\":\"smithy.api#String\"}}},\"n#Output\":",
        );
    let root = fixture(&model, "")?;
    let mut plan = complete()?;
    plan.fixed.insert(
        overlay::MemberPath::new("MissingOne")?,
        overlay::FixedValue::String("a".to_owned()),
    );
    plan.fixed.insert(
        overlay::MemberPath::new("MissingTwo")?,
        overlay::FixedValue::String("b".to_owned()),
    );
    let errors = violations(&root, vec![("ListThings", plan)])?;
    assert_eq!(
        errors
            .iter()
            .map(|error| (error.key(), error.value(), error.rule()))
            .collect::<Vec<_>>(),
        [
            ("fixed", "MissingOne", ViolationRule::InvalidPath),
            ("fixed", "MissingTwo", ViolationRule::InvalidPath),
        ]
    );
    Ok(())
}
