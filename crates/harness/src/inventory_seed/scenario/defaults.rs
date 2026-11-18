use super::*;

pub(super) fn scenario() -> Result<SeedScenario, SeedError> {
    let pairs = [
        (0, 1, "10.88.1.0/24"),
        (2, 3, "10.88.2.0/24"),
        (4, 5, "10.88.3.0/24"),
        (6, 7, "10.88.4.0/24"),
        (8, 9, "10.88.5.0/24"),
        (10, 11, "10.88.6.0/24"),
    ];
    let initial = (Vec::with_capacity(12), Vec::with_capacity(6));
    let (steps, relationships) = pairs.into_iter().try_fold(
        initial,
        |(mut steps, mut relationships), (vpc, subnet, cidr)| {
            steps.push(vpc_step(vpc)?);
            steps.push(subnet_step(subnet, vpc, cidr)?);
            relationships.push(SeedRelationship::new(
                StepId::new(subnet),
                StepId::new(vpc),
                String::from("IN_VPC"),
                FieldPath::new(&["Subnet", "VpcId"])?,
            ));
            Ok::<_, SeedError>((steps, relationships))
        },
    )?;
    SeedScenario::with_relationships(steps, relationships)
}

fn vpc_step(id: u8) -> Result<SeedStep, SeedError> {
    Ok(SeedStep::new(
        StepId::new(id),
        String::from("ec2"),
        String::from("CreateVpc"),
        String::from("AWS::EC2::VPC"),
        FieldPath::new(&["Vpc", "VpcId"])?,
        vec![SeedInput::new(
            FieldPath::new(&["CidrBlock"])?,
            SeedInputValue::Text(String::from("10.88.0.0/16")),
        )],
    )
    .with_response_checks(vec![SeedOutputCheck::new(
        FieldPath::new(&["Vpc", "CidrBlock"])?,
        FieldPath::new(&["CidrBlock"])?,
    )]))
}

fn subnet_step(id: u8, vpc: u8, cidr: &str) -> Result<SeedStep, SeedError> {
    let inputs = vec![
        SeedInput::new(
            FieldPath::new(&["VpcId"])?,
            SeedInputValue::PreviousOutput {
                step: StepId::new(vpc),
                path: FieldPath::new(&["Vpc", "VpcId"])?,
            },
        ),
        SeedInput::new(
            FieldPath::new(&["CidrBlock"])?,
            SeedInputValue::Text(cidr.to_owned()),
        ),
    ];
    let checks = ["VpcId", "CidrBlock"]
        .into_iter()
        .map(|field| {
            Ok(SeedOutputCheck::new(
                FieldPath::new(&["Subnet", field])?,
                FieldPath::new(&[field])?,
            ))
        })
        .collect::<Result<Vec<_>, SeedError>>()?;
    Ok(SeedStep::new(
        StepId::new(id),
        String::from("ec2"),
        String::from("CreateSubnet"),
        String::from("AWS::EC2::Subnet"),
        FieldPath::new(&["Subnet", "SubnetId"])?,
        inputs,
    )
    .with_response_checks(checks))
}
