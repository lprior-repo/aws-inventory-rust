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
    let mut plan = complete()?;
    plan.paginator = Some(token("Next", "Next")?);
    let errors = violations(&root, vec![("ListThings", plan)])?;
    assert!(errors.iter().any(|error| error.key() == "paginator.token"
        && error.value() == "Next"
        && error.rule() == ViolationRule::InvalidValue));
    Ok(())
}
