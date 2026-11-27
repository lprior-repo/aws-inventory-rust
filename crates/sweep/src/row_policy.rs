#[cfg(test)]
mod tests;
use super::{SweepError, record};
use inventory_core::{
    catalog::runtime::{
        CallablePlan, ContextSource, ExecutionBranch, MissingSelected, RowPolicyPlan,
        RowPredicatePlan, RowValueType,
    },
    inventory::DiscoveryContext,
    record::{FieldId, MapRef, Record, ValueRef},
};

pub(crate) fn selected(
    plan: &CallablePlan<'_>,
    branch: Option<&ExecutionBranch>,
    row: &Record,
    request: &Record,
    response: &Record,
) -> Result<bool, SweepError> {
    let read = |source, path: &[FieldId]| {
        lookup(
            match source {
                ContextSource::SelectedItem => row,
                ContextSource::Request => request,
                ContextSource::Response => response,
            },
            path,
        )
    };
    select(policies(plan, branch), &read)
}

fn policies<'a, 'catalog>(
    plan: &'a CallablePlan<'catalog>,
    branch: Option<&'a ExecutionBranch>,
) -> impl Iterator<Item = &'a RowPolicyPlan> + Clone {
    std::iter::once(plan.row_policy()).chain(branch.and_then(|branch| branch.row_policy.as_ref()))
}

fn select<'policy, 'value>(
    mut policies: impl Iterator<Item = &'policy RowPolicyPlan> + Clone,
    read: &impl Fn(ContextSource, &[FieldId]) -> Result<Option<ValueRef<'value>>, SweepError>,
) -> Result<bool, SweepError> {
    policies
        .clone()
        .flat_map(|policy| policy.assertions.iter())
        .try_for_each(|predicate| {
            if evaluate(predicate, read)? {
                Ok(())
            } else {
                Err(SweepError::Operation(
                    "returned row disagrees with its bound request parent",
                ))
            }
        })?;
    policies.try_fold(true, |owned, policy| {
        Ok(owned
            & policy
                .owned
                .as_ref()
                .map_or(Ok(true), |predicate| evaluate(predicate, read))?)
    })
}

pub(crate) fn child(
    plan: &CallablePlan<'_>,
    branch: Option<&ExecutionBranch>,
    parent: &CallablePlan<'_>,
    row: &Record,
    ancestry: &DiscoveryContext,
    parent_branch: Option<&str>,
) -> Result<bool, SweepError> {
    policies(plan, branch)
        .filter_map(|policy| policy.child.as_ref())
        .try_fold(true, |eligible, eligibility| {
            if parent.service_name() != plan.service_name()
                || parent.operation_name() != eligibility.operation.as_ref()
            {
                return Ok(false);
            }
            let matches = evaluate(&eligibility.predicate, &|source, path| {
                if source == ContextSource::SelectedItem {
                    return lookup(row, path);
                }
                ancestry_value(parent, ancestry, parent_branch, source, path)
            })?;
            Ok(eligible & matches)
        })
}

fn ancestry_value<'a>(
    parent: &CallablePlan<'_>,
    ancestry: &'a DiscoveryContext,
    branch: Option<&str>,
    source: ContextSource,
    path: &[FieldId],
) -> Result<Option<ValueRef<'a>>, SweepError> {
    let value = match branch {
        Some(branch) => ancestry.value_from_branch(
            parent.service_name(),
            parent.operation_name(),
            branch,
            source,
            path,
        ),
        None => ancestry.value(parent.service_name(), parent.operation_name(), source, path),
    };
    value.map_err(|_| SweepError::Operation("child eligibility context is invalid"))
}

fn evaluate<'a>(
    predicate: &RowPredicatePlan,
    read: &impl Fn(ContextSource, &[FieldId]) -> Result<Option<ValueRef<'a>>, SweepError>,
) -> Result<bool, SweepError> {
    match predicate {
        RowPredicatePlan::Absent(operand) => match read(operand.source, &operand.path)? {
            None => Ok(true),
            Some(value) => {
                typed(value, operand.value_type)?;
                Ok(false)
            }
        },
        RowPredicatePlan::BooleanEquals { operand, value } => boolean(operand, *value, read),
        RowPredicatePlan::StringNotIn { operand, values } => {
            string_not_in(operand, values, read)
        }
        RowPredicatePlan::StringIn { operand, values } => {
            string_in(operand, values, read)
        }
        RowPredicatePlan::SelectedRequestEquals {
            selected,
            request,
            value_type,
            missing,
        } => selected_equals(selected, request, *value_type, *missing, read),
    }
}

