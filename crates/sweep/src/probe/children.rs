use super::*;
use crate::execution::page::{
    child::{self, BindingContext, BindingRow, FieldResolution},
    consumer_index::{ConsumerIndex, ConsumerIndexError},
};
use futures_util::{StreamExt, stream};
use inventory_core::inventory::MAX_DISCOVERY_FRAMES;
use std::{collections::BTreeMap, num::NonZeroUsize};

type Plan<'a, 'catalog> = &'a CallablePlan<'catalog>;
type Parent<'a, 'catalog> = (Plan<'a, 'catalog>, ProbeAnswer);
type ChildResult<'a, 'catalog> = (Plan<'a, 'catalog>, ChildProbeOutcome);

#[derive(Clone, Copy, Debug)]
pub struct ChildProbeSamples(NonZeroUsize);

impl ChildProbeSamples {
    pub const MAX: usize = 64;

    #[must_use]
    pub fn new(count: usize) -> Option<Self> {
        NonZeroUsize::new(count)
            .filter(|count| count.get() <= Self::MAX)
            .map(Self)
    }
}

impl Default for ChildProbeSamples {
    fn default() -> Self {
        Self(NonZeroUsize::MIN)
    }
}

#[non_exhaustive]
pub struct ProbeChildren<'a, 'catalog> {
    plans: &'a [Plan<'a, 'catalog>],
    parents: Vec<Parent<'a, 'catalog>>,
    samples: ChildProbeSamples,
}

impl<'a, 'catalog> ProbeChildren<'a, 'catalog> {
    #[must_use]
    pub fn new(
        plans: &'a [Plan<'a, 'catalog>],
        parents: Vec<Parent<'a, 'catalog>>,
        samples: ChildProbeSamples,
    ) -> Self {
        Self {
            plans,
            parents,
            samples,
        }
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum ChildProbeOutcome {
    NoParentItems,
    Unbound(Option<String>),
    TagEndpointBinding,
    Called { items: u64, errors: Vec<SweepError> },
}

pub async fn probe_children<'a, 'catalog>(
    catalog: &'a RuntimeCatalog<'catalog>,
    credentials: &impl CredentialSource,
    client: &Client,
    executor: &ApiExecutor,
    scope: ProbeScope<'_>,
    children: ProbeChildren<'a, 'catalog>,
) -> Result<Vec<ChildResult<'a, 'catalog>>, SweepError> {
    let ProbeChildren {
        plans,
        mut parents,
        samples,
    } = children;
    if plans.iter().any(|plan| !belongs(catalog, plan))
        || parents
            .iter()
            .any(|(plan, answer)| !belongs(catalog, plan) || key(plan) != answer.page.key())
    {
        return Err(SweepError::InvalidConfig(
            "probe child plans and parents must come from this catalog",
        ));
    }
    parents.sort_unstable_by_key(|(plan, _)| key(plan));
    let mut wave = Wave::new(catalog, plans, scope, samples)?;
    let send = |task: Bound<'a, 'catalog>| async move {
        let plan = task.plan;
        let probe = Probe {
            catalog,
            credentials,
            client,
            executor,
            plan,
            scope,
        };
        (plan, task.send(probe).await)
    };
    for _ in 0..=MAX_DISCOVERY_FRAMES {
        let tasks = wave.prepare(&parents)?;
        if tasks.is_empty() {
            break;
        }
        parents = stream::iter(tasks.into_iter().map(send))
            .buffered(8)
            .fold(Ok(Vec::new()), |next, result| {
                std::future::ready(next.and_then(|next| wave.record(next, result)))
            })
            .await?;
    }
    Ok(wave.finish())
}

struct Wave<'a, 'catalog, 'scope> {
    index: ConsumerIndex<'a, 'catalog>,
    states: BTreeMap<(&'a str, &'a str), (Plan<'a, 'catalog>, State)>,
    scope: ProbeScope<'scope>,
    limit: usize,
}

#[derive(Default)]
struct State {
    requests: usize,
    parent_items: bool,
    binding_error: Option<String>,
    items: u64,
    errors: Vec<SweepError>,
}

impl<'a, 'catalog> Wave<'a, 'catalog, '_> {
    fn new<'scope>(
        catalog: &'a RuntimeCatalog<'catalog>,
        plans: &[Plan<'a, 'catalog>],
        scope: ProbeScope<'scope>,
        samples: ChildProbeSamples,
    ) -> Result<Wave<'a, 'catalog, 'scope>, SweepError> {
        Ok(Wave {
            index: ConsumerIndex::build(catalog).map_err(|error| match error {
                ConsumerIndexError::Allocation(error) => SweepError::Allocation(error),
                ConsumerIndexError::Bounds | ConsumerIndexError::SourceDepth => {
                    SweepError::Operation("reviewed consumer index construction failed")
                }
            })?,
            states: plans
                .iter()
                .copied()
                .filter(|plan| ProbeAnswer::requires_parent(plan))
                .map(|plan| (key(plan), (plan, State::default())))
                .collect(),
            scope,
            limit: samples.0.get(),
        })
    }

    fn prepare(
        &mut self,
        parents: &[Parent<'a, 'catalog>],
    ) -> Result<Vec<Bound<'a, 'catalog>>, SweepError> {
        parents
            .iter()
            .filter(|(_, answer)| answer.items != 0)
            .try_fold(Vec::new(), |mut tasks, (parent, answer)| {
                self.prepare_parent(parent, &answer.page, &mut tasks)?;
                Ok(tasks)
            })
    }

    fn prepare_parent(
        &mut self,
        parent: Plan<'a, 'catalog>,
        page: &ProbePage,
        tasks: &mut Vec<Bound<'a, 'catalog>>,
    ) -> Result<(), SweepError> {
        let branch = branch_at(parent, page.branch);
        let branch_name = branch.map(|branch| branch.discriminator.as_str());
        let mut samples = None;
        self.index
            .candidates_for_branch(parent.service_name(), parent.operation_name(), branch_name)
            .try_for_each(|plan| {
                let Some((_, state)) = self.states.get_mut(&key(plan)) else {
                    return Ok(());
                };
                if state.requests == self.limit || !consumes(plan, parent, branch_name) {
                    return Ok(());
                }
                state.parent_items = true;
                let samples =
                    samples.get_or_insert_with(|| page.samples(parent, branch, self.limit));
                let context = BindingContext {
                    parent_plan: parent,
                    account: self.scope.account,
                };
                state.admit(plan, samples, context, self.scope.region, self.limit, tasks)
            })
    }

    fn record(
        &mut self,
        mut next: Vec<Parent<'a, 'catalog>>,
        (plan, result): (Plan<'a, 'catalog>, Result<ProbeAnswer, SweepError>),
    ) -> Result<Vec<Parent<'a, 'catalog>>, SweepError> {
        let (_, state) = self
            .states
            .get_mut(&key(plan))
            .ok_or(SweepError::Operation("missing probe child state"))?;
        match result {
            Ok(answer) if answer.items != 0 => {
                state.items = state
                    .items
                    .checked_add(answer.items)
                    .ok_or(SweepError::Operation("probe item count overflow"))?;
                next.try_reserve(1)?;
                next.push((plan, answer));
            }
            Ok(_) => {}
            Err(error) => {
                state.errors.try_reserve(1)?;
                state.errors.push(error);
            }
        }
        Ok(next)
    }

    fn finish(self) -> Vec<ChildResult<'a, 'catalog>> {
        self.states
            .into_values()
            .map(|(plan, state)| (plan, state.finish(plan)))
            .collect()
    }
}

