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
    overlay(
        root.path(),
        &format!(
            "{HEADER}[operation.ListThings]\nclass='enumerate'\nitems='Items'\n[operation.ListThings.fixed]\nMissingOne='a'\nMissingTwo='b'\n[operation.ListThings.paginator]\nkind='none'\n[operation.ListThings.identity]\nkey=['Id']\n"
        ),
    )?;
    let errors = violations(root.path())?;
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
