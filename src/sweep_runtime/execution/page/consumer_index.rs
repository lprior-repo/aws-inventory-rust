#[cfg(test)]
mod tests;

use inventory_core::catalog::runtime::{
    BindingPlan, BindingSource, CallablePlan, ExecutionBranch, PlanPurpose, RuntimeCatalog,
};
use std::{collections::TryReserveError, slice};

// Runtime catalog schema limits: 65,536 plans, 4,096 bindings per base/branch,
// 16 branches, and 64 tag parents. Wildcards avoid the producer × consumer product.
const MAX_PLANS: usize = 65_536;
const MAX_ENTRIES: usize = MAX_PLANS * 4096 * 17 * 8;
// The compiled schema currently permits one transform. Bound traversal explicitly
// rather than making recursive source inspection part of index construction.
const MAX_SOURCE_DEPTH: usize = 16;

#[derive(Debug, thiserror::Error)]
pub(in crate::sweep_runtime) enum ConsumerIndexError {
    #[error("consumer fanout index exceeds the catalog schema bound")]
    Bounds,
    #[error("consumer binding source exceeds the transform depth bound")]
    SourceDepth,
    #[error("consumer fanout index allocation failed")]
    Allocation(#[from] TryReserveError),
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Producer<'a> {
    service: &'a str,
    operation: &'a str,
    branch: Option<&'a str>,
}

enum Dependency<'a> {
    Producer(Producer<'a>),
    Alternatives(&'a [inventory_core::catalog::runtime::BindingAlternativePlan]),
    AnyProducer,
    Independent,
}

#[derive(Clone, Copy)]
struct Consumer<'a, 'catalog> {
    ordinal: usize,
    plan: &'a CallablePlan<'catalog>,
}

struct Entry<'a, 'catalog> {
    producer: Producer<'a>,
    consumer: Consumer<'a, 'catalog>,
}

/// Immutable conservative fanout, built once per execution. Candidates retain
/// catalog order and borrowed branch metadata; row/branch eligibility remains
/// exclusively the responsibility of the existing production consumers.
pub(in crate::sweep_runtime) struct ConsumerIndex<'a, 'catalog> {
    entries: Vec<Entry<'a, 'catalog>>,
    wildcards: Vec<Consumer<'a, 'catalog>>,
}

impl<'a, 'catalog> ConsumerIndex<'a, 'catalog> {
    pub(in crate::sweep_runtime) fn build(
        catalog: &'a RuntimeCatalog<'catalog>,
    ) -> Result<Self, ConsumerIndexError> {
        Self::from_plans(catalog.plans())
    }

    fn from_plans(plans: &'a [CallablePlan<'catalog>]) -> Result<Self, ConsumerIndexError> {
        if plans.len() > MAX_PLANS {
            return Err(ConsumerIndexError::Bounds);
        }
        let mut index = Self { entries: Vec::new(), wildcards: Vec::new() };
        plans.iter().enumerate().try_for_each(|(ordinal, plan)| {
            index.add_plan(Consumer { ordinal, plan })
        })?;
        Ok(index.freeze())
    }

    fn freeze(mut self) -> Self {
        self.entries.sort_unstable_by(|left, right| {
            left.producer.cmp(&right.producer)
                .then(left.consumer.ordinal.cmp(&right.consumer.ordinal))
        });
        self.entries.dedup_by(|left, right| {
            left.producer == right.producer && left.consumer.ordinal == right.consumer.ordinal
        });
        self.wildcards.sort_unstable_by_key(|consumer| consumer.ordinal);
        self.wildcards.dedup_by_key(|consumer| consumer.ordinal);
        self
    }

    #[cfg(test)]
    fn candidates(
        &self, service: &str, operation: &str,
    ) -> impl Iterator<Item = &'a CallablePlan<'catalog>> + '_ {
        self.candidates_for_branch(service, operation, None)
    }

