use super::*;

#[test]
fn nested_list_token_values_must_have_compatible_element_types()
-> Result<(), Box<dyn std::error::Error>> {
    let model = r#"{"smithy":"2.0","shapes":{
"n#Service":{"type":"service","operations":[{"target":"n#ListThings"}]},
"n#ListThings":{"type":"operation","input":{"target":"n#Input"},"output":{"target":"n#Output"}},
"n#Input":{"type":"structure","members":{"Next":{"target":"n#StringMap"}}},
"n#Output":{"type":"structure","members":{"Next":{"target":"n#IntegerMap"},"Items":{"target":"n#Things"}}},
"n#StringMap":{"type":"map","key":{"target":"smithy.api#String"},"value":{"target":"n#StringList"}},
"n#IntegerMap":{"type":"map","key":{"target":"smithy.api#String"},"value":{"target":"n#IntegerList"}},
"n#StringList":{"type":"list","member":{"target":"smithy.api#String"}},
"n#IntegerList":{"type":"list","member":{"target":"smithy.api#Integer"}},
"n#Things":{"type":"list","member":{"target":"n#Thing"}},
"n#Thing":{"type":"structure","members":{"Id":{"target":"smithy.api#String"}}}}}"#;
    let root = fixture(model, "")?;
    overlay(
        root.path(),
        &format!(
            "{HEADER}[operation.ListThings]\nclass='enumerate'\nitems='Items'\n[operation.ListThings.paginator]\nkind='token'\ninput='Next'\noutput='Next'\n[operation.ListThings.identity]\nkey=['Id']\n"
        ),
    )?;
    let errors = violations(root.path())?;
    assert!(errors.iter().any(|error| error.key() == "paginator.token"
        && error.value() == "Next"
        && error.rule() == ViolationRule::InvalidValue));
    Ok(())
}
