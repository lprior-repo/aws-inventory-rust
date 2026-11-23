use catalog_tool::{
    classes::DenyList,
    kind_map::{MappingSource, generate},
    names::Exceptions,
    overlay::{self, Identity, ItemSelector, MemberPath, OperationOverlay, OverlayClass, Paginator, Scope, ServiceOverlay},
};
use serde_json::{Value, json};
use std::{collections::BTreeMap, fs};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn given_model(scalar: bool) -> Value {
    let item = if scalar { "smithy.api#String" } else { "n#WidgetSummary" };
    json!({"smithy":"2.0","shapes":{
        "n#Service":{"type":"service","operations":[{"target":"n#ListWidgets"}],"traits":{
            "aws.api#service":{"sdkId":"Example Service","cloudFormationName":"Example"}}},
        "n#ListWidgets":{"type":"operation","input":{"target":"n#Input"},"output":{"target":"n#Output"}},
        "n#Input":{"type":"structure","members":{}},
        "n#Output":{"type":"structure","members":{"Widgets":{"target":"n#Widgets"}}},
        "n#Widgets":{"type":"list","member":{"target":item}},
        "n#WidgetSummary":{"type":"structure","members":{
            "Name":{"target":"smithy.api#String"},"Arn":{"target":"smithy.api#String"}}}
    }})
}

fn given_policy(scalar: bool) -> Result<ServiceOverlay> {
    let mut plan = OperationOverlay::default();
    plan.class = Some(OverlayClass::Enumerate);
    plan.scope = Some(Scope::Region);
    plan.items = Some(ItemSelector::parse("Widgets")?);
    plan.paginator = Some(Paginator::None);
    plan.identity = Some(if scalar {
        Identity::Key(vec![MemberPath::selected("$")?])
    } else {
        Identity::Arn(MemberPath::new("Arn")?)
    });
    Ok(ServiceOverlay::new("example", vec![("ListWidgets", plan)])?)
}

fn when_generated(scalar: bool) -> Result<generate::Generated> {
    let source = tempfile::tempdir()?;
    let directory = source.path().join("models/example/service/2026-01-01");
    fs::create_dir_all(&directory)?;
    let model = given_model(scalar);
    fs::write(directory.join("model.json"), serde_json::to_vec(&model)?)?;
    let checked = overlay::check_policies(source.path(), &[given_policy(scalar)?], &DenyList::builtin())?;
    let hand = MappingSource::default().compile(&Exceptions::none())?;
    generate::generate(&BTreeMap::from([("example".to_owned(), model)]), &checked, &hand,
        br#"{"resource_types":[{"type_name":"AWS::Example::Widget"}]}"#)
}

#[test]
fn kind_map_given_modeled_summary_when_generated_then_suffix_and_string_display_are_selected() -> Result {
    let generated = when_generated(false)?;
    assert!(generated.text.contains("cfn\tAWS::Example::Widget\tmap\tAWS::Example::Widget\tName\n"));
    assert!(generated.conflicts.is_empty());
    Ok(())
}

#[test]
fn kind_map_given_scalar_list_when_generated_then_resource_comes_from_member_name() -> Result {
    let generated = when_generated(true)?;
    assert!(generated.text.contains("api\tAWS::ApiOnly::ExampleServiceWidgets\tkind\tgenerated: no exact CloudFormation type\t-\n"));
    assert!(generated.text.contains("operation\texample\tListWidgets\tAWS::ApiOnly::ExampleServiceWidgets\n"));
    Ok(())
}
