use super::*;
use inventory_core::catalog::runtime::{BindingAlternativePlan, ContextSource, UrlResourceKind};

pub(super) fn consumes_current(
    source: &BindingSource,
    parent: &CallablePlan<'_>,
    branch: Option<&str>,
) -> bool {
    match source {
        BindingSource::ParentIdentity { .. } => true,
        BindingSource::ContextFromBranch { service, operation, branch: producer_branch, .. } => {
            matches_producer(service, operation, parent) && branch == Some(producer_branch.as_ref())
        }
        BindingSource::Alternatives(alternatives) => alternatives.iter().any(|alternative| {
            matches_producer(&alternative.service, &alternative.operation, parent)
        }),
        BindingSource::Transformed { source, .. } => consumes_current(source, parent, branch),
        _ => producer(source).is_some_and(|(service, operation)| matches_producer(service, operation, parent)),
    }
}

fn producer(source: &BindingSource) -> Option<(&str, &str)> {
    match source {
        BindingSource::Operation { service, operation, .. }
        | BindingSource::OperationArn { service, operation, .. }
        | BindingSource::OperationUrl { service, operation, .. }
        | BindingSource::Context { service, operation, .. } => Some((service, operation)),
        _ => None,
    }
}

fn matches_producer(service: &str, operation: &str, parent: &CallablePlan<'_>) -> bool {
    service == parent.service_name() && operation == parent.operation_name()
}

pub(crate) fn resolve_binding(
    context: &Context<'_, '_>,
    request_region: &RegionId,
    row: &RowContext<'_>,
    source: &BindingSource,
) -> Result<(bool, Option<Value>), SweepError> {
    match source {
        BindingSource::Account => Ok((false, Some(Value::Text(context.account.as_str().to_owned())))),
        BindingSource::Region => Ok((false, Some(Value::Text(request_region.as_str().to_owned())))),
        BindingSource::RegionList => Ok((false, Some(Value::List(vec![Value::Text(request_region.as_str().to_owned())])))),
        BindingSource::Fixed(value) => Ok((false, Some(crate::request::fixed_input::value(value)?))),
        BindingSource::ParentIdentity { path } => row_value(row.row, path),
        _ => binding_value(context, request_region, row, source),
    }
}

fn binding_value(
    context: &Context<'_, '_>,
    request_region: &RegionId,
    row: &RowContext<'_>,
    source: &BindingSource,
) -> Result<(bool, Option<Value>), SweepError> {
    match source {
        BindingSource::Alternatives(alternatives) => alternative(context, row, alternatives),
        BindingSource::Transformed { source, transform } => {
            let (dependent, value) = resolve_binding(context, request_region, row, source)?;
            Ok((dependent, value.map(|value| input::transform_value(value, transform)).transpose()?))
        }
        BindingSource::ContextFromBranch { service, operation, branch, source, path } => {
            stage_value(context, row, service, operation, branch, *source, path)
        }
        _ => ancestor_or_operation(context, request_region, row, source),
    }
}

fn alternative(
    context: &Context<'_, '_>,
    row: &RowContext<'_>,
    alternatives: &[BindingAlternativePlan],
) -> Result<(bool, Option<Value>), SweepError> {
    let Some(alternative) = alternatives.iter().find(|alternative| {
        matches_producer(&alternative.service, &alternative.operation, context.parent_plan)
    }) else { return Ok((false, None)); };
    if alternative.source == ContextSource::SelectedItem {
        return row_value(row.row, &alternative.path);
    }
    let value = row.ancestry.value(
        &alternative.service, &alternative.operation, alternative.source, &alternative.path,
    ).map_err(|_| SweepError::Operation("reviewed alternative binding is invalid"))?;
    resolved(true, value)
}

fn stage_value(
    context: &Context<'_, '_>,
    row: &RowContext<'_>,
    service: &str,
    operation: &str,
    branch: &str,
    source: ContextSource,
    path: &[FieldId],
) -> Result<(bool, Option<Value>), SweepError> {
    let current = matches_producer(service, operation, context.parent_plan)
        && row.scope.branch() == Some(branch);
    let value = row.ancestry.value_from_branch(service, operation, branch, source, path)
        .map_err(|_| SweepError::Operation("reviewed stage binding is invalid"))?;
    resolved(current, value)
}

fn ancestor_or_operation(
    context: &Context<'_, '_>,
    request_region: &RegionId,
    row: &RowContext<'_>,
    source: &BindingSource,
) -> Result<(bool, Option<Value>), SweepError> {
    match source {
        BindingSource::Context { service, operation, source, path } => {
            ancestor_value(context, row, service, operation, *source, path)
        }
        _ => operation_value(context, request_region, row, source),
    }
}

fn ancestor_value(
    context: &Context<'_, '_>,
    row: &RowContext<'_>,
    service: &str,
    operation: &str,
    source: ContextSource,
    path: &[FieldId],
) -> Result<(bool, Option<Value>), SweepError> {
    let current = matches_producer(service, operation, context.parent_plan);
    let value = row.ancestry.value(service, operation, source, path)
        .map_err(|_| SweepError::Operation("reviewed ancestor binding is invalid"))?;
    resolved(current, value)
}

fn operation_value(
    context: &Context<'_, '_>,
    request_region: &RegionId,
    row: &RowContext<'_>,
    source: &BindingSource,
) -> Result<(bool, Option<Value>), SweepError> {
    if !consumes_current(source, context.parent_plan, row.scope.branch()) {
        return Ok((false, None));
    }
    match source {
        BindingSource::Operation { path, .. } => row_value(row.row, path),
        BindingSource::OperationArn { path, arn_service, resource_kind, .. } => {
            eligibility::value(row.row, path, arn_service, *resource_kind, row.scope).map(|value| (true, value))
        }
        BindingSource::OperationUrl { path, resource_kind, .. } => {
            trusted_url(context, request_region, row, path, *resource_kind)
        }
        _ => Ok((false, None)),
    }
}

fn trusted_url(
    context: &Context<'_, '_>,
    request_region: &RegionId,
    row: &RowContext<'_>,
    path: &[FieldId],
    resource_kind: UrlResourceKind,
) -> Result<(bool, Option<Value>), SweepError> {
    let Some(ValueRef::Text(url)) = record::path_value(row.row.root(), path)? else {
        return Err(SweepError::Operation("reviewed parent URL is missing or not text"));
    };
    let trusted = record::IdentityContext { account: context.account, region: request_region };
    match resource_kind {
        UrlResourceKind::SqsQueue => { record::trusted_arn::queue_url(url, &trusted)?; }
    }
    Ok((true, Some(Value::Text(url.to_owned()))))
}

fn resolved(current: bool, value: Option<ValueRef<'_>>) -> Result<(bool, Option<Value>), SweepError> {
    match value {
        None | Some(ValueRef::Null) => Ok((current, None)),
        Some(value) => Ok((current, Some(value.to_value()?))),
    }
}

pub(super) fn row_value(row: &Record, path: &[FieldId]) -> Result<(bool, Option<Value>), SweepError> {
    resolved(true, record::path_value(row.root(), path)?)
}
