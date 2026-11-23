use super::{CallablePlan, PlanPurpose, RuntimeCatalog, RuntimeCatalogError};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

impl RuntimeCatalog<'_> {
    /// Select services or `service:Operation` roots, retaining bootstrap plans and
    /// their required ancestors. Descendants run only when selected explicitly.
    pub fn select_services(mut self, selectors: &[String]) -> Result<Self, RuntimeCatalogError> {
        if selectors.is_empty() {
            return Ok(self);
        }
        let names: Vec<_> = self.plans.iter().map(name).collect();
        let roots = roots(&names, selectors)?;
        let links = links(&self.plans)?;
        let selected = closure(&links, roots)?;
        self.plans = self
            .plans
            .into_vec()
            .into_iter()
            .enumerate()
            .filter(|(index, plan)| selected.contains(index) || bootstrap(plan))
            .map(|(_, plan)| plan)
            .collect();
        Ok(self)
    }
}

type Name<'a> = (&'a str, &'a str);
fn name<'a>(plan: &'a CallablePlan<'_>) -> Name<'a> {
    (plan.service_name(), plan.operation_name())
}

fn bootstrap(plan: &CallablePlan<'_>) -> bool {
    matches!(
        plan.purpose(),
        PlanPurpose::AccountIdentity { .. }
            | PlanPurpose::RegionDiscovery { .. }
            | PlanPurpose::EndpointDiscovery
    )
}

fn roots(names: &[Name<'_>], selectors: &[String]) -> Result<BTreeSet<usize>, RuntimeCatalogError> {
    let mut selected = BTreeSet::new();
    for selector in selectors {
        select_root(names, selector, &mut selected)?;
    }
    Ok(selected)
}

fn select_root(
    names: &[Name<'_>],
    selector: &str,
    selected: &mut BTreeSet<usize>,
) -> Result<(), RuntimeCatalogError> {
    let (service, operation) = selector
        .split_once(':')
        .map_or((selector, None), |(service, operation)| {
            (service, Some(operation))
        });
    let mut found = false;
    for (index, name) in names.iter().enumerate() {
        if name.0 == service && operation.is_none_or(|operation| name.1 == operation) {
            selected.insert(index);
            found = true;
        }
    }
    if !found {
        return Err(RuntimeCatalogError::Selection(selector.to_owned()));
    }
    Ok(())
}

fn links(plans: &[CallablePlan<'_>]) -> Result<Vec<Vec<usize>>, RuntimeCatalogError> {
    let names: BTreeMap<_, _> = plans
        .iter()
        .enumerate()
        .map(|(index, plan)| (name(plan), index))
        .collect();
    let mut links = vec![Vec::new(); plans.len()];
    for (child, plan) in plans.iter().enumerate() {
        for parent in parents(plan) {
            let parent = *names.get(&parent).ok_or(RuntimeCatalogError::Capability)?;
            connect(&mut links, child, parent)?;
        }
    }
    Ok(links)
}

fn parents<'a>(plan: &'a CallablePlan<'_>) -> impl Iterator<Item = Name<'a>> {
    plan.bindings()
        .iter()
        .chain(
            plan.branches()
                .iter()
                .flat_map(|branch| branch.bindings.iter()),
        )
        .flat_map(|binding| super::decode::producers(&binding.source))
        .chain(tag_parents(plan))
}

fn tag_parents<'a>(plan: &'a CallablePlan<'_>) -> impl Iterator<Item = Name<'a>> {
    let parents = match plan.purpose() {
        PlanPurpose::TagEndpoint(endpoint) => endpoint.parents.as_ref(),
        _ => &[],
    };
    parents
        .iter()
        .map(|parent| (plan.service_name(), parent.operation.as_ref()))
}

fn connect(
    links: &mut [Vec<usize>],
    child: usize,
    parent: usize,
) -> Result<(), RuntimeCatalogError> {
    links
        .get_mut(child)
        .ok_or(RuntimeCatalogError::Capability)?
        .push(parent);
    Ok(())
}

fn closure(
    links: &[Vec<usize>],
    roots: BTreeSet<usize>,
) -> Result<BTreeSet<usize>, RuntimeCatalogError> {
    let mut queue: VecDeque<_> = roots.iter().copied().collect();
    let mut selected = roots;
    while let Some(index) = queue.pop_front() {
        for next in links.get(index).ok_or(RuntimeCatalogError::Capability)? {
            if selected.insert(*next) {
                queue.push_back(*next);
            }
        }
    }
    Ok(selected)
}

#[cfg(test)]
mod tests;
