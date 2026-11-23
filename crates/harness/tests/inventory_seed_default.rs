use inventory_harness::inventory_seed::{
    FieldPath, SeedIdentity, SeedInputValue, SeedRelationship, SeedScope, SeedStep, StepId,
    default_scenario,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn inventory_seed_embedded_default_matches_every_legacy_descriptor() -> TestResult {
    let scenario = default_scenario()?;
    assert_eq!(scenario.steps().len(), 12);
    assert_eq!(scenario.relationships().len(), 6);
    scenario
        .steps()
        .as_chunks::<2>()
        .0
        .iter()
        .enumerate()
        .zip(scenario.relationships())
        .try_for_each(|((index, steps), edge)| pair(index, steps, edge))
}

fn common(step: &SeedStep, id: u16, operation: &str, kind: &str, identity: &[&str]) -> TestResult {
    assert_eq!(step.id, StepId::new(id));
    assert_eq!(step.service, "ec2");
    assert_eq!(step.operation, operation);
    assert_eq!(step.kind, kind);
    assert_eq!(
        step.identity,
        Some(SeedIdentity::Response(FieldPath::new(identity)?))
    );
    assert_eq!(step.scope, SeedScope::Regional);
    assert_eq!(step.expected_identity, None);
    Ok(())
}

fn vpc_descriptor(step: &SeedStep, id: u16) -> TestResult {
    common(step, id, "CreateVpc", "AWS::EC2::VPC", &["Vpc", "VpcId"])?;
    assert_eq!(step.inputs.len(), 1);
    let input = step.inputs.first().ok_or("missing cidr")?;
    assert_eq!(input.field, FieldPath::new(&["CidrBlock"])?);
    assert_eq!(input.value, SeedInputValue::Text("10.88.0.0/16".into()));
    checks(step, "Vpc", &["CidrBlock"])
}

fn subnet_descriptor(step: &SeedStep, id: u16, parent: u16, n: u16) -> TestResult {
    let identity = ["Subnet", "SubnetId"];
    common(step, id, "CreateSubnet", "AWS::EC2::Subnet", &identity)?;
    assert_eq!(step.inputs.len(), 2);
    let parent_input = step.inputs.first().ok_or("missing parent input")?;
    assert_eq!(parent_input.field, FieldPath::new(&["VpcId"])?);
    assert_eq!(
        parent_input.value,
        SeedInputValue::PreviousOutput {
            step: StepId::new(parent),
            path: FieldPath::new(&["Vpc", "VpcId"])?,
        }
    );
    let cidr_input = step.inputs.get(1).ok_or("missing subnet cidr")?;
    assert_eq!(cidr_input.field, FieldPath::new(&["CidrBlock"])?);
    assert_eq!(
        cidr_input.value,
        SeedInputValue::Text(format!("10.88.{n}.0/24"))
    );
    checks(step, "Subnet", &["VpcId", "CidrBlock"])
}

fn checks(step: &SeedStep, resource: &str, fields: &[&str]) -> TestResult {
    assert_eq!(step.response_checks.len(), fields.len());
    step.response_checks
        .iter()
        .zip(fields)
        .try_for_each(|(check, name)| {
            assert_eq!(check.path, FieldPath::new(&[resource, name])?);
            assert_eq!(check.input_field, FieldPath::new(&[name])?);
            Ok(())
        })
}

fn relationship(edge: &SeedRelationship, source: u16, target: u16) -> TestResult {
    assert_eq!(edge.source, StepId::new(source));
    assert_eq!(edge.target, StepId::new(target));
    assert_eq!(edge.relation, "IN_VPC");
    assert_eq!(edge.observed_path, FieldPath::new(&["Subnet", "VpcId"])?);
    Ok(())
}

fn pair(index: usize, [vpc, subnet]: &[SeedStep; 2], edge: &SeedRelationship) -> TestResult {
    let ordinal = u16::try_from(index)?;
    let vpc_id = ordinal.checked_mul(2).ok_or("id overflow")?;
    let subnet_id = vpc_id.checked_add(1).ok_or("id overflow")?;
    let n = ordinal.checked_add(1).ok_or("index overflow")?;
    vpc_descriptor(vpc, vpc_id)?;
    subnet_descriptor(subnet, subnet_id, vpc_id, n)?;
    relationship(edge, subnet_id, vpc_id)
}