impl State {
    fn admit<'a, 'catalog>(
        &mut self,
        plan: Plan<'a, 'catalog>,
        samples: &Result<Vec<Result<Sample<'_>, SweepError>>, SweepError>,
        context: BindingContext<'_, '_>,
        home: &RegionId,
        limit: usize,
        tasks: &mut Vec<Bound<'a, 'catalog>>,
    ) -> Result<(), SweepError> {
        let samples = match samples {
            Ok(samples) => samples,
            Err(error) => {
                self.binding_error.get_or_insert_with(|| error.to_string());
                return Ok(());
            }
        };
        let remaining = limit.saturating_sub(self.requests);
        let mut tasks_for_plan = samples
            .iter()
            .filter_map(|sample| {
                let sample = match sample {
                    Ok(sample) => sample,
                    Err(error) => {
                        self.binding_error.get_or_insert_with(|| error.to_string());
                        return None;
                    }
                };
                sample
                    .bind(plan, context, home)
                    .inspect_err(|error| {
                        self.binding_error.get_or_insert_with(|| error.to_string());
                    })
                    .ok()
                    .flatten()
            })
            .take(remaining);
        tasks_for_plan.try_for_each(|task| {
            tasks.try_reserve(1)?;
            tasks.push(task);
            self.requests = self.requests.saturating_add(1);
            Ok(())
        })
    }

    fn finish(self, plan: &CallablePlan<'_>) -> ChildProbeOutcome {
        if self.requests != 0 {
            return ChildProbeOutcome::Called {
                items: self.items,
                errors: self.errors,
            };
        }
        if self.parent_items {
            return ChildProbeOutcome::Unbound(self.binding_error);
        }
        if matches!(plan.purpose(), PlanPurpose::TagEndpoint(_))
            && !branches(plan).any(|(_, branch)| input::requires_parent_branch(plan, branch))
        {
            return ChildProbeOutcome::TagEndpointBinding;
        }
        ChildProbeOutcome::NoParentItems
    }
}

struct Sample<'a> {
    row: Record,
    region: &'a Region,
    branch: Option<&'a str>,
    ancestry: DiscoveryContext,
}

impl ProbePage {
    fn key(&self) -> (&str, &str) {
        (self.service.as_str(), self.operation.as_str())
    }

