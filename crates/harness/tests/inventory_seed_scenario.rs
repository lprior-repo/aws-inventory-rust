use inventory_harness::inventory_seed::{
    FieldPath, SeedInput, SeedInputValue, SeedScenario, SeedStep, StepId, default_scenario,
};

#[test]
fn default_scenario_describes_vpc_subnet_mutations_and_observed_relationships()
-> Result<(), Box<dyn std::error::Error>> {
    let scenario = default_scenario()?;
    assert_eq!(scenario.steps().len(), 12);
    assert_eq!(scenario.relationships().len(), 6);
    let first = scenario.steps().first().ok_or("missing VPC step")?;
    let second = scenario.steps().get(1).ok_or("missing subnet step")?;
    assert_eq!(first.operation, "CreateVpc");
    assert_eq!(first.kind, "AWS::EC2::VPC");
    assert_eq!(second.operation, "CreateSubnet");
    assert_eq!(second.kind, "AWS::EC2::Subnet");
    let binding = second
        .inputs
        .iter()
        .find(|input| {
            input
                .field
                .parts()
                .first()
                .is_some_and(|field| field == "VpcId")
        })
        .ok_or("missing VPC output binding")?;
    assert!(matches!(
        &binding.value,
        SeedInputValue::PreviousOutput { step, path }
            if *step == StepId::new(0)
                && path.parts().iter().map(String::as_str).eq(["Vpc", "VpcId"])
    ));
    let checked = SeedScenario::with_relationships(
        scenario.steps().to_vec(),
        scenario.relationships().to_vec(),
    )?;
    assert_eq!(checked.steps().len(), 12);
    Ok(())
}

#[test]
fn scenario_rejects_forward_dependency_cycles_and_invalid_relationships()
-> Result<(), Box<dyn std::error::Error>> {
    let output = FieldPath::new(&["Output", "Id"])?;
    let first_binding = SeedInput::new(
        FieldPath::new(&["ResourceId"])?,
        SeedInputValue::PreviousOutput {
            step: StepId::new(1),
            path: output.clone(),
        },
    );
    let second_binding = SeedInput::new(
        FieldPath::new(&["OtherId"])?,
        SeedInputValue::PreviousOutput {
            step: StepId::new(0),
            path: output.clone(),
        },
    );
    let forward = vec![
        step(0, "CreateA", vec![first_binding])?,
        step(1, "CreateB", Vec::new())?,
    ];
    assert!(SeedScenario::new(forward).is_err());
    let cycle = vec![
        step(
            0,
            "CreateA",
            vec![SeedInput::new(
                FieldPath::new(&["ResourceId"])?,
                SeedInputValue::PreviousOutput {
                    step: StepId::new(1),
                    path: output.clone(),
                },
            )],
        )?,
        step(1, "CreateB", vec![second_binding])?,
    ];
    assert!(SeedScenario::new(cycle).is_err());
    let relationship = inventory_harness::inventory_seed::SeedRelationship::new(
        StepId::new(0),
        StepId::new(0),
        String::from("RELATED_TO"),
        output,
    );
    assert!(
        SeedScenario::with_relationships(
            vec![
                step(0, "CreateA", Vec::new())?,
                step(1, "CreateB", Vec::new())?
            ],
            vec![relationship],
        )
        .is_err()
    );
    Ok(())
}

#[test]
fn scenario_enforces_step_binding_and_path_limits() -> Result<(), Box<dyn std::error::Error>> {
    assert!(FieldPath::new(&["Vpc-Id"]).is_err());
    assert!(FieldPath::new(&["a", "b", "c", "d", "e", "f", "g", "h", "i"]).is_err());
    let oversized = (0_u8..13)
        .map(|id| step(id, "CreateResource", Vec::new()))
        .collect::<Result<Vec<_>, _>>()?;
    assert!(SeedScenario::new(oversized).is_err());
    let inputs = (0_u8..9)
        .map(|_| {
            Ok(SeedInput::new(
                FieldPath::new(&["Field"])?,
                SeedInputValue::Text(String::from("value")),
            ))
        })
        .collect::<Result<Vec<_>, inventory_harness::inventory_seed::SeedError>>()?;
    assert!(
        SeedScenario::new(vec![
            step(0, "CreateA", inputs)?,
            step(1, "CreateB", Vec::new())?
        ])
        .is_err()
    );
    let overlapping = vec![
        SeedInput::new(
            FieldPath::new(&["Options"])?,
            SeedInputValue::Text(String::from("x")),
        ),
        SeedInput::new(
            FieldPath::new(&["Options", "Name"])?,
            SeedInputValue::Text(String::from("y")),
        ),
    ];
    assert!(
        SeedScenario::new(vec![
            step(0, "CreateA", overlapping)?,
            step(1, "CreateB", Vec::new())?,
        ])
        .is_err()
    );
    Ok(())
}

fn step(
    id: u8,
    operation: &str,
    inputs: Vec<SeedInput>,
) -> Result<SeedStep, Box<dyn std::error::Error>> {
    Ok(SeedStep::new(
        StepId::new(id),
        String::from("ec2"),
        String::from(operation),
        String::from("AWS::EC2::Resource"),
        FieldPath::new(&["Result", "Id"])?,
        inputs,
    ))
}
