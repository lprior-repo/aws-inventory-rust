use inventory_harness::inventory_seed::{SeedInputValue, StepId, load};

type TestResult = Result<(), Box<dyn std::error::Error>>;
const HEADER: &str = "format = 1\nservice = 'ec2'\n";
const STEP: &str = "[[step]]\nid = 'parent'\noperation = 'CreateVpc'\nkind = 'AWS::EC2::VPC'\nidentity = 'Vpc.VpcId'\n";

#[test]
fn inventory_seed_repeated_relationships_follow_same_index() -> TestResult {
    let content = format!(
        "{HEADER}{STEP}repeat = 50\n[[step]]\nid = 'child'\noperation = 'CreateSubnet'\nkind = 'AWS::EC2::Subnet'\nidentity = 'Subnet.SubnetId'\nrepeat = 50\n[step.input]\nVpcId = {{from = 'parent', path = 'Vpc.VpcId'}}\n[[relationship]]\nsource = 'child'\ntarget = 'parent'\nrelation = 'IN_VPC'\nobserved = 'Subnet.VpcId'"
    );
    let scenario = load(&content)?;
    assert_eq!(scenario.steps().len(), 100);
    assert_eq!(scenario.relationships().len(), 50);
    scenario
        .relationships()
        .iter()
        .enumerate()
        .try_for_each(|(index, edge)| {
            let parent = u16::try_from(index)?.checked_mul(2).ok_or("id overflow")?;
            assert_eq!(
                edge.source,
                StepId::new(parent.checked_add(1).ok_or("id overflow")?)
            );
            assert_eq!(edge.target, StepId::new(parent));
            Ok::<_, Box<dyn std::error::Error>>(())
        })
}

#[test]
fn inventory_seed_rejects_missing_repeat_instances_and_mismatched_relationships() {
    let content = format!(
        "{HEADER}{STEP}repeat = 2\n[[step]]\nid = 'child'\noperation = 'CreateSubnet'\nkind = 'AWS::EC2::Subnet'\nidentity = 'Subnet.SubnetId'\nrepeat = 3\n"
    );
    assert!(
        load(&format!(
            "{content}[step.input]\nVpcId = {{from = 'parent', path = 'Vpc.VpcId'}}"
        ))
        .is_err()
    );
    assert!(load(&format!("{content}[[relationship]]\nsource = 'child'\ntarget = 'parent'\nrelation = 'IN_VPC'\nobserved = 'Subnet.VpcId'")).is_err());
}

#[test]
fn inventory_seed_bounds_total_expanded_inputs() -> TestResult {
    let fields = (0..64)
        .map(|index| format!("Field{index} = true\n"))
        .collect::<String>();
    let content = format!("{HEADER}{STEP}repeat = 1024\n[step.input]\n{fields}");
    assert_eq!(load(&content)?.steps().len(), 1024);
    assert!(load(&content.replace("repeat = 1024", "repeat = 1025")).is_err());
    Ok(())
}

#[test]
fn inventory_seed_bounds_expanded_bytes_before_cloning_inputs() -> TestResult {
    let value = format!("'{}'", "x".repeat(1024));
    let values = std::iter::repeat_n(value, 31).collect::<Vec<_>>().join(",");
    let content = format!("{HEADER}{STEP}repeat = 2048\n[step.input]\nValue = [{values}]");
    let error = load(&content).err().ok_or("expected expanded byte limit")?;
    assert!(error.to_string().contains("byte limits"));
    Ok(())
}

#[test]
fn inventory_seed_rejects_duplicate_ids_and_malformed_nested_bindings() {
    assert!(load(&format!("{HEADER}{STEP}{STEP}")).is_err());
    [
        "{from = 'parent'}",
        "{path = 'Vpc.VpcId'}",
        "{from = 1, path = 'Vpc.VpcId'}",
        "nan",
        "inf",
        "1979-05-27",
    ]
    .iter()
    .for_each(|value| {
        assert!(load(&format!("{HEADER}{STEP}[step.input]\nValue = [{value}]")).is_err());
    });
}

#[test]
fn inventory_seed_accepts_node_boundary_and_unit_repeat() -> TestResult {
    let values = std::iter::repeat_n("true", 255)
        .collect::<Vec<_>>()
        .join(",");
    let content = format!("{HEADER}{STEP}repeat = 1\n[step.input]\nValue = [{values}]");
    let scenario = load(&content)?;
    assert_eq!(scenario.steps().len(), 1);
    let step = scenario.steps().first().ok_or("missing step")?;
    assert_eq!(step.id, StepId::new(0));
    assert!(
        matches!(step.inputs.first().map(|input| &input.value), Some(SeedInputValue::List(items)) if items.len() == 255)
    );
    Ok(())
}

#[test]
fn inventory_seed_bounds_repeated_relationship_and_service_bytes() -> TestResult {
    let metadata = "R".repeat(65_536);
    let relation = format!(
        "{HEADER}{STEP}repeat = 2048\n[[relationship]]\nsource = 'parent'\ntarget = 'parent'\nrelation = '{metadata}'\nobserved = 'Vpc.VpcId'"
    );
    let service = format!("format = 1\nservice = '{metadata}'\n{STEP}repeat = 2048");
    [relation, service].iter().try_for_each(|content| {
        let error = load(content).err().ok_or("expected expanded byte limit")?;
        assert!(error.to_string().contains("byte limits"));
        Ok(())
    })
}