    fn samples<'a>(
        &'a self,
        plan: &CallablePlan<'_>,
        branch: Option<&'a ExecutionBranch>,
        limit: usize,
    ) -> Result<Vec<Result<Sample<'a>, SweepError>>, SweepError> {
        let (count, selected) = record::item_selection(&self.response, plan.items())?;
        let mut samples = Vec::new();
        samples.try_reserve_exact(count.min(limit))?;
        let branch_name = branch.map(|branch| branch.discriminator.as_str());
        samples.extend(
            selected
                .take(limit)
                .enumerate()
                .filter_map(|(ordinal, row)| {
                    row.and_then(|row| {
                        let Some(row) = crate::resource_projection::select_item(
                            plan,
                            branch,
                            row,
                            &self.request,
                            &self.response,
                        )?
                        else {
                            return Ok(None);
                        };
                        let frame = child::project_frame(
                            plan,
                            ordinal,
                            &row,
                            &self.request,
                            &self.response,
                            (&self.service, &self.operation, branch_name),
                        )?;
                        let ancestry = self.ancestry.push(frame).map_err(|_| {
                            SweepError::Operation("discovery ancestry exceeds bounds")
                        })?;
                        Ok(Some(Sample {
                            row,
                            region: &self.region,
                            branch: branch_name,
                            ancestry,
                        }))
                    })
                    .transpose()
                }),
        );
        Ok(samples)
    }
}

impl Sample<'_> {
    fn bind<'a, 'catalog>(
        &self,
        plan: Plan<'a, 'catalog>,
        context: BindingContext<'_, '_>,
        home: &RegionId,
    ) -> Result<Option<Bound<'a, 'catalog>>, SweepError> {
        let row = BindingRow {
            row: &self.row,
            region: self.region,
            branch: self.branch,
            ancestry: &self.ancestry,
        };
        let result = branches(plan).find_map(|(ordinal, branch)| {
            match self.bind_branch(plan, &context, &row, home, (ordinal, branch)) {
                Ok(None) => None,
                Ok(Some(bound)) => Some(Ok(bound)),
                Err(error) => Some(Err(error)),
            }
        });
        result.transpose()
    }

    fn bind_branch<'a, 'catalog>(
        &self,
        plan: Plan<'a, 'catalog>,
        context: &BindingContext<'_, '_>,
        row: &BindingRow<'_>,
        home: &RegionId,
        (ordinal, branch): (Option<usize>, Option<&ExecutionBranch>),
    ) -> Result<Option<Bound<'a, 'catalog>>, SweepError> {
        if !child::eligible(context.parent_plan, plan, row, branch)? {
            return Ok(None);
        }
        let region = child::child_region(
            branch.map_or(plan.scope(), |branch| branch.scope),
            self.region,
            home,
        );
        let request_region = routing::request_region(
            plan,
            &region,
            home,
            branch.map(|branch| branch.discriminator.as_str()),
        )?;
        let FieldResolution::Ready(fields) =
            child::fields(context, plan, row, branch, &request_region)?
        else {
            return Ok(None);
        };
        let base = input::merge_input(&child::empty_record()?, &fields)?;
        Ok(Some(Bound {
            plan,
            branch: ordinal,
            region,
            request_region,
            base,
            ancestry: self.ancestry.clone(),
        }))
    }
}

struct Bound<'a, 'catalog> {
    plan: Plan<'a, 'catalog>,
    branch: Option<usize>,
    region: Region,
    request_region: RegionId,
    base: Record,
    ancestry: DiscoveryContext,
}

impl Bound<'_, '_> {
    async fn send(
        self,
        probe: Probe<'_, '_, impl CredentialSource>,
    ) -> Result<ProbeAnswer, SweepError> {
        let endpoint = routing::resolve(self.plan, &self.request_region)?;
        probe
            .first_page(
                &endpoint,
                self.base,
                self.region,
                self.branch,
                self.ancestry,
            )
            .await
    }
}

fn consumes(
    plan: &CallablePlan<'_>,
    parent: &CallablePlan<'_>,
    source_branch: Option<&str>,
) -> bool {
    branches(plan).any(|(_, branch)| {
        plan.effective_bindings(branch)
            .any(|binding| child::consumes_current(&binding.source, parent, source_branch))
    })
}

fn branches<'a>(
    plan: &'a CallablePlan<'_>,
) -> impl Iterator<Item = (Option<usize>, Option<&'a ExecutionBranch>)> {
    plan.branches()
        .iter()
        .enumerate()
        .map(|(ordinal, branch)| (Some(ordinal), Some(branch)))
        .chain(plan.branches().is_empty().then_some((None, None)))
}

fn key<'a>(plan: &'a CallablePlan<'_>) -> (&'a str, &'a str) {
    (plan.service_name(), plan.operation_name())
}

fn belongs(catalog: &RuntimeCatalog<'_>, plan: &CallablePlan<'_>) -> bool {
    catalog
        .lookup(plan.service_name(), plan.operation_name())
        .is_some_and(|entry| std::ptr::eq(entry, plan))
}
