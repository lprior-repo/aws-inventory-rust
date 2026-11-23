use crate::{
    overlay::{
        Identity, ItemSelector, MemberPath,
        generate::{identity, items},
    },
    plans::{Model, Pagination},
};
use serde_json::{Value, json};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const OPERATION: &str = "DescribeLocationAzureBlob";

fn given_model(operation: &str, required: bool, matching: bool) -> Value {
    let traits = if required {
        json!({"smithy.api#required":{}})
    } else {
        json!({})
    };
    let input_type = if matching {
        "n#LocationArn"
    } else {
        "n#OtherArn"
    };
    json!({"smithy":"2.0","shapes":{
        "n#Service":{"type":"service","operations":[{"target":format!("n#{operation}")}]},
        format!("n#{operation}"):{"type":"operation","input":{"target":"n#Input"},"output":{"target":"n#Output"}},
        "n#Input":{"type":"structure","members":{"LocationArn":{"target":input_type,"traits":traits}}},
        "n#Output":{"type":"structure","members":{"LocationArn":{"target":"n#LocationArn"},"AgentArns":{"target":"n#Arns"}}},
        "n#LocationArn":{"type":"string"},"n#OtherArn":{"type":"string"},
        "n#Arns":{"type":"list","member":{"target":"n#LocationArn"}}
    }})
}

#[test]
fn given_required_location_identity_when_selected_then_root_wins_over_agent_references() -> Result {
    let value = given_model(OPERATION, true, true);
    let model = Model::parse(&value, "sample")?;
    let pagination = Pagination::Unpaginated;
    let selected = items::expand(&model, OPERATION, &pagination)?;
    assert_eq!(selected, Some(ItemSelector::Root));
    assert_eq!(
        identity::derive(&model, OPERATION, selected.as_ref(), &pagination)?,
        Identity::Arn(MemberPath::new("LocationArn")?)
    );
    Ok(())
}

#[test]
fn given_optional_or_different_shape_when_root_identity_is_derived_then_name_alone_is_insufficient()
-> Result {
    for (required, matching) in [(false, true), (true, false)] {
        let value = given_model(OPERATION, required, matching);
        let model = Model::parse(&value, "sample")?;
        assert!(
            identity::derive(
                &model,
                OPERATION,
                Some(&ItemSelector::Root),
                &Pagination::Unpaginated
            )
            .is_err()
        );
    }
    Ok(())
}

#[test]
fn given_list_operation_when_root_identity_is_derived_then_request_identity_is_not_reused() -> Result
{
    let operation = "ListLocationDetails";
    let value = given_model(operation, true, true);
    let model = Model::parse(&value, "sample")?;
    assert!(
        identity::derive(
            &model,
            operation,
            Some(&ItemSelector::Root),
            &Pagination::Unpaginated
        )
        .is_err()
    );
    Ok(())
}

#[test]
fn given_selected_collection_when_identity_is_derived_then_request_identity_is_not_reused() -> Result
{
    let value = given_model(OPERATION, true, true);
    let model = Model::parse(&value, "sample")?;
    let selected = ItemSelector::parse("AgentArns")?;
    assert_eq!(
        identity::derive(&model, OPERATION, Some(&selected), &Pagination::Unpaginated)?,
        Identity::Key(vec![MemberPath::selected("$")?])
    );
    Ok(())
}
