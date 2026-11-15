#[cfg(test)]
mod scalar_tests;

use inventory_core::{
    catalog::runtime::{
        BindingSource, BindingValue, CallablePlan, PlanPurpose, PlanScope, RuntimeCatalog,
    },
    inventory::{
        AccountId, Observation, OperationId, Region, RegionId, ResourceKey, ScopeKey, ServiceId,
    },
    record::{FieldId, Record, Value, ValueRef},
};
use inventory_engine::inventory::{SweepId, WorkSpec};

use crate::sweep_runtime::{SweepError, input, record};

const MAX_PAGE_CHILDREN: usize = 10_000;

pub(super) struct Context<'a, 'catalog> {
    catalog: &'a RuntimeCatalog<'catalog>,
    parent_plan: &'a CallablePlan<'catalog>,
    sweep: &'a SweepId,
    account: &'a AccountId,
    home: &'a RegionId,
}

impl<'a, 'catalog> Context<'a, 'catalog> {
    pub(super) fn new(
        catalog: &'a RuntimeCatalog<'catalog>,
        parent_plan: &'a CallablePlan<'catalog>,
        sweep: &'a SweepId,
        account: &'a AccountId,
        home: &'a RegionId,
    ) -> Self {
        Self {
            catalog,
            parent_plan,
            sweep,
            account,
            home,
        }
    }
}

pub(super) fn work(
    context: &Context<'_, '_>,
    rows: &[Record],
    observations: &[Observation],
) -> Result<Vec<WorkSpec>, SweepError> {
    let mut children = Vec::new();
    for (row, observation) in rows.iter().zip(observations) {
        for child in context
            .catalog
            .plans()
            .iter()
            .filter(|plan| matches!(plan.purpose(), PlanPurpose::Enumeration))
        {
            let Some(work) = child_for_row(context, child, row, observation)? else {
                continue;
            };
            if children.len() >= MAX_PAGE_CHILDREN {
                return Err(SweepError::Operation(
                    "page exceeds the atomic child-work bound",
                ));
            }
            children.try_reserve(1)?;
            children.push(work);
        }
    }
    Ok(children)
}

fn child_for_row(
    context: &Context<'_, '_>,
    child: &CallablePlan<'_>,
    row: &Record,
    observation: &Observation,
) -> Result<Option<WorkSpec>, SweepError> {
    let mut fields = Vec::new();
    let mut dependent = false;
    for binding in child.bindings() {
        let (is_dependent, value) =
            child_value(context, child.scope(), row, observation, &binding.source)?;
        let Some(value) = value else {
            return Ok(None);
        };
        dependent |= is_dependent;
        fields.try_reserve(1)?;
        fields.push(input::InputField {
            path: binding.path.clone(),
            value,
        });
    }
    if !dependent {
        return Ok(None);
    }
    make_child(context, child, observation, fields).map(Some)
}

fn child_value(
    context: &Context<'_, '_>,
    child_scope: PlanScope,
    row: &Record,
    observation: &Observation,
    source: &BindingSource,
) -> Result<(bool, Option<Value>), SweepError> {
    match source {
        BindingSource::Account => Ok((
            false,
            Some(Value::Text(context.account.as_str().to_owned())),
        )),
        BindingSource::Region => Ok((
            false,
            Some(Value::Text(
                binding_region(child_scope, context, observation).to_owned(),
            )),
        )),
        BindingSource::Fixed(value) => Ok((false, Some(fixed_value(value)))),
        BindingSource::ParentIdentity { path } => row_value(row, path),
        BindingSource::Operation {
            service,
            operation,
            path,
        } => {
            if service.as_ref() != context.parent_plan.service_name()
                || operation.as_ref() != context.parent_plan.operation_name()
            {
                return Ok((true, None));
            }
            row_value(row, path)
        }
    }
}
fn binding_region<'a>(
    scope: PlanScope,
    context: &'a Context<'_, '_>,
    observation: &'a Observation,
) -> &'a str {
    match scope {
        PlanScope::Account | PlanScope::HomeRegion => context.home.as_str(),
        PlanScope::Region => match &observation.key().region {
            Region::Global => context.home.as_str(),
            Region::Regional(region) => region.as_str(),
        },
    }
}

fn row_value(row: &Record, path: &[FieldId]) -> Result<(bool, Option<Value>), SweepError> {
    let value = record::path_value(row.root(), path)?;
    match value {
        Some(ValueRef::Null) | None => Ok((true, None)),
        Some(value) => Ok((true, Some(value.to_value()?))),
    }
}

fn fixed_value(value: &BindingValue) -> Value {
    match value {
        BindingValue::String(value) => Value::Text(value.clone()),
        BindingValue::Integer(value) => Value::Int(*value),
        BindingValue::Boolean(value) => Value::Bool(*value),
        BindingValue::Strings(values) => {
            Value::List(values.iter().cloned().map(Value::Text).collect())
        }
    }
}

fn child_region(scope: PlanScope, parent: &Region, home: &RegionId) -> Region {
    match scope {
        PlanScope::Account => Region::Global,
        PlanScope::HomeRegion => Region::Regional(home.clone()),
        PlanScope::Region => match parent {
            Region::Global => Region::Regional(home.clone()),
            Region::Regional(region) => Region::Regional(region.clone()),
        },
    }
}
fn make_child(
    context: &Context<'_, '_>,
    plan: &CallablePlan<'_>,
    observation: &Observation,
    fields: Vec<input::InputField>,
) -> Result<WorkSpec, SweepError> {
    let service = ServiceId::parse(plan.service_name()).map_err(SweepError::from)?;
    let operation = OperationId::parse(plan.operation_name()).map_err(SweepError::from)?;
    let parent: ResourceKey = observation.key().clone();
    let region = child_region(plan.scope(), &parent.region, context.home);
    let scope = ScopeKey::new(
        parent.account.clone(),
        region,
        service,
        operation,
        Some(parent),
    )
    .map_err(SweepError::from)?;
    let bindings = input::merge_input(&empty_record()?, &fields)?.into_bytes();
    WorkSpec::new(context.sweep.clone(), scope, None, bindings).map_err(SweepError::from)
}

fn empty_record() -> Result<Record, SweepError> {
    Record::encode(&Value::Map(std::collections::BTreeMap::new())).map_err(SweepError::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn child_binding_reads_actual_parent_identity() -> Result<(), Box<dyn std::error::Error>> {
        let field = FieldId::new(7)?;
        let map = std::collections::BTreeMap::from([(
            inventory_core::record::Key::Field(field),
            Value::Text("vpc-actual".to_owned()),
        )]);
        let row = Record::encode(&Value::Map(map))?;
        assert_eq!(
            row_value(&row, &[field])?,
            (true, Some(Value::Text("vpc-actual".to_owned())))
        );
        Ok(())
    }

    #[test]
    fn child_scope_obeys_compiled_region_class() -> Result<(), Box<dyn std::error::Error>> {
        let home = RegionId::parse("us-east-1")?;
        assert_eq!(
            child_region(PlanScope::Account, &Region::Regional(home.clone()), &home),
            Region::Global
        );
        assert_eq!(
            child_region(PlanScope::HomeRegion, &Region::Global, &home),
            Region::Regional(home)
        );
        Ok(())
    }
}
