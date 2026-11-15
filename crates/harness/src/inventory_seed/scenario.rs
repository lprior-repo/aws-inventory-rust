#[path = "scenario/validation.rs"]
mod validation;
use super::config::SeedError;
use validation::*;

const MAX_STEPS: usize = 12;
const MAX_INPUTS_PER_STEP: usize = 8;
const MAX_TOTAL_INPUTS: usize = 24;
const MAX_BINDING_DEPTH: usize = 8;
const MAX_FIELD_NAME: usize = 128;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub struct StepId(pub u8);

impl StepId {
    pub fn new(value: u8) -> Self {
        Self(value)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldPath(Vec<String>);

impl FieldPath {
    pub fn new(parts: &[&str]) -> Result<Self, SeedError> {
        if parts.is_empty() || parts.len() > MAX_BINDING_DEPTH {
            return Err(SeedError::Invalid(
                "seed field path exceeds its depth limit",
            ));
        }
        let fields = parts
            .iter()
            .map(validate_field)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self(fields))
    }

    pub fn parts(&self) -> &[String] {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum SeedInputValue {
    Text(String),
    Boolean(bool),
    Integer(i64),
    PreviousOutput { step: StepId, path: FieldPath },
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct SeedInput {
    pub field: FieldPath,
    pub value: SeedInputValue,
}

impl SeedInput {
    pub fn new(field: FieldPath, value: SeedInputValue) -> Self {
        Self { field, value }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct SeedOutputCheck {
    pub path: FieldPath,
    pub input_field: FieldPath,
}

impl SeedOutputCheck {
    pub fn new(path: FieldPath, input_field: FieldPath) -> Self {
        Self { path, input_field }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct SeedStep {
    pub id: StepId,
    pub service: String,
    pub operation: String,
    pub kind: String,
    pub identity: FieldPath,
    pub inputs: Vec<SeedInput>,
    pub response_checks: Vec<SeedOutputCheck>,
}

impl SeedStep {
    pub fn new(
        id: StepId,
        service: String,
        operation: String,
        kind: String,
        identity: FieldPath,
        inputs: Vec<SeedInput>,
    ) -> Self {
        Self {
            id,
            service,
            operation,
            kind,
            identity,
            inputs,
            response_checks: Vec::new(),
        }
    }

    pub fn with_response_checks(mut self, checks: Vec<SeedOutputCheck>) -> Self {
        self.response_checks = checks;
        self
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct SeedRelationship {
    pub source: StepId,
    pub target: StepId,
    pub relation: String,
    pub observed_path: FieldPath,
}

impl SeedRelationship {
    pub fn new(source: StepId, target: StepId, relation: String, observed_path: FieldPath) -> Self {
        Self {
            source,
            target,
            relation,
            observed_path,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SeedScenario {
    steps: Vec<SeedStep>,
    relationships: Vec<SeedRelationship>,
}

impl SeedScenario {
    pub fn new(steps: Vec<SeedStep>) -> Result<Self, SeedError> {
        Self::with_relationships(steps, Vec::new())
    }

    pub fn with_relationships(
        steps: Vec<SeedStep>,
        relationships: Vec<SeedRelationship>,
    ) -> Result<Self, SeedError> {
        validate_limits(&steps, &relationships)?;
        steps.iter().enumerate().try_for_each(validate_step)?;
        validate_bindings(&steps)?;
        validate_relationships(&steps, &relationships)?;
        Ok(Self {
            steps,
            relationships,
        })
    }

    pub fn steps(&self) -> &[SeedStep] {
        &self.steps
    }

    pub fn relationships(&self) -> &[SeedRelationship] {
        &self.relationships
    }
}

pub fn default_scenario() -> SeedScenario {
    let pairs = [
        (0, 1, "10.88.1.0/24"),
        (2, 3, "10.88.2.0/24"),
        (4, 5, "10.88.3.0/24"),
        (6, 7, "10.88.4.0/24"),
        (8, 9, "10.88.5.0/24"),
        (10, 11, "10.88.6.0/24"),
    ];
    let steps = pairs.into_iter().flat_map(default_pair_steps).collect();
    let relationships = pairs.into_iter().map(default_relationship).collect();
    SeedScenario {
        steps,
        relationships,
    }
}

fn default_pair_steps((vpc, subnet, cidr): (u8, u8, &'static str)) -> [SeedStep; 2] {
    [
        default_vpc_step(vpc),
        default_subnet_step(vpc, subnet, cidr),
    ]
}

fn default_vpc_step(vpc: u8) -> SeedStep {
    let cidr = SeedInputValue::Text(String::from("10.88.0.0/16"));
    SeedStep::new(
        StepId(vpc),
        String::from("ec2"),
        String::from("CreateVpc"),
        String::from("AWS::EC2::VPC"),
        path(&["Vpc", "VpcId"]),
        vec![input(&["CidrBlock"], cidr)],
    )
    .with_response_checks(vec![SeedOutputCheck::new(
        path(&["Vpc", "CidrBlock"]),
        path(&["CidrBlock"]),
    )])
}

fn default_subnet_step(vpc: u8, subnet: u8, cidr: &'static str) -> SeedStep {
    let vpc_id = SeedInputValue::PreviousOutput {
        step: StepId(vpc),
        path: path(&["Vpc", "VpcId"]),
    };
    SeedStep::new(
        StepId(subnet),
        String::from("ec2"),
        String::from("CreateSubnet"),
        String::from("AWS::EC2::Subnet"),
        path(&["Subnet", "SubnetId"]),
        vec![
            input(&["VpcId"], vpc_id),
            input(&["CidrBlock"], SeedInputValue::Text(String::from(cidr))),
        ],
    )
    .with_response_checks(vec![
        SeedOutputCheck::new(path(&["Subnet", "VpcId"]), path(&["VpcId"])),
        SeedOutputCheck::new(path(&["Subnet", "CidrBlock"]), path(&["CidrBlock"])),
    ])
}

fn default_relationship((vpc, subnet, _): (u8, u8, &'static str)) -> SeedRelationship {
    SeedRelationship::new(
        StepId(subnet),
        StepId(vpc),
        String::from("IN_VPC"),
        path(&["Subnet", "VpcId"]),
    )
}

fn path(parts: &[&str]) -> FieldPath {
    FieldPath(parts.iter().map(|part| (*part).to_owned()).collect())
}

fn input(field: &[&str], value: SeedInputValue) -> SeedInput {
    SeedInput::new(path(field), value)
}

fn validate_limits(
    steps: &[SeedStep],
    relationships: &[SeedRelationship],
) -> Result<(), SeedError> {
    if !(2..=MAX_STEPS).contains(&steps.len()) || relationships.len() > 6 {
        return Err(SeedError::Invalid(
            "seed scenario exceeds its step or relationship limit",
        ));
    }
    let input_count = steps.iter().try_fold(0_usize, |count, step| {
        count
            .checked_add(step.inputs.len())
            .ok_or(SeedError::Invalid("seed scenario input count is invalid"))
    })?;
    if input_count > MAX_TOTAL_INPUTS {
        return Err(SeedError::Invalid(
            "seed scenario exceeds its input binding limit",
        ));
    }
    Ok(())
}
