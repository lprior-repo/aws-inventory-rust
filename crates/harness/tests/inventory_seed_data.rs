use inventory_core::record::{Float, Value};
use inventory_harness::inventory_seed::{
    FieldPath, SeedError, SeedInput, SeedInputValue, SeedOutputCheck, SeedScenario, SeedStep,
    StepId, default_scenario,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn single(value: SeedInputValue) -> Result<SeedStep, SeedError> {
    Ok(SeedStep::new(
        StepId::new(0),
        String::from("example"),
        String::from("CreateResource"),
        String::from("AWS::Example::Resource"),
        FieldPath::new(&["Resource", "Id"])?,
        vec![SeedInput::new(FieldPath::new(&["Name"])?, value)],
    )
    .with_expected_identity(String::from("exact-name"))
    .with_response_checks(vec![SeedOutputCheck::new(
        FieldPath::new(&["Resource", "Name"])?,
        FieldPath::new(&["Name"])?,
    )]))
}

#[test]
fn given_single_typed_resource_when_validated_then_no_ec2_relationship_is_required() -> TestResult {
    let scenario = SeedScenario::new(vec![single(SeedInputValue::Text(String::from(
        "exact-name",
    )))?])?;
    let step = scenario.steps().first().ok_or("missing step")?;
    assert_eq!(step.id, StepId::new(0));
    assert_eq!(step.expected_identity.as_deref(), Some("exact-name"));
    assert_eq!(step.response_checks.len(), 1);
    assert!(scenario.relationships().is_empty());
    Ok(())
}

#[test]
fn given_embedded_default_when_validated_then_all_six_response_dependencies_survive() -> TestResult
{
    let scenario = default_scenario()?;
    assert_eq!(scenario.steps().len(), 12);
    assert_eq!(scenario.relationships().len(), 6);
    scenario
        .relationships()
        .iter()
        .try_for_each(|relationship| {
            let source = scenario
                .steps()
                .get(usize::from(relationship.source.0))
                .ok_or("missing source")?;
            assert!(source.inputs.iter().any(|input| matches!(&input.value,
            SeedInputValue::PreviousOutput { step, path } if *step == relationship.target
                && path.parts().iter().map(String::as_str).eq(["Vpc", "VpcId"]))));
            assert_eq!(relationship.relation, "IN_VPC");
            Ok::<_, Box<dyn std::error::Error>>(())
        })
}

#[test]
fn given_invalid_typed_literals_when_validated_then_bounds_fail_closed() -> TestResult {
    let deep = (0..10).fold(Value::Int(1), |value, _| Value::List(vec![value]));
    [
        Value::Null,
        Value::Text("x".repeat(1_048_577)),
        Value::List(vec![Value::Int(1); 257]),
        deep,
        Value::Float(Float::new(f64::INFINITY)),
    ]
    .into_iter()
    .try_for_each(|value| {
        assert!(SeedScenario::new(vec![single(SeedInputValue::Literal(value))?]).is_err());
        Ok::<_, Box<dyn std::error::Error>>(())
    })
}

#[test]
fn given_text_at_byte_limit_when_validated_then_literal_and_text_inputs_are_accepted() -> TestResult
{
    for value in [
        SeedInputValue::Literal(Value::Text("x".repeat(1_048_576))),
        SeedInputValue::Text("x".repeat(1_048_576)),
    ] {
        assert!(SeedScenario::new(vec![single(value)?]).is_ok());
    }
    Ok(())
}

#[test]
fn given_missing_response_check_input_when_validated_then_scenario_is_rejected() -> TestResult {
    let step =
        single(SeedInputValue::Text(String::from("exact-name")))?.with_response_checks(vec![
            SeedOutputCheck::new(
                FieldPath::new(&["Resource", "Name"])?,
                FieldPath::new(&["Missing"])?,
            ),
        ]);
    assert!(SeedScenario::new(vec![step]).is_err());
    Ok(())
}

#[test]
fn given_empty_identity_or_noncontiguous_step_when_validated_then_scenario_is_rejected()
-> TestResult {
    let step = single(SeedInputValue::Text(String::from("exact-name")))?;
    assert!(SeedScenario::new(vec![step.clone().with_expected_identity(String::new())]).is_err());
    let mut invalid = step;
    invalid.id = StepId::new(1);
    assert!(SeedScenario::new(vec![invalid]).is_err());
    Ok(())
}