    /// Lookup allocates nothing and merges ordinary producers with the exact
    /// stable source-stage key. Ordinary dependencies also accept inherited stages.
    pub(in crate::sweep_runtime) fn candidates_for_branch(
        &self, service: &str, operation: &str, branch: Option<&str>,
    ) -> impl Iterator<Item = &'a CallablePlan<'catalog>> + '_ {
        let exact = self.entries_for(service, operation, None);
        let staged = branch.map_or(&[][..], |branch| self.entries_for(service, operation, Some(branch)));
        Candidates {
            exact: exact.iter(), staged: staged.iter(), wildcards: self.wildcards.iter(),
        }
    }

    fn entries_for(&self, service: &str, operation: &str, branch: Option<&str>) -> &[Entry<'a, 'catalog>] {
        let producer = Producer { service, operation, branch };
        let start = self.entries.partition_point(|entry| entry.producer < producer);
        let entries = self.entries.get(start..).map_or(&[][..], |entries| entries);
        let end = entries.partition_point(|entry| entry.producer == producer);
        entries.get(..end).map_or(&[][..], |entries| entries)
    }

    fn add_plan(&mut self, consumer: Consumer<'a, 'catalog>) -> Result<(), ConsumerIndexError> {
        match consumer.plan.purpose() {
            PlanPurpose::Enumeration => self.add_enumeration(consumer),
            PlanPurpose::TagEndpoint(endpoint) => endpoint.parents.iter().try_for_each(|parent| {
                let producer = Producer {
                    service: consumer.plan.service_name(), operation: &parent.operation, branch: None,
                };
                bounded_push(&mut self.entries, Entry { producer, consumer }, MAX_ENTRIES)
            }),
            _ => Ok(()),
        }
    }

    fn add_enumeration(&mut self, consumer: Consumer<'a, 'catalog>) -> Result<(), ConsumerIndexError> {
        let plan = consumer.plan;
        let wildcard = bindings(plan.bindings(), plan.branches()).try_fold(false, |found, binding| {
            Ok::<_, ConsumerIndexError>(matches!(dependency(&binding.source)?, Dependency::AnyProducer) || found)
        })?;
        if wildcard {
            return bounded_push(&mut self.wildcards, consumer, MAX_PLANS);
        }
        bindings(plan.bindings(), plan.branches()).try_for_each(|binding| {
            self.add_dependency(consumer, dependency(&binding.source)?)
        })
    }

    fn add_dependency(
        &mut self, consumer: Consumer<'a, 'catalog>, dependency: Dependency<'a>,
    ) -> Result<(), ConsumerIndexError> {
        match dependency {
            Dependency::Producer(producer) => {
                bounded_push(&mut self.entries, Entry { producer, consumer }, MAX_ENTRIES)
            }
            Dependency::Alternatives(alternatives) => alternatives.iter().try_for_each(|alternative| {
                let producer = Producer { service: &alternative.service, operation: &alternative.operation, branch: None };
                bounded_push(&mut self.entries, Entry { producer, consumer }, MAX_ENTRIES)
            }),
            Dependency::AnyProducer => bounded_push(&mut self.wildcards, consumer, MAX_PLANS),
            Dependency::Independent => Ok(()),
        }
    }
}

fn bindings<'a>(
    base: &'a [BindingPlan], branches: &'a [ExecutionBranch],
) -> impl Iterator<Item = &'a BindingPlan> {
    base.iter().chain(branches.iter().flat_map(|branch| branch.bindings.iter()))
}

fn dependency(source: &BindingSource) -> Result<Dependency<'_>, ConsumerIndexError> {
    let leaf = std::iter::successors(Some(source), |source| match source {
        BindingSource::Transformed { source, .. } => Some(source.as_ref()),
        _ => None,
    }).take(MAX_SOURCE_DEPTH).find(|source| !matches!(source, BindingSource::Transformed { .. }))
        .ok_or(ConsumerIndexError::SourceDepth)?;
    match leaf {
        BindingSource::Operation { service, operation, .. }
        | BindingSource::OperationArn { service, operation, .. }
        | BindingSource::OperationUrl { service, operation, .. }
        | BindingSource::Context { service, operation, .. } => {
            Ok(Dependency::Producer(Producer { service, operation, branch: None }))
        }
        BindingSource::ContextFromBranch { service, operation, branch, .. } => {
            Ok(Dependency::Producer(Producer { service, operation, branch: Some(branch) }))
        }
        BindingSource::ParentIdentity { .. } => Ok(Dependency::AnyProducer),
        BindingSource::Alternatives(alternatives) => {
            if alternatives.is_empty() || alternatives.len() > 8 { return Err(ConsumerIndexError::Bounds); }
            Ok(Dependency::Alternatives(alternatives))
        }
        BindingSource::Account | BindingSource::Region | BindingSource::RegionList | BindingSource::Fixed(_) => Ok(Dependency::Independent),
        BindingSource::Transformed { .. } => Err(ConsumerIndexError::SourceDepth),
    }
}

fn bounded_push<T>(values: &mut Vec<T>, value: T, maximum: usize) -> Result<(), ConsumerIndexError> {
    if values.len() >= maximum {
        return Err(ConsumerIndexError::Bounds);
    }
    values.try_reserve(1)?;
    values.push(value);
    Ok(())
}

struct Candidates<'index, 'a, 'catalog> {
    exact: slice::Iter<'index, Entry<'a, 'catalog>>,
    staged: slice::Iter<'index, Entry<'a, 'catalog>>,
    wildcards: slice::Iter<'index, Consumer<'a, 'catalog>>,
}

impl<'a, 'catalog> Iterator for Candidates<'_, 'a, 'catalog> {
    type Item = &'a CallablePlan<'catalog>;

    fn next(&mut self) -> Option<Self::Item> {
        let consumer = [
            self.exact.as_slice().first().map(|entry| entry.consumer),
            self.staged.as_slice().first().map(|entry| entry.consumer),
            self.wildcards.as_slice().first().copied(),
        ].into_iter().flatten().min_by_key(|consumer| consumer.ordinal)?;
        if self.exact.as_slice().first().is_some_and(|entry| entry.consumer.ordinal == consumer.ordinal) {
            self.exact.next();
        }
        if self.staged.as_slice().first().is_some_and(|entry| entry.consumer.ordinal == consumer.ordinal) {
            self.staged.next();
        }
        if self.wildcards.as_slice().first().is_some_and(|entry| entry.ordinal == consumer.ordinal) {
            self.wildcards.next();
        }
        Some(consumer.plan)
    }
}
