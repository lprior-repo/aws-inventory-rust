use super::{BindingSource, CallablePlan, RuntimeCatalogError, tags};
use crate::{
    catalog::{
        Target,
        runtime::{
            ContextSource, IdentityChoicePlan, IdentityFieldPlan, IdentityLeafPlan, IdentityPlan,
        },
    },
    record::FieldId,
};

pub(super) fn validate(plans: &[CallablePlan<'_>]) -> Result<(), RuntimeCatalogError> {
    let mut ancestors = vec![false; plans.len()];
    for (child_index, child) in plans.iter().enumerate() {
        validate_alternative_sets(child)?;
        ancestors.fill(false);
        *ancestors
            .get_mut(child_index)
            .ok_or(RuntimeCatalogError::Bounds)? = true;
        for _ in 0..plans.len() {
            let mut changed = false;
            for (index, plan) in plans.iter().enumerate() {
                if !ancestors
                    .get(index)
                    .copied()
                    .ok_or(RuntimeCatalogError::Bounds)?
                {
                    continue;
                }
                for binding in plan.bindings().iter().chain(
                    plan.branches()
                        .iter()
                        .flat_map(|branch| branch.bindings.iter()),
                ) {
                    producers(&binding.source).try_for_each(|(service, operation)| {
                        let parent = parent_index(plans, service, operation)?;
                        let ancestor = ancestors
                            .get_mut(parent)
                            .ok_or(RuntimeCatalogError::Bounds)?;
                        if !*ancestor {
                            *ancestor = true;
                            changed = true;
                        }
                        Ok::<_, RuntimeCatalogError>(())
                    })?;
                }
            }
            if !changed {
                break;
            }
        }
        for binding in child.bindings().iter().chain(
            child
                .branches()
                .iter()
                .flat_map(|branch| branch.bindings.iter()),
        ) {
            if let BindingSource::Alternatives(alternatives) = base(&binding.source) {
                validate_alternatives(plans, child_index, child, binding, alternatives)?;
                continue;
            }
            let (service, operation, source, path) = match base(&binding.source) {
                BindingSource::Operation {
                    service,
                    operation,
                    path,
                }
                | BindingSource::OperationArn {
                    service,
                    operation,
                    path,
                    ..
                }
                | BindingSource::OperationUrl {
                    service,
                    operation,
                    path,
                    ..
                } => (service, operation, ContextSource::SelectedItem, path),
                BindingSource::Context {
                    service,
                    operation,
                    source,
                    path,
                } => (service, operation, *source, path),
                BindingSource::ContextFromBranch {
                    service,
                    operation,
                    source,
                    path,
                    branch,
                } => {
                    let parent = plans
                        .get(parent_index(plans, service, operation)?)
                        .ok_or(RuntimeCatalogError::Capability)?;
                    if !parent
                        .branches()
                        .iter()
                        .any(|stage| stage.discriminator.as_str() == branch.as_ref())
                    {
                        return Err(RuntimeCatalogError::Capability);
                    }
                    (service, operation, *source, path)
                }
                BindingSource::ParentIdentity { .. } => {
                    return Err(RuntimeCatalogError::Capability);
                }
                _ => continue,
            };
            let index = parent_index(plans, service, operation)?;
            if index == child_index
                && !matches!(
                    base(&binding.source),
                    BindingSource::ContextFromBranch { .. }
                )
            {
                return Err(RuntimeCatalogError::Capability);
            }
            let parent = plans.get(index).ok_or(RuntimeCatalogError::Capability)?;
            require_projection(parent, source, path)?;
            let source_kind = tags::target_kind(parent.service(), target(parent, source, path)?)?;
            let destination =
                tags::resolve(child.service(), child.operation().input()?, &binding.path)?;
            let destination_kind = tags::target_kind(child.service(), destination)?;
            if !compatible(source_kind, destination_kind) {
                return Err(RuntimeCatalogError::Capability);
            }
            if (matches!(
                base(&binding.source),
                BindingSource::OperationArn { .. } | BindingSource::OperationUrl { .. }
            ) || matches!(binding.source, BindingSource::Transformed { .. }))
                && (!matches!(source_kind, "string" | "enum")
                    || !matches!(destination_kind, "string" | "enum"))
            {
                return Err(RuntimeCatalogError::Capability);
            }
        }
        identity_fields(child.identity(), &mut |field| {
            let IdentityFieldPlan::Context {
                service,
                operation,
                source,
                path,
            } = field
            else {
                return Ok(());
            };
            let index = parent_index(plans, service, operation)?;
            if index == child_index {
                if !matches!(*source, ContextSource::Response | ContextSource::Request) {
                    return Err(RuntimeCatalogError::Capability);
                }
            } else if !ancestors
                .get(index)
                .copied()
                .ok_or(RuntimeCatalogError::Bounds)?
            {
                return Err(RuntimeCatalogError::Capability);
            }
            let parent = plans.get(index).ok_or(RuntimeCatalogError::Capability)?;
            require_projection(parent, *source, path)?;
            if !matches!(
                tags::target_kind(parent.service(), target(parent, *source, path)?)?,
                "string" | "enum" | "byte" | "short" | "integer" | "long" | "boolean"
            ) {
                return Err(RuntimeCatalogError::Capability);
            }
            Ok(())
        })?;
    }
    Ok(())
}

pub(crate) fn producers(source: &BindingSource) -> impl Iterator<Item = (&str, &str)> {
    let source = base(source);
    let single = match source {
        BindingSource::Operation {
            service, operation, ..
        }
        | BindingSource::OperationArn {
            service, operation, ..
        }
        | BindingSource::OperationUrl {
            service, operation, ..
        }
        | BindingSource::Context {
            service, operation, ..
        }
        | BindingSource::ContextFromBranch {
            service, operation, ..
        } => Some((service.as_ref(), operation.as_ref())),
        _ => None,
    };
    let alternatives = match source {
        BindingSource::Alternatives(alternatives) => alternatives.as_ref(),
        _ => &[],
    };
    single.into_iter().chain(
        alternatives
            .iter()
            .map(|alternative| (alternative.service.as_ref(), alternative.operation.as_ref())),
    )
}

fn base(source: &BindingSource) -> &BindingSource {
    match source {
        BindingSource::Transformed { source, .. } => source,
        _ => source,
    }
}

fn parent_index(
    plans: &[CallablePlan<'_>],
    service: &str,
    operation: &str,
) -> Result<usize, RuntimeCatalogError> {
    plans
        .iter()
        .position(|plan| plan.service_name() == service && plan.operation_name() == operation)
        .ok_or(RuntimeCatalogError::Capability)
}

fn target(
    parent: &CallablePlan<'_>,
    source: ContextSource,
    path: &[FieldId],
) -> Result<Target, RuntimeCatalogError> {
    let root = match source {
        ContextSource::SelectedItem => {
            tags::selected_item(parent.service(), parent.operation(), parent.items())?
        }
        ContextSource::Response => parent.operation().output()?,
        ContextSource::Request => parent.operation().input()?,
    };
    tags::resolve(parent.service(), root, path)
}

fn require_projection(
    parent: &CallablePlan<'_>,
    source: ContextSource,
    path: &[FieldId],
) -> Result<(), RuntimeCatalogError> {
    let projection = parent.context_projection();
    let paths = match source {
        ContextSource::SelectedItem => &projection.selected,
        ContextSource::Response => &projection.response,
        ContextSource::Request => &projection.request,
    };
    if paths.iter().any(|candidate| candidate.as_ref() == path) {
        Ok(())
    } else {
        Err(RuntimeCatalogError::Capability)
    }
}

fn validate_alternative_sets(child: &CallablePlan<'_>) -> Result<(), RuntimeCatalogError> {
    child
        .branches()
        .iter()
        .map(Some)
        .chain(std::iter::once(None).filter(|_| child.branches().is_empty()))
        .try_for_each(|branch| {
            let mut sets = child.effective_bindings(branch).filter_map(|binding| {
                match base(&binding.source) {
                    BindingSource::Alternatives(alternatives) => Some(alternatives.as_ref()),
                    _ => None,
                }
            });
            let Some(first) = sets.next() else {
                return Ok(());
            };
            if sets.all(|set| {
                set.len() == first.len()
                    && first.iter().all(|producer| {
                        set.iter().any(|other| {
                            other.service == producer.service
                                && other.operation == producer.operation
                        })
                    })
            }) {
                Ok(())
            } else {
                Err(RuntimeCatalogError::Capability)
            }
        })
}

fn validate_alternatives(
    plans: &[CallablePlan<'_>],
    child_index: usize,
    child: &CallablePlan<'_>,
    binding: &crate::catalog::runtime::BindingPlan,
    alternatives: &[crate::catalog::runtime::BindingAlternativePlan],
) -> Result<(), RuntimeCatalogError> {
    if alternatives.is_empty() || alternatives.len() > 8 {
        return Err(RuntimeCatalogError::Capability);
    }
    let destination = tags::resolve(child.service(), child.operation().input()?, &binding.path)?;
    let destination_kind = tags::target_kind(child.service(), destination)?;
    alternatives
        .iter()
        .enumerate()
        .try_for_each(|(index, alternative)| {
            if alternative.service.as_ref() != child.service_name()
                || alternatives
                    .iter()
                    .take(index)
                    .any(|prior| prior.operation == alternative.operation)
            {
                return Err(RuntimeCatalogError::Capability);
            }
            let parent_index = parent_index(plans, &alternative.service, &alternative.operation)?;
            if parent_index == child_index {
                return Err(RuntimeCatalogError::Capability);
            }
            let parent = plans
                .get(parent_index)
                .ok_or(RuntimeCatalogError::Capability)?;
            require_projection(parent, alternative.source, &alternative.path)?;
            let source = target(parent, alternative.source, &alternative.path)?;
            if compatible(
                tags::target_kind(parent.service(), source)?,
                destination_kind,
            ) {
                Ok(())
            } else {
                Err(RuntimeCatalogError::Capability)
            }
        })
}

fn compatible(left: &str, right: &str) -> bool {
    left == right || matches!((left, right), ("string", "enum") | ("enum", "string"))
}

fn identity_fields(
    identity: &IdentityPlan,
    check: &mut impl FnMut(&IdentityFieldPlan) -> Result<(), RuntimeCatalogError>,
) -> Result<(), RuntimeCatalogError> {
    match identity {
        IdentityPlan::CorrelatedKey(fields) => fields.iter().try_for_each(check),
        IdentityPlan::Choice(IdentityChoicePlan::PreferArn { fallback, .. }) => {
            leaf_fields(fallback, check)
        }
        IdentityPlan::Choice(IdentityChoicePlan::Discriminated(branches)) => branches
            .iter()
            .try_for_each(|branch| leaf_fields(&branch.identity, check)),
        _ => Ok(()),
    }
}

fn leaf_fields(
    identity: &IdentityLeafPlan,
    check: &mut impl FnMut(&IdentityFieldPlan) -> Result<(), RuntimeCatalogError>,
) -> Result<(), RuntimeCatalogError> {
    match identity {
        IdentityLeafPlan::Fields(fields) => fields.iter().try_for_each(check),
        _ => Ok(()),
    }
}
