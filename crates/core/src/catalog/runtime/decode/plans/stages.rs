use super::{BindingSource, CallablePlan, RuntimeCatalogError};
use crate::catalog::runtime::ExecutionBranch;
use std::collections::{BTreeMap, BTreeSet};

type StageKey<'a> = (&'a str, &'a str, Option<&'a str>);
struct Node<'p, 'catalog> {
    plan: &'p CallablePlan<'catalog>,
    branch: Option<&'p ExecutionBranch>,
    parents: Vec<usize>,
    depth: Option<u16>,
}

pub(super) fn depths(plans: &[CallablePlan<'_>]) -> Result<Vec<u16>, RuntimeCatalogError> {
    let mut nodes = Vec::new();
    let mut keys = BTreeMap::new();
    for plan in plans {
        for branch in plan
            .branches()
            .iter()
            .map(Some)
            .chain(std::iter::once(None).filter(|_| plan.branches().is_empty()))
        {
            let key = (
                plan.service_name(),
                plan.operation_name(),
                branch.map(|stage| stage.discriminator.as_str()),
            );
            if keys.insert(key, nodes.len()).is_some() {
                return Err(RuntimeCatalogError::Capability);
            }
            nodes.push(Node {
                plan,
                branch,
                parents: Vec::new(),
                depth: None,
            });
        }
    }
    for node in &mut nodes {
        let mut parents = BTreeSet::new();
        for binding in node.plan.effective_bindings(node.branch) {
            add_parents(&keys, &binding.source, &mut parents)?;
        }
        node.parents = parents.into_iter().collect();
    }
    for _ in 0..nodes.len() {
        let mut changed = false;
        for index in 0..nodes.len() {
            let node = nodes.get(index).ok_or(RuntimeCatalogError::Bounds)?;
            if node.depth.is_some() {
                continue;
            }
            let mut depth = Some(0_u16);
            for parent in &node.parents {
                match nodes.get(*parent).ok_or(RuntimeCatalogError::Bounds)?.depth {
                    Some(parent) => {
                        depth = depth.map(|depth| depth.max(parent.saturating_add(1)));
                        if parent == u16::MAX {
                            return Err(RuntimeCatalogError::Bounds);
                        }
                    }
                    None => {
                        depth = None;
                        break;
                    }
                }
            }
            if let Some(depth) = depth {
                nodes
                    .get_mut(index)
                    .ok_or(RuntimeCatalogError::Bounds)?
                    .depth = Some(depth);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    let mut result = Vec::new();
    result
        .try_reserve(plans.len())
        .map_err(|_| RuntimeCatalogError::Bounds)?;
    for plan in plans {
        let mut maximum = 0;
        for branch in plan
            .branches()
            .iter()
            .map(Some)
            .chain(std::iter::once(None).filter(|_| plan.branches().is_empty()))
        {
            let key = (
                plan.service_name(),
                plan.operation_name(),
                branch.map(|stage| stage.discriminator.as_str()),
            );
            let index = keys.get(&key).ok_or(RuntimeCatalogError::Capability)?;
            let depth = nodes
                .get(*index)
                .and_then(|node| node.depth)
                .ok_or(RuntimeCatalogError::Capability)?;
            maximum = maximum.max(depth);
        }
        result.push(maximum);
    }
    Ok(result)
}

fn add_parents(
    keys: &BTreeMap<StageKey<'_>, usize>,
    source: &BindingSource,
    parents: &mut BTreeSet<usize>,
) -> Result<(), RuntimeCatalogError> {
    let source = match source {
        BindingSource::Transformed { source, .. } => source.as_ref(),
        _ => source,
    };
    match source {
        BindingSource::ContextFromBranch {
            service,
            operation,
            branch,
            ..
        } => {
            let index = keys
                .get(&(service.as_ref(), operation.as_ref(), Some(branch.as_ref())))
                .ok_or(RuntimeCatalogError::Capability)?;
            parents.insert(*index);
        }
        _ => {
            for (service, operation) in super::context::producers(source) {
                let mut found = false;
                for ((candidate_service, candidate_operation, _), index) in
                    keys.range((service, operation, None)..)
                {
                    if *candidate_service != service || *candidate_operation != operation {
                        break;
                    }
                    found = true;
                    parents.insert(*index);
                }
                if !found {
                    return Err(RuntimeCatalogError::Capability);
                }
            }
        }
    }
    Ok(())
}
