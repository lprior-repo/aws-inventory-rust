use super::super::{SeedNestedBinding, SeedTemplate, SeedValuePart};
use super::*;
use inventory_core::record::Value;

#[derive(Clone)]
pub(in crate::inventory_seed) struct PreparedNestedBinding {
    pub destination: Vec<Selector>,
    pub step: StepId,
    pub path: Vec<Selector>,
}

#[derive(Clone)]
pub(in crate::inventory_seed) struct PreparedTemplate {
    pub value: Value,
    pub bindings: Vec<PreparedNestedBinding>,
}

pub(super) fn prepare(
    operations: &[SeedOperation<'_>],
    service: Service<'_>,
    target: Target,
    template: &SeedTemplate,
) -> Result<PreparedInputValue, SeedError> {
    let value = super::super::literal::resolve_template(service, target, &template.value)?;
    let bindings = template
        .bindings
        .iter()
        .map(|binding| binding_plan(operations, service, target, binding))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(PreparedInputValue::Template(PreparedTemplate {
        value,
        bindings,
    }))
}

fn binding_plan(
    operations: &[SeedOperation<'_>],
    service: Service<'_>,
    target: Target,
    binding: &SeedNestedBinding,
) -> Result<PreparedNestedBinding, SeedError> {
    let destination = destination(service, target, &binding.destination)?;
    let PreparedInputValue::PreviousOutput { step, path } =
        previous_output(operations, binding.step, &binding.path)?
    else {
        return Err(SeedError::Invalid(
            "nested seed binding is not an output reference",
        ));
    };
    Ok(PreparedNestedBinding {
        destination,
        step,
        path,
    })
}

fn destination(
    service: Service<'_>,
    mut target: Target,
    parts: &[SeedValuePart],
) -> Result<Vec<Selector>, SeedError> {
    parts
        .iter()
        .map(|part| {
            let (selector, next) = match part {
                SeedValuePart::Field(name) => output::select_target(service, target, name)?,
                SeedValuePart::Index(index) => list_target(service, target, *index)?,
            };
            target = next;
            Ok(selector)
        })
        .collect()
}

fn list_target(
    service: Service<'_>,
    target: Target,
    index: usize,
) -> Result<(Selector, Target), SeedError> {
    let Target::Shape(shape) = target else {
        return Err(SeedError::Invalid("nested seed list has no modeled shape"));
    };
    let target = service
        .shape(shape)?
        .ok_or(SeedError::Invalid("nested seed list shape is missing"))?
        .element()?
        .ok_or(SeedError::Invalid(
            "nested seed index requires a list shape",
        ))?;
    Ok((Selector::Index(index), target))
}
