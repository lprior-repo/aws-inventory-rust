#[path = "model/members.rs"]
pub(super) mod members;
mod operation;
pub(super) use operation::locate;
mod identity;
mod output;
mod template;
mod values;
use values::prepare as prepare_value;
#[cfg(test)]
mod tests;
use aws_wire::request::Protocol;
pub(super) use identity::{PreparedIdentity, PreparedResource};
use inventory_core::{
    catalog::runtime::RuntimeCatalog,
    catalog::{Catalog, Member, Operation, Service, Target},
    inventory::{Kind, Relation},
    record::FieldId,
};
use members::*;
pub(super) use output::{Selector, output_path};
#[cfg(test)]
pub(super) use template::PreparedNestedBinding;
pub(super) use template::PreparedTemplate;

use super::{
    config::SeedError,
    scenario::{FieldPath, SeedInputValue, SeedRelationship, SeedScenario, SeedStep, StepId},
};

#[derive(Clone, Copy)]
pub(super) struct SeedOperation<'a> {
    pub service: Service<'a>,
    pub operation: Operation<'a>,
    pub protocol: Protocol,
}

#[derive(Clone)]
pub(super) struct PreparedInput {
    pub path: Vec<FieldId>,
    pub value: PreparedInputValue,
}
#[derive(Clone)]
pub(super) struct PreparedCheck {
    pub input_path: Vec<FieldId>,
    pub output_path: Vec<Selector>,
}

#[derive(Clone)]
pub(super) enum PreparedInputValue {
    Text(String),
    Boolean(bool),
    Integer(i64),
    PreviousOutput { step: StepId, path: Vec<Selector> },
    Literal(inventory_core::record::Value),
    Template(PreparedTemplate),
    List(Vec<PreparedInputValue>),
    Structure(Vec<(inventory_core::record::Key, PreparedInputValue)>),
}
pub(super) struct PreparedStep<'a> {
    pub id: StepId,
    pub resource: Option<PreparedResource>,
    pub operation: SeedOperation<'a>,
    pub inputs: Vec<PreparedInput>,
    pub checks: Vec<PreparedCheck>,
}

pub(super) struct PreparedRelationship {
    pub source: StepId,
    pub target: StepId,
    pub relation: Relation,
    pub observed_path: Vec<Selector>,
}

pub(super) struct SeedModels<'a> {
    pub caller: SeedOperation<'a>,
    pub caller_account: Vec<Selector>,
    pub caller_arn: Vec<Selector>,
    pub steps: Vec<PreparedStep<'a>>,
    pub relationships: Vec<PreparedRelationship>,
}

impl<'a> SeedModels<'a> {
    pub fn load(runtime: &RuntimeCatalog<'a>, scenario: &SeedScenario) -> Result<Self, SeedError> {
        let catalog = runtime.catalog();
        let caller = locate(catalog, "sts", "GetCallerIdentity")?;
        let caller_account = caller_path(caller, "Account")?;
        let caller_arn = caller_path(caller, "Arn")?;
        let operations = operation::prepare(catalog, scenario)?;
        let steps = scenario
            .steps()
            .iter()
            .map(|step| prepare_step(&operations, step))
            .collect::<Result<Vec<_>, _>>()?;
        let relationships = scenario
            .relationships()
            .iter()
            .map(|spec| prepare_relationship(runtime, scenario, &steps, spec))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            caller,
            caller_account,
            caller_arn,
            steps,
            relationships,
        })
    }
}

fn caller_path(caller: SeedOperation<'_>, field: &str) -> Result<Vec<Selector>, SeedError> {
    output_path(
        caller.service,
        caller.operation.output()?,
        &FieldPath::new(&[field])?,
    )
}

fn prepare_step<'a>(
    operations: &[SeedOperation<'a>],
    step: &SeedStep,
) -> Result<PreparedStep<'a>, SeedError> {
    let operation = *operations
        .get(usize::from(step.id.0))
        .ok_or(SeedError::Invalid("seed prepared operation is missing"))?;
    let resource = identity::resource(operation, step)?;
    let (inputs, checks) = step_parts(operations, operation, step)?;
    Ok(PreparedStep {
        id: step.id,
        operation,
        resource,
        inputs,
        checks,
    })
}

type StepParts = (Vec<PreparedInput>, Vec<PreparedCheck>);