fn string_not_in<'a>(
    operand: &inventory_core::catalog::runtime::RowOperandPlan,
    values: &[Box<str>],
    read: &impl Fn(ContextSource, &[FieldId]) -> Result<Option<ValueRef<'a>>, SweepError>,
) -> Result<bool, SweepError> {
    let Some(actual) = read(operand.source, &operand.path)? else {
        return Ok(false);
    };
    typed(actual, RowValueType::String)?;
    match actual {
        ValueRef::Text(actual) => Ok(!values.iter().any(|value| value.as_ref() == actual)),
        _ => Err(SweepError::Operation("row policy string has invalid type")),
    }
}

fn string_in<'a>(
    operand: &inventory_core::catalog::runtime::RowOperandPlan,
    values: &[Box<str>],
    read: &impl Fn(ContextSource, &[FieldId]) -> Result<Option<ValueRef<'a>>, SweepError>,
) -> Result<bool, SweepError> {
    let Some(actual) = read(operand.source, &operand.path)? else {
        return Ok(false);
    };
    typed(actual, RowValueType::String)?;
    match actual {
        ValueRef::Text(actual) => Ok(values.iter().any(|value| value.as_ref() == actual)),
        _ => Err(SweepError::Operation("row policy string has invalid type")),
    }
}

fn boolean<'a>(
    operand: &inventory_core::catalog::runtime::RowOperandPlan,
    expected: bool,
    read: &impl Fn(ContextSource, &[FieldId]) -> Result<Option<ValueRef<'a>>, SweepError>,
) -> Result<bool, SweepError> {
    let actual = read(operand.source, &operand.path)?
        .ok_or(SweepError::Operation("row policy boolean is absent"))?;
    typed(actual, RowValueType::Boolean)?;
    match actual {
        ValueRef::Bool(actual) => Ok(actual == expected),
        _ => Err(SweepError::Operation("row policy boolean has invalid type")),
    }
}

fn selected_equals<'a>(
    selected: &[FieldId],
    request: &[FieldId],
    value_type: RowValueType,
    missing: MissingSelected,
    read: &impl Fn(ContextSource, &[FieldId]) -> Result<Option<ValueRef<'a>>, SweepError>,
) -> Result<bool, SweepError> {
    let request = read(ContextSource::Request, request)?
        .ok_or(SweepError::Operation("row policy request parent is absent"))?;
    typed(request, value_type)?;
    match read(ContextSource::SelectedItem, selected)? {
        None if missing == MissingSelected::AllowAbsent => Ok(true),
        None => Err(SweepError::Operation(
            "row policy returned parent is absent",
        )),
        Some(selected) => {
            typed(selected, value_type)?;
            equal(selected, request)
        }
    }
}

fn typed(value: ValueRef<'_>, kind: RowValueType) -> Result<(), SweepError> {
    match (kind, value) {
        (RowValueType::String, ValueRef::Text(_))
        | (RowValueType::Boolean, ValueRef::Bool(_))
        | (RowValueType::Structure, ValueRef::Map(_)) => Ok(()),
        _ => Err(SweepError::Operation(
            "row policy value is null or has invalid modeled type",
        )),
    }
}

fn equal(selected: ValueRef<'_>, request: ValueRef<'_>) -> Result<bool, SweepError> {
    match (selected, request) {
        (ValueRef::Text(left), ValueRef::Text(right)) => Ok(left == right),
        _ => Err(SweepError::Operation(
            "row policy equality has invalid modeled operands",
        )),
    }
}

fn lookup<'a>(value: &'a Record, path: &[FieldId]) -> Result<Option<ValueRef<'a>>, SweepError> {
    if path.is_empty() {
        return record::path_value(value.root(), path);
    }
    let root = match value.root().get_name("$value")? {
        Some(ValueRef::Map(root)) => root,
        Some(_) => {
            return Err(SweepError::Operation(
                "row policy selected container has invalid type",
            ));
        }
        None => value.root(),
    };
    let (leaf, parents) = path
        .split_last()
        .ok_or(SweepError::Operation("row policy path is absent"))?;
    let parent = parents
        .iter()
        .try_fold(Some(root), |current, id| descend(current, *id))?;
    parent.map_or(Ok(None), |parent| {
        parent.get_field(*leaf).map_err(SweepError::from)
    })
}

fn descend(current: Option<MapRef<'_>>, id: FieldId) -> Result<Option<MapRef<'_>>, SweepError> {
    match current
        .map(|current| current.get_field(id))
        .transpose()?
        .flatten()
    {
        None => Ok(None),
        Some(ValueRef::Map(map)) => Ok(Some(map)),
        Some(_) => Err(SweepError::Operation(
            "row policy path traverses a null or non-structure value",
        )),
    }
}
