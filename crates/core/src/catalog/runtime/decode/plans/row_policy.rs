use super::{metadata, tags, CallablePlan, PlanParts, RuntimeCatalogError};
use crate::{catalog::{runtime::{BindingSource, ChildEligibilityPlan, ContextSource, MissingSelected, RowOperandPlan, RowPolicyPlan, RowPredicatePlan, RowValueType}, Service, Target}, record::{FieldId, MapRef, ValueRef}};

pub(super) fn read(fields: MapRef<'_>) -> Result<RowPolicyPlan, RuntimeCatalogError> {
    let Some(value) = fields.get_field(FieldId::new(22)?)? else { return Ok(RowPolicyPlan::default()); };
    read_value(value)
}

pub(super) fn read_value(value: ValueRef<'_>) -> Result<RowPolicyPlan, RuntimeCatalogError> {
    let fields = super::map(value)?;
    metadata::only_fields(fields,&[1,2,3])?;
    let owned = fields.get_field(FieldId::new(1)?)?.map(predicate).transpose()?;
    let assertions = metadata::bounded(metadata::list(fields,2)?,predicate,8)?.into_boxed_slice();
    let child = fields.get_field(FieldId::new(3)?)?.map(|value| {
        let fields = super::map(value)?;
        metadata::only_fields(fields,&[1,2])?;
        let operation = metadata::text(fields,1)?;
        if operation.is_empty() || operation.len() > 256 { return Err(RuntimeCatalogError::Capability); }
        Ok::<_,RuntimeCatalogError>(ChildEligibilityPlan { operation:operation.into_boxed_str(), predicate:predicate(metadata::field(fields,2)?)? })
    }).transpose()?;
    Ok(RowPolicyPlan { owned,assertions,child })
}

fn predicate(value: ValueRef<'_>) -> Result<RowPredicatePlan, RuntimeCatalogError> {
    let fields = super::map(value)?;
    match metadata::integer(fields,1)? {
        0 => { metadata::only_fields(fields,&[1,2])?; Ok(RowPredicatePlan::Absent(operand(metadata::field(fields,2)?)?)) }
        1 => {
            metadata::only_fields(fields,&[1,2,3])?;
            let ValueRef::Bool(value) = metadata::field(fields,3)? else { return Err(RuntimeCatalogError::Capability); };
            let operand = operand(metadata::field(fields,2)?)?;
            if operand.value_type != RowValueType::Boolean { return Err(RuntimeCatalogError::Capability); }
            Ok(RowPredicatePlan::BooleanEquals { operand,value })
        }
        2 => {
            metadata::only_fields(fields,&[1,2,3,4,5])?;
            let value_type = value_type(metadata::integer(fields,4)?)?;
            if value_type != RowValueType::String { return Err(RuntimeCatalogError::Capability); }
            Ok(RowPredicatePlan::SelectedRequestEquals { selected:metadata::selected_path(metadata::list(fields,2)?)?,
                request:metadata::selected_path(metadata::list(fields,3)?)?, value_type,
                missing:match metadata::integer(fields,5)? { 0 => MissingSelected::Reject, 1 => MissingSelected::AllowAbsent,
                    _ => return Err(RuntimeCatalogError::Capability) } })
        }
        _ => Err(RuntimeCatalogError::Capability),
    }
}

fn operand(value: ValueRef<'_>) -> Result<RowOperandPlan, RuntimeCatalogError> {
    let fields = super::map(value)?;
    metadata::only_fields(fields,&[1,2,3])?;
    Ok(RowOperandPlan { source:match metadata::integer(fields,1)? { 5 => ContextSource::SelectedItem,
        6 => ContextSource::Response, 7 => ContextSource::Request, _ => return Err(RuntimeCatalogError::Capability) },
        path:metadata::selected_path(metadata::list(fields,2)?)?, value_type:value_type(metadata::integer(fields,3)?)? })
}

fn value_type(value: i64) -> Result<RowValueType, RuntimeCatalogError> {
    match value { 0 => Ok(RowValueType::String), 1 => Ok(RowValueType::Boolean), 2 => Ok(RowValueType::Structure),
        _ => Err(RuntimeCatalogError::Capability) }
}

pub(super) fn validate(parts: &PlanParts<'_>) -> Result<(),RuntimeCatalogError> {
    let policies = std::iter::once(&parts.row_policy).chain(parts.branches.iter().filter_map(|branch| branch.row_policy.as_ref()));
    if !policies.clone().any(|policy| policy.owned.is_some() || !policy.assertions.is_empty()) { return Ok(()); }
    let selected = tags::selected_item(parts.service,parts.operation,&parts.items)?;
    policies.flat_map(|policy| policy.owned.iter().chain(policy.assertions.iter()))
        .try_for_each(|predicate| validate_predicate(parts.service,parts.operation,selected,predicate))
}

