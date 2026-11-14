use super::*;

#[test]
fn invalid_binding_paths_remain_distinct_in_the_aggregate() -> Result<(), Box<dyn std::error::Error>>
{
    let model = r#"{"smithy":"2.0","shapes":{
"n#Service":{"type":"service","operations":[{"target":"n#ListThings"},{"target":"n#GetThing"}]},
"n#ListThings":{"type":"operation","input":{"target":"n#Input"},"output":{"target":"n#Output"}},
"n#GetThing":{"type":"operation","output":{"target":"n#Output"}},
"n#Input":{"type":"structure","members":{"Id":{"target":"smithy.api#String"}}},
"n#Output":{"type":"structure","members":{"Id":{"target":"smithy.api#String"},"Items":{"target":"n#Things"}}},
"n#Things":{"type":"list","member":{"target":"n#Thing"}},
"n#Thing":{"type":"structure","members":{"Id":{"target":"smithy.api#String"}}}}}"#;
    let root = fixture(model, "")?;
    overlay(
        root.path(),
        &format!(
            "{HEADER}[operation.ListThings]\nclass='enumerate'\nitems='Items'\n[operation.ListThings.bind]\nMissingOne={{from='GetThing',path='Id'}}\nMissingTwo={{from='GetThing',path='Id'}}\n[operation.ListThings.paginator]\nkind='none'\n[operation.ListThings.identity]\nkey=['Id']\n[operation.GetThing]\nclass='expand'\nitems='Items'\n[operation.GetThing.paginator]\nkind='none'\n[operation.GetThing.identity]\nkey=['Id']\n"
        ),
    )?;
    let errors = violations(root.path())?;
    assert_eq!(
        errors
            .iter()
            .map(|error| (error.key(), error.value(), error.rule()))
            .collect::<Vec<_>>(),
        [
            ("bind", "MissingOne", ViolationRule::InvalidPath),
            ("bind", "MissingTwo", ViolationRule::InvalidPath),
        ]
    );
    Ok(())
}
