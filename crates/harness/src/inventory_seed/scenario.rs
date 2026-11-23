mod defaults;
mod identity;
mod template;
#[path = "scenario/validation.rs"]
mod validation;
use super::config::SeedError;
pub use identity::{SeedIdentity, SeedIdentityEvidence, SeedScope};
pub use template::{SeedNestedBinding, SeedTemplate, SeedValuePart};
use validation::*;

const MAX_STEPS: usize = 12;
const MAX_INPUTS_PER_STEP: usize = 64;
const MAX_TOTAL_INPUTS: usize = 65536;
const MAX_BINDING_DEPTH: usize = 8;
const MAX_FIELD_NAME: usize = 128;
const MAX_STEP_ID: u16 = 4095;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub struct StepId(pub u16);

impl StepId {
    pub fn new(value: u16) -> Self {
        Self(value)
    }
}

impl TryFrom<usize> for StepId {
    type Error = SeedError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        let id = u16::try_from(value).map_err(|_| {
            SeedError::Invalid("seed step id value exceeds its limit")
        })?;
        Ok(Self::new(id))
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
    Literal(inventory_core::record::Value),
    Template(SeedTemplate),
    List(Vec<SeedInputValue>),
    Structure(Vec<(String, SeedInputValue)>),
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
    pub identity: Option<SeedIdentity>,
    pub inputs: Vec<SeedInput>,
    pub response_checks: Vec<SeedOutputCheck>,
    pub expected_identity: Option<String>,
    pub scope: SeedScope,
    pub repeat: u32,
}

impl SeedStep {
    pub fn new(
        id: StepId,
        service: String,
        operation: String,
        kind: String,
        identity: impl Into<SeedIdentity>,
        inputs: Vec<SeedInput>,
    ) -> Self {
        Self {
            id,
            service,
            operation,
            kind,
            identity: Some(identity.into()),
            inputs,
            response_checks: Vec::new(),
            expected_identity: None,
            scope: SeedScope::Regional,
            repeat: 1,
        }
    }

    pub fn with_response_checks(mut self, checks: Vec<SeedOutputCheck>) -> Self {
        self.response_checks = checks;
        self
    }

    pub fn with_expected_identity(mut self, identity: String) -> Self {
        self.expected_identity = Some(identity);
        self
    }

    pub fn with_scope(mut self, scope: SeedScope) -> Self {
        self.scope = scope;
        self
    }

    pub fn with_repeat(mut self, repeat: u32) -> Self {
        self.repeat = repeat;
        self
    }

    pub fn auxiliary(
        id: StepId,
        service: String,
        operation: String,
        inputs: Vec<SeedInput>,
    ) -> Self {
        Self {
            id,
            service,
            operation,
            inputs,
            kind: String::new(),
            identity: None,
            response_checks: Vec::new(),
            expected_identity: None,
            scope: SeedScope::Regional,
            repeat: 1,
        }
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

pub fn default_scenario() -> Result<SeedScenario, SeedError> {
    defaults::scenario()
}

fn validate_limits(
    steps: &[SeedStep],
    relationships: &[SeedRelationship],
) -> Result<(), SeedError> {
    if steps.is_empty() || steps.len() > MAX_STEPS {
        return Err(SeedError::Invalid(
            "seed scenario exceeds its step limit",
        ));
    }
    let expanded = steps
        .iter()
        .try_fold(0_usize, |count, step| {
            let step_count = usize::try_from(step.repeat).map_err(|_| {
                SeedError::Invalid("seed step repeat count is invalid")
            })?;
            count
                .checked_add(step_count)
                .ok_or(SeedError::Invalid(
                    "seed scenario expanded step count overflow",
                ))
        })?;
    if expanded > MAX_STEPS {
        return Err(SeedError::Invalid(
            "seed scenario exceeds its expanded step limit",
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
    if relationships.len() > MAX_STEPS {
        return Err(SeedError::Invalid(
            "seed scenario exceeds its relationship limit",
        ));
    }
    Ok(())
}
