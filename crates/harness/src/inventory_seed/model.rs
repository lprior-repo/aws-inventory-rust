#[path = "model/members.rs"]
mod members;
use aws_wire::request::Protocol;
use inventory_core::{
    catalog::runtime::RuntimeCatalog,
    catalog::{Catalog, Member, Operation, Service, Target},
    inventory::{Kind, Relation},
    record::FieldId,
};
use members::*;

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
    pub output_path: Vec<FieldId>,
}

#[derive(Clone)]
pub(super) enum PreparedInputValue {
    Text(String),
    Boolean(bool),
    Integer(i64),
    PreviousOutput { step: StepId, path: Vec<FieldId> },
}
pub(super) struct PreparedStep<'a> {
    pub id: StepId,
    pub kind: Kind,
    pub operation: SeedOperation<'a>,
    pub identity: Vec<FieldId>,
    pub inputs: Vec<PreparedInput>,
    pub checks: Vec<PreparedCheck>,
}

pub(super) struct PreparedRelationship {
    pub source: StepId,
    pub target: StepId,
    pub relation: Relation,
    pub observed_path: Vec<FieldId>,
}

pub(super) struct SeedModels<'a> {
    pub caller: SeedOperation<'a>,
    pub caller_account: Vec<FieldId>,
    pub steps: Vec<PreparedStep<'a>>,
    pub relationships: Vec<PreparedRelationship>,
}

impl<'a> SeedModels<'a> {
    pub fn load(runtime: &RuntimeCatalog<'a>, scenario: &SeedScenario) -> Result<Self, SeedError> {
        let catalog = runtime.catalog();
        let caller = locate(catalog, "sts", "GetCallerIdentity")?;
        let caller_account = field_path(caller.service, caller.operation.output()?, &["Account"])?;
        let steps = scenario
            .steps()
            .iter()
            .map(|step| prepare_step(catalog, scenario, step))
            .collect::<Result<Vec<_>, _>>()?;
        let relationships = scenario
            .relationships()
            .iter()
            .map(|spec| prepare_relationship(runtime, scenario, &steps, spec))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            caller,
            caller_account,
            steps,
            relationships,
        })
    }
}

fn prepare_step<'a>(
    catalog: Catalog<'a>,
    scenario: &SeedScenario,
    step: &SeedStep,
) -> Result<PreparedStep<'a>, SeedError> {
    let operation = locate(catalog, &step.service, &step.operation)?;
    let output = operation.operation.output()?;
    let identity = modeled_path(operation.service, output, &step.identity)?;
    let (inputs, checks) = step_parts(catalog, scenario, operation, step)?;
    Ok(PreparedStep {
        id: step.id,
        kind: Kind::parse(&step.kind)?,
        operation,
        identity,
        inputs,
        checks,
    })
}

type StepParts = (Vec<PreparedInput>, Vec<PreparedCheck>);

fn step_parts<'a>(
    catalog: Catalog<'a>,
    scenario: &SeedScenario,
    operation: SeedOperation<'a>,
    step: &SeedStep,
) -> Result<StepParts, SeedError> {
    let inputs = step
        .inputs
        .iter()
        .map(|input| prepare_input(catalog, scenario, operation, &input.field, &input.value))
        .collect::<Result<Vec<_>, _>>()?;
    let checks = step
        .response_checks
        .iter()
        .map(|check| prepare_check(operation, &check.path, &check.input_field))
        .collect::<Result<Vec<_>, _>>()?;
    Ok((inputs, checks))
}

fn prepare_input<'a>(
    catalog: Catalog<'a>,
    scenario: &SeedScenario,
    operation: SeedOperation<'a>,
    field: &FieldPath,
    value: &SeedInputValue,
) -> Result<PreparedInput, SeedError> {
    let path = modeled_path(operation.service, operation.operation.input()?, field)?;
    let value = match value {
        SeedInputValue::Text(text) => PreparedInputValue::Text(text.clone()),
        SeedInputValue::Boolean(value) => PreparedInputValue::Boolean(*value),
        SeedInputValue::Integer(value) => PreparedInputValue::Integer(*value),
        SeedInputValue::PreviousOutput { step, path } => {
            previous_output(catalog, scenario, *step, path)?
        }
    };
    Ok(PreparedInput { path, value })
}

fn previous_output(
    catalog: Catalog<'_>,
    scenario: &SeedScenario,
    step: StepId,
    path: &FieldPath,
) -> Result<PreparedInputValue, SeedError> {
    let source = scenario
        .steps()
        .get(usize::from(step.0))
        .ok_or(SeedError::Invalid("seed binding source is missing"))?;
    let operation = locate(catalog, &source.service, &source.operation)?;
    let path = modeled_path(operation.service, operation.operation.output()?, path)?;
    Ok(PreparedInputValue::PreviousOutput { step, path })
}

fn prepare_check(
    operation: SeedOperation<'_>,
    output: &FieldPath,
    input: &FieldPath,
) -> Result<PreparedCheck, SeedError> {
    let input_path = modeled_path(operation.service, operation.operation.input()?, input)?;
    let output_path = modeled_path(operation.service, operation.operation.output()?, output)?;
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
    Ok(PreparedRelationship {
        source: spec.source,
        target: spec.target,
        relation: read_relation(runtime, &source.kind, &target.kind, &spec.relation)?,
        observed_path: observed_path(runtime, scenario, spec)?,
    })
}

fn observed_path(
    runtime: &RuntimeCatalog<'_>,
    scenario: &SeedScenario,
    spec: &SeedRelationship,
) -> Result<Vec<FieldId>, SeedError> {
    let missing = SeedError::Invalid("seed relationship source descriptor is missing");
    let source = scenario
        .steps()
        .get(usize::from(spec.source.0))
        .ok_or(missing)?;
    let operation = locate(runtime.catalog(), &source.service, &source.operation)?;
    let output = operation.operation.output()?;
    modeled_path(operation.service, output, &spec.observed_path)
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

pub(super) fn locate<'a>(
    catalog: Catalog<'a>,
    service_name: &str,
    operation_name: &str,
) -> Result<SeedOperation<'a>, SeedError> {
    let service = catalog.service(service_name)?.ok_or(SeedError::Invalid(
        "modeled service is missing from catalog",
    ))?;
    let operation = find_operation(service, operation_name)?;
    let protocol = protocol(service)?;
    Ok(SeedOperation {
        service,
        operation,
        protocol,
    })
}

fn find_operation<'a>(service: Service<'a>, name: &str) -> Result<Operation<'a>, SeedError> {
    service
        .operations()?
        .try_fold(None, |found, candidate| {
            let candidate = candidate?;
            if candidate.name()? == name {
                Ok::<Option<Operation<'a>>, inventory_core::catalog::CatalogError>(Some(candidate))
            } else {
                Ok(found)
            }
        })?
        .ok_or(SeedError::Invalid(
            "modeled operation is missing from catalog",
        ))
}

fn protocol(service: Service<'_>) -> Result<Protocol, SeedError> {
    service
        .protocols()?
        .try_fold(None, |found, candidate| {
            let protocol = match candidate? {
                "aws.protocols#awsQuery" => Some(Protocol::AwsQuery),
                "aws.protocols#ec2Query" => Some(Protocol::Ec2Query),
                _ => None,
            };
            Ok::<_, SeedError>(found.or(protocol))
        })?
        .ok_or(SeedError::Invalid(
            "seed operation service has no supported query protocol",
        ))
}
