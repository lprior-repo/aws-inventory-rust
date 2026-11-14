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
            overlay(root.path(), &format!(
                "{HEADER}[operation.ListThings]\nclass='enumerate'\nitems='Items'\n[operation.ListThings.fixed]\nScope='{value}'\n[operation.ListThings.paginator]\nkind='none'\n[operation.ListThings.identity]\nkey=['Id']\n"
            ))?;
            if valid {
                let checked = overlay::check(root.path(), &source(root.path()))?;
                assert_eq!(checked.services().first().ok_or("missing service")?.callable().len(), 1);
            } else {
                let errors = violations(root.path())?;
                assert!(errors.iter().any(|error| error.key() == "fixed"
                    && error.value() == "Scope"
                    && error.rule() == ViolationRule::InvalidValue));
            }
            Ok(())
        })
}
