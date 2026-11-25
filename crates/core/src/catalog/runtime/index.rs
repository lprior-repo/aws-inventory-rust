use super::{CallablePlan, KindNamePath, RelationshipPlan, RuntimeCatalogError};
use crate::inventory::Kind;

#[derive(Debug)]
pub(super) struct Index {
    plans: Box<[usize]>,
    name_paths: Box<[usize]>,
    relations: Box<[usize]>,
}

impl Index {
    pub(super) fn new(
        plans: &[CallablePlan<'_>],
        name_paths: &[KindNamePath],
        relations: &[RelationshipPlan],
    ) -> Result<Self, RuntimeCatalogError> {
        Ok(Self {
            plans: sorted(plans, plan_key)?,
            name_paths: sorted(name_paths, name_path_kind)?,
            relations: sorted(relations, source_kind)?,
        })
    }

    /// Plan positions are only valid for the slice they were built from, so a
    /// catalog that replaces its plans must replace them here too.
    pub(super) fn with_plans(
        mut self,
        plans: &[CallablePlan<'_>],
    ) -> Result<Self, RuntimeCatalogError> {
        self.plans = sorted(plans, plan_key)?;
        Ok(self)
    }

    pub(super) fn lookup<'p, 'a>(
        &self,
        plans: &'p [CallablePlan<'a>],
        service: &str,
        operation: &str,
    ) -> Option<&'p CallablePlan<'a>> {
        plans.get(self.lookup_index(plans, service, operation)?)
    }

    pub(super) fn lookup_index(
        &self,
        plans: &[CallablePlan<'_>],
        service: &str,
        operation: &str,
    ) -> Option<usize> {
        let target = (service, operation);
        let start = first_at_least(&self.plans, plans, plan_key, &target);
        let position = *self.plans.get(start)?;
        let plan = plans.get(position)?;
        (plan_key(plan) == target).then_some(position)
    }

    pub(super) fn name_path<'p>(
        &self,
        name_paths: &'p [KindNamePath],
        kind: &Kind,
    ) -> Option<&'p KindNamePath> {
        let start = first_at_least(&self.name_paths, name_paths, name_path_kind, &kind);
        let entry = name_paths.get(*self.name_paths.get(start)?)?;
        (&entry.kind == kind).then_some(entry)
    }

    pub(super) fn relationships_from<'p>(
        &'p self,
        relations: &'p [RelationshipPlan],
        kind: &Kind,
    ) -> impl Iterator<Item = &'p RelationshipPlan> + 'p {
        let start = first_at_least(&self.relations, relations, source_kind, &kind);
        let end = first_above(&self.relations, relations, source_kind, &kind);
        self.relations
            .get(start..end)
            .into_iter()
            .flatten()
            .filter_map(move |position| relations.get(*position))
    }
}

fn plan_key<'p>(plan: &'p CallablePlan<'_>) -> (&'p str, &'p str) {
    (plan.service_name(), plan.operation_name())
}

fn name_path_kind(entry: &KindNamePath) -> &Kind {
    &entry.kind
}

fn source_kind(relation: &RelationshipPlan) -> &Kind {
    &relation.source_kind
}

fn sorted<'t, T, K: Ord>(
    items: &'t [T],
    key: impl Fn(&'t T) -> K,
) -> Result<Box<[usize]>, RuntimeCatalogError> {
    let mut positions = Vec::new();
    positions
        .try_reserve_exact(items.len())
        .map_err(|_| RuntimeCatalogError::Bounds)?;
    positions.extend(0..items.len());
    positions.sort_by(|left, right| {
        let left_key = items.get(*left).map(&key);
        let right_key = items.get(*right).map(&key);
        left_key.cmp(&right_key).then_with(|| left.cmp(right))
    });
    Ok(positions.into_boxed_slice())
}

fn first_at_least<'t, T, K: Ord>(
    positions: &[usize],
    items: &'t [T],
    key: impl Fn(&'t T) -> K,
    target: &K,
) -> usize {
    positions.partition_point(|position| items.get(*position).map(&key).as_ref() < Some(target))
}

fn first_above<'t, T, K: Ord>(
    positions: &[usize],
    items: &'t [T],
    key: impl Fn(&'t T) -> K,
    target: &K,
) -> usize {
    positions.partition_point(|position| items.get(*position).map(&key).as_ref() <= Some(target))
}
