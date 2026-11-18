use inventory_core::record::{Key, Value};
use inventory_harness::inventory_seed::{
    FieldPath, SeedError, SeedIdentity, SeedInput, SeedInputValue, SeedNestedBinding,
    SeedRelationship, SeedScenario, SeedScope, SeedStep, SeedTemplate, SeedValuePart, StepId,
};
use std::collections::BTreeMap;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn template() -> Result<SeedTemplate, SeedError> {
    Ok(SeedTemplate {
        value: Value::Map(BTreeMap::from([
            (
                Key::Name(String::from("Members")),
                Value::List(vec![Value::Null]),
            ),
            (Key::Name(String::from("Enabled")), Value::Bool(true)),
        ])),
        bindings: vec![SeedNestedBinding {
            destination: vec![
                SeedValuePart::Field(String::from("Members")),
                SeedValuePart::Index(0),
            ],
            step: StepId::new(0),
            path: FieldPath::new(&["Items", "0", "Id"])?,
        }],
    })
}

fn flow(template: SeedTemplate) -> Result<SeedScenario, SeedError> {
    SeedScenario::new(vec![
        SeedStep::auxiliary(
            StepId::new(0),
            String::from("example"),
            String::from("DescribeParents"),
            Vec::new(),
        ),
        SeedStep::new(
            StepId::new(1),
            String::from("example"),
            String::from("CreateChild"),
            String::from("AWS::Example::Child"),
            FieldPath::new(&["Id"])?,
            vec![SeedInput::new(
                FieldPath::new(&["Configuration"])?,
                SeedInputValue::Template(template),
            )],
        ),
    ])
}

fn request_identity(template: Option<String>) -> Result<SeedStep, SeedError> {
    Ok(SeedStep::new(
        StepId::new(0),
        String::from("s3"),
        String::from("CreateBucket"),
        String::from("AWS::S3::Bucket"),
        SeedIdentity::Request {
            input_field: FieldPath::new(&["Bucket"])?,
            template,
        },
        vec![SeedInput::new(
            FieldPath::new(&["Bucket"])?,
            SeedInputValue::Text(String::from("inventory-test")),
        )],
    )
    .with_scope(SeedScope::Account))
}

#[test]
fn given_nested_typed_binding_when_validated_then_earlier_auxiliary_response_can_supply_it()
-> TestResult {
    let scenario = flow(template()?)?;
    assert_eq!(scenario.steps().len(), 2);
    assert!(
        scenario
            .steps()
            .first()
            .ok_or("missing auxiliary")?
            .identity
            .is_none()
    );
    assert!(scenario.relationships().is_empty());
    Ok(())
}

#[test]
fn given_forward_nested_binding_when_validated_then_scenario_is_rejected() -> TestResult {
    let mut value = template()?;
    value.bindings.first_mut().ok_or("missing binding")?.step = StepId::new(1);
    assert!(flow(value).is_err());
    Ok(())
}

#[test]
fn given_missing_duplicate_or_wrong_placeholder_when_validated_then_scenario_is_rejected()
-> TestResult {
    let mut missing = template()?;
    missing.bindings.clear();
    assert!(flow(missing).is_err());
    let mut duplicate = template()?;
    duplicate
        .bindings
        .push(duplicate.bindings.first().ok_or("missing binding")?.clone());
    assert!(flow(duplicate).is_err());
    let mut wrong = template()?;
    wrong
        .bindings
        .first_mut()
        .ok_or("missing binding")?
        .destination = vec![SeedValuePart::Field(String::from("Enabled"))];
    assert!(flow(wrong).is_err());
    Ok(())
}

#[test]
fn given_account_request_identity_when_validated_then_explicit_source_and_scope_are_retained()
-> TestResult {
    let scenario = SeedScenario::new(vec![request_identity(Some(String::from(
        "arn:{partition}:s3:::{value}",
    )))?])?;
    let step = scenario.steps().first().ok_or("missing resource")?;
    assert_eq!(step.scope, SeedScope::Account);
    assert!(
        matches!(&step.identity, Some(SeedIdentity::Request { input_field, template: Some(template) })
        if input_field.parts().iter().map(String::as_str).eq(["Bucket"])
            && template == "arn:{partition}:s3:::{value}")
    );
    Ok(())
}

#[test]
fn given_invalid_identity_template_when_validated_then_scenario_is_rejected() -> TestResult {
    [
        "arn:aws:s3:::fixed",
        "arn:{missing}:s3:::{value}",
        "arn:{value",
        "arn:{value}}",
    ]
    .into_iter()
    .try_for_each(|value| {
        assert!(SeedScenario::new(vec![request_identity(Some(value.to_owned()))?]).is_err());
        Ok::<_, Box<dyn std::error::Error>>(())
    })
}

#[test]
fn given_auxiliary_relationship_endpoint_when_validated_then_no_resource_edge_is_accepted()
-> TestResult {
    let scenario = flow(template()?)?;
    let relationship = SeedRelationship::new(
        StepId::new(1),
        StepId::new(0),
        String::from("RELATED_TO"),
        FieldPath::new(&["ParentId"])?,
    );
    assert!(
        SeedScenario::with_relationships(scenario.steps().to_vec(), vec![relationship]).is_err()
    );
    Ok(())
}
