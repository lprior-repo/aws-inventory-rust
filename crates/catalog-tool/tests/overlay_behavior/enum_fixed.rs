use super::*;

#[test]
fn required_enum_inputs_accept_only_declared_fixed_values() -> Result<(), Box<dyn std::error::Error>>
{
    let model = r#"{"smithy":"2.0","shapes":{
"n#Service":{"type":"service","operations":[{"target":"n#ListThings"}]},
"n#ListThings":{"type":"operation","input":{"target":"n#Input"},"output":{"target":"n#Output"}},
"n#Input":{"type":"structure","members":{"Scope":{"target":"n#Scope","traits":{"smithy.api#required":{}}}}},
"n#Scope":{"type":"enum","members":{"REGIONAL":{"target":"smithy.api#Unit","traits":{"smithy.api#enumValue":"REGIONAL"}}}},
"n#Output":{"type":"structure","members":{"Items":{"target":"n#Things"}}},
"n#Things":{"type":"list","member":{"target":"n#Thing"}},
"n#Thing":{"type":"structure","members":{"Id":{"target":"smithy.api#String"}}}}}"#;
    [("REGIONAL", true), ("GLOBAL", false)]
        .into_iter()
        .try_for_each(|(value, valid)| -> Result<(), Box<dyn std::error::Error>> {
            let root = fixture(model, "")?;
            let mut plan = complete()?;
            plan.fixed.insert(
                overlay::MemberPath::new("Scope")?,
                overlay::FixedValue::String(value.to_owned()),
            );
            if valid {
                let checked = checked(&root, vec![("ListThings", plan)])?;
                assert_eq!(
                    checked
                        .services()
                        .first()
                        .ok_or("missing service")?
                        .callable()
                        .len(),
                    1
                );
            } else {
                let errors = violations(&root, vec![("ListThings", plan)])?;
                assert!(errors.iter().any(|error| error.key() == "fixed"
                    && error.value() == "Scope"
                    && error.rule() == ViolationRule::InvalidValue));
            }
            Ok(())
        })
}
