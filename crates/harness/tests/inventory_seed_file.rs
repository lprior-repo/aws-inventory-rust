use inventory_harness::inventory_seed::{SeedInputValue, StepId, load};

type TestResult = Result<(), Box<dyn std::error::Error>>;
const HEADER: &str = "format = 1\nservice = 'ec2'\n";
const STEP: &str = "[[step]]\nid = 'vpc'\noperation = 'CreateVpc'\nkind = 'AWS::EC2::VPC'\nidentity = 'Vpc.VpcId'\n";

fn document(extra: &str) -> String {
    format!("{HEADER}{STEP}{extra}")
}

#[test]
fn inventory_seed_rejects_unknown_schema_keys() {
    [
        format!("{HEADER}unknown = 1\n{STEP}"),
        document("unknown = 1"),
        document("[step.input]\nVpcId = { from = 'vpc', path = 'Vpc.VpcId', unknown = 1 }"),
        document("[[relationship]]\nsource = 'vpc'\ntarget = 'vpc'\nrelation = 'IN_VPC'\nobserved = 'VpcId'\nunknown = 1"),
    ].iter().for_each(|content| assert!(load(content).is_err()));
}

#[test]
fn inventory_seed_rejects_invalid_field_types() {
    [
        "kind = 1",
        "identity = 1",
        "scope = 1",
        "repeat = '2'",
        "input = []",
        "check = []",
    ]
    .iter()
    .for_each(|field| {
        let content = match *field {
            "kind = 1" => document("").replace("kind = 'AWS::EC2::VPC'", field),
            "identity = 1" => document("").replace("identity = 'Vpc.VpcId'", field),
            _ => document(field),
        };
        assert!(load(&content).is_err(), "{field}");
    });
    assert!(load(&format!("{HEADER}relationship = 1\n{STEP}")).is_err());
}

#[test]
fn inventory_seed_rejects_forward_missing_and_self_references() {
    ["later", "missing", "vpc"].iter().for_each(|source| {
        let content = document(&format!("[step.input]\nValue = {{ from = '{source}', path = 'Vpc.VpcId' }}\n[[step]]\nid = 'later'\noperation = 'CreateVpc'\nkind = ''"));
        assert!(load(&content).is_err());
    });
}

#[test]
fn inventory_seed_expands_indices_and_binds_same_index() -> TestResult {
    let content = document(
        "repeat = 3\n[step.input]\nCidrBlock = '10.{n}.0.0/16'\n[[step]]\nid = 'child'\noperation = 'CreateSubnet'\nkind = ''\nrepeat = 3\n[step.input]\nVpcId = { from = 'vpc', path = 'Vpc.VpcId' }",
    );
    let scenario = load(&content)?;
    assert_eq!(scenario.steps().len(), 6);
    scenario.steps().as_chunks::<2>().0.iter().enumerate().try_for_each(|(index, pair)| {
        let parent = pair.first().ok_or("parent missing")?;
        let child = pair.get(1).ok_or("child missing")?;
        let n = index.checked_add(1).ok_or("overflow")?;
        assert_eq!(parent.inputs.first().map(|input| &input.value), Some(&SeedInputValue::Text(format!("10.{n}.0.0/16"))));
        assert!(matches!(child.inputs.first().map(|input| &input.value), Some(SeedInputValue::PreviousOutput { step, .. }) if *step == parent.id));
        Ok::<_, Box<dyn std::error::Error>>(())
    })
}

#[test]
fn inventory_seed_broadcasts_single_parent_without_aliasing_dash_ids() -> TestResult {
    let content = document(
        "[[step]]\nid = 'vpc-0'\noperation = 'CreateSubnet'\nkind = ''\nrepeat = 2\n[step.input]\nVpcId = { from = 'vpc', path = 'Vpc.VpcId' }",
    );
    let scenario = load(&content)?;
    assert_eq!(scenario.steps().len(), 3);
    scenario.steps().iter().skip(1).for_each(|step| {
        assert!(matches!(step.inputs.first().map(|input| &input.value), Some(SeedInputValue::PreviousOutput { step, .. }) if *step == StepId::new(0)));
    });
    Ok(())
}

#[test]
fn inventory_seed_accepts_auxiliary_steps_without_identity() -> TestResult {
    let scenario = load(&format!(
        "{HEADER}[[step]]\nid = 'attach'\noperation = 'AttachPolicy'\nkind = ''"
    ))?;
    assert!(
        scenario
            .steps()
            .first()
            .ok_or("missing step")?
            .identity
            .is_none()
    );
    Ok(())
}

#[test]
fn inventory_seed_enforces_repeat_boundaries_before_expansion() -> TestResult {
    [0, -1, 4097].iter().for_each(|repeat| {
        assert!(load(&document(&format!("repeat = {repeat}"))).is_err());
    });
    assert_eq!(load(&document("repeat = 4096"))?.steps().len(), 4096);
    let content =
        document("repeat = 4096\n[[step]]\nid = 'extra'\noperation = 'CreateVpc'\nkind = ''");
    assert!(load(&content).is_err());
    Ok(())
}

#[test]
fn inventory_seed_enforces_file_depth_text_and_node_limits() {
    assert!(load(&" ".repeat(1_048_577)).is_err());
    let at_depth = format!("{}'value'{}", "[".repeat(8), "]".repeat(8));
    assert!(load(&document(&format!("[step.input]\nValue = {at_depth}"))).is_ok());
    let beyond_depth = format!("[{at_depth}]");
    assert!(load(&document(&format!("[step.input]\nValue = {beyond_depth}"))).is_err());
    assert!(
        load(&document(&format!(
            "[step.input]\nValue = '{}'",
            "x".repeat(1_000_000)
        )))
        .is_ok()
    );
    let too_many = std::iter::repeat_n("1", 257).collect::<Vec<_>>().join(",");
    assert!(load(&document(&format!("[step.input]\nValue = [{too_many}]"))).is_err());
}

#[test]
fn inventory_seed_preserves_nested_binding_and_template_values() -> TestResult {
    let content = document(
        "[[step]]\nid = 'child'\noperation = 'CreateSubnet'\nkind = ''\nrepeat = 2\n[step.input]\nTags = [{ Key = 'name', Value = 'seed-{n}' }, { Key = 'parent', Value = { from = 'vpc', path = 'Vpc.VpcId' } }]",
    );
    let scenario = load(&content)?;
    let child = scenario.steps().get(2).ok_or("missing child")?;
    let Some(SeedInputValue::List(tags)) = child.inputs.first().map(|input| &input.value) else {
        return Err("missing list".into());
    };
    let Some(SeedInputValue::Structure(tag)) = tags.first() else {
        return Err("missing structure".into());
    };
    assert!(
        tag.iter()
            .any(|(_, value)| *value == SeedInputValue::Text("seed-2".into()))
    );
    Ok(())
}
