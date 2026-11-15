#[cfg(test)]
mod scalar_tests;
#[cfg(test)]
pub(in crate::sweep_runtime) mod policy_dfgk_tests;
#[cfg(test)]
mod binding_tests;
mod eligibility;
mod context_budget;
pub(super) use context_budget::{PageContextBudget, Rows};

use inventory_core::{
    catalog::runtime::{BindingSource, CallablePlan, ExecutionBranch, PlanPurpose, PlanScope},
    inventory::{AccountId, DiscoveryContext, DiscoveryFrame, Observation, OperationId, Region, RegionId, ResourceKey, ScopeKey, ServiceId},
    record::{FieldId, Record, Value, ValueRef},
};
use inventory_engine::inventory::{SweepId, WorkSpec};
use crate::sweep_runtime::{SweepError, input, record};
use super::consumer_index::ConsumerIndex;

const MAX_PAGE_CHILDREN: usize = 10_000;

pub(in crate::sweep_runtime) struct Context<'a, 'catalog> {
    consumers: &'a ConsumerIndex<'a,'catalog>,
    parent_plan: &'a CallablePlan<'catalog>,
    sweep: &'a SweepId,
    account: &'a AccountId,
    home: &'a RegionId,
}

impl<'a, 'catalog> Context<'a, 'catalog> {
    pub(in crate::sweep_runtime) fn new(parent_plan: &'a CallablePlan<'catalog>,
        sweep: &'a SweepId, account: &'a AccountId, home: &'a RegionId,
        consumers: &'a ConsumerIndex<'a,'catalog>) -> Self {
        Self { consumers, parent_plan, sweep, account, home }
    }
}

pub(in crate::sweep_runtime) struct RowContext<'a> {
    pub row: &'a Record,
    pub parent: Option<&'a ResourceKey>,
    pub scope: &'a ScopeKey,
    pub ancestry: &'a DiscoveryContext,
}

#[cfg(test)]
pub(super) fn contexts(
    plan: &CallablePlan<'_>, rows: &[Record], request: &Record, response: &Record,
    ancestry: &DiscoveryContext, scope: &ScopeKey,
) -> Result<Vec<DiscoveryContext>,SweepError> {
    let mut budget = PageContextBudget::new(ancestry, rows.len())?;
    contexts_from(plan, Rows { items: rows, offset: 0 }, request, response, ancestry, scope, &mut budget)
}

pub(super) fn contexts_from(
    plan: &CallablePlan<'_>, rows: Rows<'_>, request: &Record, response: &Record,
    ancestry: &DiscoveryContext, scope: &ScopeKey, budget: &mut PageContextBudget,
) -> Result<Vec<DiscoveryContext>, SweepError> {
    let end = rows.offset.checked_add(rows.items.len())
        .ok_or(SweepError::Operation("discovery item ordinal exceeds bound"))?;
    contexts_at(plan,(rows.offset..end).zip(rows.items.iter()),request,response,ancestry,scope,budget)
}

pub(super) fn contexts_at<'row>(
    plan: &CallablePlan<'_>, rows: impl ExactSizeIterator<Item = (usize,&'row Record)>,
    request: &Record, response: &Record, ancestry: &DiscoveryContext, scope: &ScopeKey,
    budget: &mut PageContextBudget,
) -> Result<Vec<DiscoveryContext>, SweepError> {
    let mut contexts = Vec::new();
    contexts.try_reserve(rows.len())?;
    for (ordinal,row) in rows {
        let ordinal = u32::try_from(ordinal)
            .map_err(|_| SweepError::Operation("discovery item ordinal exceeds bound"))?;
        let frame = DiscoveryFrame::project(
            scope.service().clone(), scope.operation().clone(), ordinal,
            plan.context_projection(), row, response, request,
        ).and_then(|frame| frame.with_branch(scope.branch()))
            .map_err(|_| SweepError::Operation("reviewed discovery projection failed"))?;
        let bytes = ancestry.pushed_encoded_len(&frame)
            .map_err(|_| SweepError::Operation("discovery ancestry exceeds bounds"))?;
        budget.charge(bytes)?;
        contexts.push(ancestry.push(frame).map_err(|_| SweepError::Operation("discovery ancestry exceeds bounds"))?);
    }
    Ok(contexts)
}