fn step_parts<'a>(
    operations: &[SeedOperation<'a>],
    operation: SeedOperation<'a>,
    step: &SeedStep,
) -> Result<StepParts, SeedError> {
    let inputs = step
        .inputs
        .iter()
        .map(|input| prepare_input(operations, operation, &input.field, &input.value))
        .collect::<Result<Vec<_>, _>>()?;
    let checks = step
        .response_checks
        .iter()
        .map(|check| prepare_check(operation, &check.path, &check.input_field))
        .collect::<Result<Vec<_>, _>>()?;
    Ok((inputs, checks))
}

fn prepare_input<'a>(
    operations: &[SeedOperation<'a>],
    operation: SeedOperation<'a>,
    field: &FieldPath,
    value: &SeedInputValue,
) -> Result<PreparedInput, SeedError> {
    let path = modeled_path(operation.service, operation.operation.input()?, field)?;
    let value = prepare_value(operations, operation, field, value)?;
    Ok(PreparedInput { path, value })
}

fn previous_output(
    operations: &[SeedOperation<'_>],
    step: StepId,
    path: &FieldPath,
) -> Result<PreparedInputValue, SeedError> {
    let operation = operations
        .get(usize::from(step.0))
        .ok_or(SeedError::Invalid("seed binding source is missing"))?;
    let path = output_path(operation.service, operation.operation.output()?, path)?;
    Ok(PreparedInputValue::PreviousOutput { step, path })
}

fn prepare_check(
    operation: SeedOperation<'_>,
    output: &FieldPath,
    input: &FieldPath,
) -> Result<PreparedCheck, SeedError> {
    let input_path = modeled_path(operation.service, operation.operation.input()?, input)?;
    let output_path = output_path(operation.service, operation.operation.output()?, output)?;
    Ok(PreparedCheck {
        input_path,
        output_path,
    })
}

fn prepare_relationship(
    runtime: &RuntimeCatalog<'_>,
    scenario: &SeedScenario,
    steps: &[PreparedStep<'_>],
    spec: &SeedRelationship,
) -> Result<PreparedRelationship, SeedError> {
    let source = prepared(steps, spec.source, "seed relationship source is missing")?;
    let target = prepared(steps, spec.target, "seed relationship target is missing")?;
    let source_kind = &source
        .resource
        .as_ref()
        .ok_or(SeedError::Invalid("relationship source must be a resource"))?
        .kind;
    let target_kind = &target
        .resource
        .as_ref()
        .ok_or(SeedError::Invalid("relationship target must be a resource"))?
        .kind;
    Ok(PreparedRelationship {
        source: spec.source,
        target: spec.target,
        relation: read_relation(runtime, source_kind, target_kind, &spec.relation)?,
        observed_path: observed_path(runtime, scenario, spec)?,
    })
}

fn observed_path(
    runtime: &RuntimeCatalog<'_>,
    scenario: &SeedScenario,
    spec: &SeedRelationship,
) -> Result<Vec<Selector>, SeedError> {
    let missing = SeedError::Invalid("seed relationship source descriptor is missing");
    let source = scenario
        .steps()
        .get(usize::from(spec.source.0))
        .ok_or(missing)?;
    let operation = locate(runtime.catalog(), &source.service, &source.operation)?;
    let output = operation.operation.output()?;
    output_path(operation.service, output, &spec.observed_path)
}

fn prepared<'s, 'a>(
    steps: &'s [PreparedStep<'a>],
    step: StepId,
    missing: &'static str,
) -> Result<&'s PreparedStep<'a>, SeedError> {
    steps
        .get(usize::from(step.0))
        .ok_or(SeedError::Invalid(missing))
}

fn modeled_path(
    service: Service<'_>,
    target: Target,
    path: &FieldPath,
) -> Result<Vec<FieldId>, SeedError> {
    let parts = path.parts().iter().map(String::as_str).collect::<Vec<_>>();
    field_path(service, target, &parts)
}

fn read_relation(
    runtime: &RuntimeCatalog<'_>,
    source: &Kind,
    target: &Kind,
    name: &str,
) -> Result<Relation, SeedError> {
    runtime
        .relationships()
        .iter()
        .find(|plan| {
            &plan.source_kind == source
                && &plan.target_kind == target
                && plan.relation.as_str() == name
        })
        .map(|plan| plan.relation.clone())
        .ok_or(SeedError::Invalid("modeled seed relationship is missing"))
}
