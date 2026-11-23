use super::{CallablePlan, PlanPurpose, RuntimeCatalog, RuntimeCatalogError};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

impl RuntimeCatalog<'_> {
    /// Select services or `service:Operation` roots, retaining bootstrap plans and
    /// the connected parent/child binding closure. Schema and source identity stay unchanged.
    pub fn select_services(mut self, selectors: &[String]) -> Result<Self, RuntimeCatalogError> {
        if selectors.is_empty() {
            return Ok(self);
        }
        let names: Vec<_> = self.plans.iter().map(name).collect();
        let roots = roots(&names, selectors)?;
        let links = links(&self.plans)?;
        let selected = closure(&links, roots)?;
        self.plans = self.plans.into_vec().into_iter().enumerate()
            .filter(|(index, plan)| selected.contains(index) || bootstrap(plan))
            .map(|(_, plan)| plan).collect();
        Ok(self)
    }
}

type Name<'a> = (&'a str, &'a str);
fn name<'a>(plan: &'a CallablePlan<'_>) -> Name<'a> {
    (plan.service_name(), plan.operation_name())
}

fn bootstrap(plan: &CallablePlan<'_>) -> bool {
    matches!(plan.purpose(), PlanPurpose::AccountIdentity { .. } | PlanPurpose::RegionDiscovery { .. } | PlanPurpose::EndpointDiscovery)
}

fn roots(names: &[Name<'_>], selectors: &[String]) -> Result<BTreeSet<usize>, RuntimeCatalogError> {
    selectors.iter().try_fold(BTreeSet::new(), |mut selected, selector| {
        let (service, operation) = selector.split_once(':').map_or((selector.as_str(), None), |(service, operation)| (service, Some(operation)));
        let matches: Vec<_> = names.iter().enumerate().filter(|(_, name)| name.0 == service && operation.is_none_or(|operation| name.1 == operation)).map(|(index, _)| index).collect();
        if matches.is_empty() {
            return Err(RuntimeCatalogError::Selection(selector.clone()));
        }
        selected.extend(matches);
        Ok(selected)
    })
}

fn links(plans: &[CallablePlan<'_>]) -> Result<Vec<Vec<usize>>, RuntimeCatalogError> {
    let names: BTreeMap<_, _> = plans.iter().enumerate().map(|(index, plan)| (name(plan), index)).collect();
    let mut links = vec![Vec::new(); plans.len()];
    plans.iter().enumerate().try_for_each(|(child, plan)| {
        plan.bindings().iter().chain(plan.branches().iter().flat_map(|branch| branch.bindings.iter()))
            .flat_map(|binding| super::decode::producers(&binding.source))
            .chain(tag_parents(plan))
            .try_for_each(|parent| {
                let parent = *names.get(&parent).ok_or(RuntimeCatalogError::Capability)?;
                connect(&mut links, child, parent, plans.get(parent).is_some_and(bootstrap))
            })
    })?;
    Ok(links)
}

fn tag_parents<'a>(plan: &'a CallablePlan<'_>) -> impl Iterator<Item = Name<'a>> {
    let parents = match plan.purpose() {
        PlanPurpose::TagEndpoint(endpoint) => endpoint.parents.as_ref(),
        _ => &[],
    };
    parents.iter().map(|parent| (plan.service_name(), parent.operation.as_ref()))
}

fn connect(links: &mut [Vec<usize>], child: usize, parent: usize, bootstrap: bool) -> Result<(), RuntimeCatalogError> {
    links.get_mut(child).ok_or(RuntimeCatalogError::Capability)?.push(parent);
    if !bootstrap {
        links.get_mut(parent).ok_or(RuntimeCatalogError::Capability)?.push(child);
    }
    Ok(())
}

fn closure(links: &[Vec<usize>], roots: BTreeSet<usize>) -> Result<BTreeSet<usize>, RuntimeCatalogError> {
    let queue: VecDeque<_> = roots.iter().copied().collect();
    let (selected, _) = (0..links.len()).try_fold((roots, queue), |(mut selected, mut queue), _| {
        if let Some(index) = queue.pop_front() {
            links.get(index).ok_or(RuntimeCatalogError::Capability)?.iter().for_each(|next| {
                if selected.insert(*next) { queue.push_back(*next); }
            });
        }
        Ok::<_, RuntimeCatalogError>((selected, queue))
    })?;
    Ok(selected)
}

#[cfg(test)]
mod tests;