pub(super) fn work(
    context: &Context<'_, '_>,
    rows: &[Record],
    observations: &[Observation],
    request: &Record,
    contexts: &[DiscoveryContext],
    scope: &ScopeKey,
    budget: &mut PageContextBudget,
) -> Result<Vec<WorkSpec>, SweepError> {
    let mut children = Vec::new();
    for (ordinal, row) in rows.iter().enumerate() {
        let next = contexts.get(ordinal).ok_or(SweepError::Operation("discovery item context is absent"))?;
        let discovery = next.encode().map_err(|_| SweepError::Operation("discovery ancestry encoding failed"))?;
        let observation = observations.get(ordinal);
        let row_context = RowContext { row, parent:observation.map(Observation::key), scope, ancestry:next };
        for child in context.consumers.candidates_for_branch(context.parent_plan.service_name(),context.parent_plan.operation_name(),scope.branch()) {
            match child.purpose() {
                PlanPurpose::Enumeration => {
                    for branch in child.branches().iter().map(Some)
                        .chain(std::iter::once(None).filter(|_| child.branches().is_empty())) {
                        if let Some(work) = child_for_row(context,child,&row_context,branch,&discovery,budget)? {
                            push(&mut children,work)?;
                        }
                    }
                }
                PlanPurpose::TagEndpoint(_) => {
                    if let Some(observation) = observation {
                        let target_stage = scope.branch().filter(|source| child.branches().iter()
                            .any(|target| target.discriminator.as_str() == *source));
                        for branch in child.branches().iter()
                            .filter(|branch| target_stage.is_none_or(|source| branch.discriminator.as_str() == source))
                            .map(Some)
                            .chain(child.branches().is_empty().then_some(None)) {
                            if !crate::sweep_runtime::row_policy::child(child,branch,context.parent_plan,
                                row,row_context.ancestry,scope)? { continue; }
                            let tag_context = super::tag_endpoint::ChildContext {
                                parent_plan:context.parent_plan, plan:child, request, sweep:context.sweep,
                                discovery:&discovery, branch,
                            };
                            if let Some(work) = super::tag_endpoint::child_work(&tag_context,row,observation,budget)? {
                                push(&mut children,work)?;
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
    Ok(children)
}

fn push(children: &mut Vec<WorkSpec>, work: WorkSpec) -> Result<(),SweepError> {
    if children.len() >= MAX_PAGE_CHILDREN { return Err(SweepError::Operation("page exceeds the atomic child-work bound")); }
    children.try_reserve(1)?;
    children.push(work);
    Ok(())
}

fn child_for_row(context: &Context<'_, '_>, child: &CallablePlan<'_>, row: &RowContext<'_>,
    branch: Option<&ExecutionBranch>, discovery: &[u8], budget: &mut PageContextBudget) -> Result<Option<WorkSpec>, SweepError> {
    if !child.effective_bindings(branch)
        .any(|binding| consumes_current(&binding.source,context.parent_plan,row.scope.branch())) { return Ok(None); }
    if !crate::sweep_runtime::row_policy::child(child,branch,context.parent_plan,
        row.row,row.ancestry,row.scope)? { return Ok(None); }
    let scope_kind = branch.map_or(child.scope(),|branch| branch.scope);
    let scope_region = child_region(scope_kind,row.scope.region(),context.home);
    let request_region = super::super::routing::request_region(child,&scope_region,context.home,
        branch.map(|branch| branch.discriminator.as_str()))?;
    let mut fields = Vec::new();
    for binding in child.effective_bindings(branch) {
        let (_,value) = resolve_binding(context,&request_region,row,&binding.source)?;
        let Some(value) = value else { return Ok(None); };
        fields.try_reserve(1)?;
        fields.push(input::InputField { path:binding.path.clone(), value });
    }
    budget.charge(discovery.len())?;
    make_child(context,child,row,branch,fields,discovery).map(Some)
}

fn consumes_current(source: &BindingSource, parent: &CallablePlan<'_>, branch: Option<&str>) -> bool {
    match source {
        BindingSource::ParentIdentity { .. } => true,
        BindingSource::ContextFromBranch { service, operation, branch: producer_branch, .. } =>
            service.as_ref() == parent.service_name() && operation.as_ref() == parent.operation_name()
                && branch == Some(producer_branch.as_ref()),
        BindingSource::Operation { service,operation,.. }
        | BindingSource::OperationArn { service,operation,.. }
        | BindingSource::OperationUrl { service,operation,.. }
        | BindingSource::Context { service,operation,.. } =>
            service.as_ref() == parent.service_name() && operation.as_ref() == parent.operation_name(),
        BindingSource::Alternatives(alternatives) => alternatives.iter().any(|alternative|
            alternative.service.as_ref() == parent.service_name() && alternative.operation.as_ref() == parent.operation_name()),
        BindingSource::Transformed { source,.. } => consumes_current(source,parent,branch),
        BindingSource::Account | BindingSource::Region | BindingSource::RegionList | BindingSource::Fixed(_) => false,
    }
}

pub(in crate::sweep_runtime) fn resolve_binding(context: &Context<'_, '_>, request_region: &RegionId, row: &RowContext<'_>,
    source: &BindingSource) -> Result<(bool,Option<Value>),SweepError> {
    match source {
        BindingSource::Account => Ok((false,Some(Value::Text(context.account.as_str().to_owned())))),
        BindingSource::Region => Ok((false,Some(Value::Text(request_region.as_str().to_owned())))),
        BindingSource::RegionList => Ok((false,Some(Value::List(vec![Value::Text(request_region.as_str().to_owned())])))),
        BindingSource::Alternatives(alternatives) => {
            let Some(alternative) = alternatives.iter().find(|alternative|
                alternative.service.as_ref() == context.parent_plan.service_name()
                    && alternative.operation.as_ref() == context.parent_plan.operation_name()) else {
                return Ok((false,None));
            };
            if alternative.source == inventory_core::catalog::runtime::ContextSource::SelectedItem {
                return row_value(row.row, &alternative.path);
            }
            let value = row.ancestry.value(&alternative.service,&alternative.operation,alternative.source,&alternative.path)
                .map_err(|_| SweepError::Operation("reviewed alternative binding is invalid"))?;
            match value {
                None | Some(ValueRef::Null) => Ok((true,None)),
                Some(value) => Ok((true,Some(value.to_value()?))),
            }
        }
        BindingSource::ContextFromBranch { service,operation,branch,source,path } => {
            let current = service.as_ref() == context.parent_plan.service_name()
                && operation.as_ref() == context.parent_plan.operation_name() && row.scope.branch() == Some(branch.as_ref());
            let value = row.ancestry.value_from_branch(service,operation,branch,*source,path)
                .map_err(|_| SweepError::Operation("reviewed stage binding is invalid"))?;
            match value {
                None | Some(ValueRef::Null) => Ok((current,None)),
                Some(value) => Ok((current,Some(value.to_value()?))),
            }
        }
        BindingSource::Fixed(value) => Ok((false,Some(crate::sweep_runtime::request::fixed_input::value(value)?))),
        BindingSource::ParentIdentity { path } => row_value(row.row,path),
        BindingSource::Operation { service,operation,path } => {
            if service.as_ref() != context.parent_plan.service_name() || operation.as_ref() != context.parent_plan.operation_name() {
                return Ok((false,None));
            }
            row_value(row.row,path)
        }
        BindingSource::OperationArn { service,operation,path,arn_service,resource_kind } => {
            if service.as_ref() != context.parent_plan.service_name() || operation.as_ref() != context.parent_plan.operation_name() {
                return Ok((false,None));
            }
            eligibility::value(row.row,path,arn_service,*resource_kind,row.scope).map(|value| (true,value))
        }
        BindingSource::Transformed { source,transform } => {
            let (dependent,value) = resolve_binding(context,request_region,row,source)?;
            Ok((dependent,value.map(|value| input::transform_value(value,transform)).transpose()?))
        }
        BindingSource::OperationUrl { service,operation,path,resource_kind } => {
            if service.as_ref() != context.parent_plan.service_name() || operation.as_ref() != context.parent_plan.operation_name() {
                return Ok((false,None));
            }
            let Some(ValueRef::Text(url)) = record::path_value(row.row.root(),path)? else {
                return Err(SweepError::Operation("reviewed parent URL is missing or not text"));
            };
            let trusted = record::IdentityContext { account:context.account,region:request_region };
            match resource_kind {
                inventory_core::catalog::runtime::UrlResourceKind::SqsQueue => { record::trusted_arn::queue_url(url,&trusted)?; }
            }
            Ok((true,Some(Value::Text(url.to_owned()))))
        }
        BindingSource::Context { service,operation,source,path } => {
            let current = service.as_ref() == context.parent_plan.service_name() && operation.as_ref() == context.parent_plan.operation_name();
            let value = row.ancestry.value(service,operation,*source,path)
                .map_err(|_| SweepError::Operation("reviewed ancestor binding is invalid"))?;
            match value {
                None | Some(ValueRef::Null) => Ok((current,None)),
                Some(value) => Ok((current,Some(value.to_value()?))),
            }
        }
    }
}

fn row_value(row: &Record, path: &[FieldId]) -> Result<(bool, Option<Value>), SweepError> {
    match record::path_value(row.root(), path)? {
        Some(ValueRef::Null) | None => Ok((true,None)),
        Some(value) => Ok((true,Some(value.to_value()?))),
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

fn make_child(context: &Context<'_, '_>, plan: &CallablePlan<'_>, row: &RowContext<'_>,
    branch: Option<&ExecutionBranch>, fields: Vec<input::InputField>, discovery: &[u8]) -> Result<WorkSpec, SweepError> {
    let region = child_region(branch.map_or(plan.scope(),|branch| branch.scope),row.scope.region(),context.home);
    let mut scope = ScopeKey::new(context.account.clone(),region,ServiceId::parse(plan.service_name())?,
        OperationId::parse(plan.operation_name())?,row.parent.or(row.scope.parent()).cloned())?;
    if let Some(branch) = branch.map(|branch| branch.discriminator.as_str()) {
        scope = scope.with_branch(branch.to_owned())?;
    }
    scope = scope.with_context(row.ancestry.discriminator().map_err(|_| SweepError::Operation("discovery context digest failed"))?);
    let bindings = input::merge_input(&empty_record()?, &fields)?.into_bytes();
    WorkSpec::new(context.sweep.clone(),scope,None,bindings)?.with_context(discovery.to_vec()).map_err(SweepError::from)
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

#[cfg(test)]
mod policy_lp_completion_tests;
#[cfg(test)]
mod policy_tz_completion_tests;
#[cfg(test)]
mod policy_qs_tests;
#[cfg(test)]
mod policy_gk_children_tests;