fn validate_predicate(service: Service<'_>, operation: crate::catalog::Operation<'_>, selected: Target,
    predicate: &RowPredicatePlan) -> Result<(),RuntimeCatalogError> {
    match predicate {
        RowPredicatePlan::Absent(operand) | RowPredicatePlan::BooleanEquals { operand,.. } => {
            let target = match operand.source { ContextSource::SelectedItem => selected,
                ContextSource::Request => operation.input()?, ContextSource::Response => operation.output()? };
            validate_type(service,target,&operand.path,operand.value_type)
        }
        RowPredicatePlan::SelectedRequestEquals { selected:path,request,value_type,.. } => {
            validate_type(service,selected,path,*value_type)?;
            validate_type(service,operation.input()?,request,*value_type)
        }
    }
}

fn validate_type(service: Service<'_>, target: Target, path: &[FieldId], value_type: RowValueType) -> Result<(),RuntimeCatalogError> {
    let kind = tags::target_kind(service,tags::resolve(service,target,path)?)?;
    match (value_type,kind) { (RowValueType::String,"string" | "enum") | (RowValueType::Boolean,"boolean")
        | (RowValueType::Structure,"structure" | "union") => Ok(()),
        _ => Err(RuntimeCatalogError::Capability) }
}

pub(super) fn validate_parents(plans: &[CallablePlan<'_>]) -> Result<(),RuntimeCatalogError> {
    plans.iter().try_for_each(|plan| {
        std::iter::once((None,plan.row_policy())).chain(plan.branches().iter()
            .filter_map(|branch| branch.row_policy.as_ref().map(|policy| (Some(branch),policy))))
            .try_for_each(|(branch,policy)| {
                let Some(child) = &policy.child else { return Ok(()); };
                if branch.is_some() && plan.row_policy().child.as_ref()
                    .is_some_and(|base| base.operation != child.operation) { return Err(RuntimeCatalogError::Capability); }
                let source = plans.iter().find(|source| source.service_name() == plan.service_name()
                    && source.operation_name() == child.operation.as_ref()).ok_or(RuntimeCatalogError::Capability)?;
                if !plan.bindings().iter().chain(plan.branches().iter()
                    .filter(|candidate| branch.is_none_or(|active| active.discriminator == candidate.discriminator))
                    .flat_map(|branch| branch.bindings.iter())).any(|binding| consumes(&binding.source,source)) {
                    return Err(RuntimeCatalogError::Capability);
                }
                let selected = tags::selected_item(source.service(),source.operation(),source.items())?;
                validate_predicate(source.service(),source.operation(),selected,&child.predicate)?;
                projected(source,&child.predicate)
            })
    })
}
fn projected(source: &CallablePlan<'_>, predicate: &RowPredicatePlan) -> Result<(),RuntimeCatalogError> {
    match predicate {
        RowPredicatePlan::Absent(operand) | RowPredicatePlan::BooleanEquals { operand,.. } =>
            projected_path(source,operand.source,&operand.path),
        RowPredicatePlan::SelectedRequestEquals { request,.. } => projected_path(source,ContextSource::Request,request),
    }
}

fn projected_path(source: &CallablePlan<'_>, namespace: ContextSource, path: &[FieldId]) -> Result<(),RuntimeCatalogError> {
    let projection = source.context_projection();
    let paths = match namespace { ContextSource::SelectedItem => return Ok(()),
        ContextSource::Request => &projection.request, ContextSource::Response => &projection.response };
    if paths.iter().any(|candidate| candidate.as_ref() == path) { Ok(()) } else { Err(RuntimeCatalogError::Capability) }
}

fn consumes(binding: &BindingSource, source: &CallablePlan<'_>) -> bool {
    let binding = match binding { BindingSource::Transformed { source,.. } => source.as_ref(), binding => binding };
    match binding {
        BindingSource::Operation { service,operation,.. } | BindingSource::OperationArn { service,operation,.. }
        | BindingSource::OperationUrl { service,operation,.. } | BindingSource::Context { service,operation,.. }
        | BindingSource::ContextFromBranch { service,operation,.. } => service.as_ref() == source.service_name() && operation.as_ref() == source.operation_name(),
        BindingSource::Alternatives(alternatives) => alternatives.iter().any(|alternative| alternative.service.as_ref() == source.service_name() && alternative.operation.as_ref() == source.operation_name()),
        _ => false,
    }
}
