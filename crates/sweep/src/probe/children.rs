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
        page: &RetainedPage,
        tasks: &mut Vec<Bound<'a, 'catalog>>,
    ) -> Result<(), SweepError> {
        let branch = branch_at(parent, page.branch);
        let branch_name = branch.map(|branch| branch.discriminator.as_str());
        let mut pending = Vec::new();
        for plan in self.index.binding_candidates_for_branch(
            parent.service_name(),
            parent.operation_name(),
            branch_name,
        ) {
            let Some((_, state)) = self.states.get_mut(&key(plan)) else {
                continue;
            };
            if state.requests < self.limit && consumes(plan, parent, branch_name) {
                state.parent_items = true;
                pending.try_reserve(1)?;
                pending.push(plan);
            }
        }
        if pending.is_empty() {
            return Ok(());
        }
        let context = BindingContext {
            parent_plan: parent,
            account: self.scope.account,
        };
        let samples = match page.samples(parent, branch) {
            Ok(samples) => samples,
            Err(error) => {
                for plan in pending {
                    self.state_mut(plan)?.note_binding_error(&error);
                }
                return Ok(());
            }
        };
        let mut remaining = pending.len();
        for sample in samples {
            for plan in &pending {
                let limit = self.limit;
                let home = self.scope.region;
                let state = self.state_mut(plan)?;
                if state.requests < limit {
                    state.admit(plan, &sample, context, home, tasks)?;
                    if state.requests == limit {
                        remaining = remaining.saturating_sub(1);
                    }
                }
            }
            if remaining == 0 {
                break;
            }
        }
        Ok(())
    }

    fn state_mut(&mut self, plan: Plan<'a, 'catalog>) -> Result<&mut State, SweepError> {
        self.states
            .get_mut(&key(plan))
            .map(|(_, state)| state)
            .ok_or(SweepError::Operation("missing probe child state"))
    }

    fn record(
        &mut self,
        mut next: Vec<Parent<'a, 'catalog>>,
        (plan, result): (Plan<'a, 'catalog>, Result<ProbeAnswer, SweepError>),
    ) -> Result<Vec<Parent<'a, 'catalog>>, SweepError> {
        let state = self.state_mut(plan)?;
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
        sample: &Result<Sample<'_>, SweepError>,
        context: BindingContext<'_, '_>,
        home: &RegionId,
        tasks: &mut Vec<Bound<'a, 'catalog>>,
    ) -> Result<(), SweepError> {
        let task = match sample {
            Ok(sample) => sample.bind(plan, context, home),
            Err(error) => {
                self.note_binding_error(error);
                return Ok(());
            }
        };
        match task {
            Ok(Some(task)) => {
                tasks.try_reserve(1)?;
                tasks.push(task);
                self.requests = self.requests.saturating_add(1);
            }
            Ok(None) => {}
            Err(error) => self.note_binding_error(&error),
        }
        Ok(())
    }

    fn note_binding_error(&mut self, error: &SweepError) {
        self.binding_error.get_or_insert_with(|| error.to_string());
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

impl RetainedPage {
    fn key(&self) -> (&str, &str) {
        (self.service.as_str(), self.operation.as_str())
    }

    fn samples<'a>(
        &'a self,
        plan: &'a CallablePlan<'_>,
        branch: Option<&'a ExecutionBranch>,
    ) -> Result<impl Iterator<Item = Result<Sample<'a>, SweepError>> + 'a, SweepError> {
        let (_, selected) = record::item_selection(&self.response, plan.items())?;
        let branch_name = branch.map(|branch| branch.discriminator.as_str());
        Ok(selected.enumerate().filter_map(move |(ordinal, row)| {
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
                let ancestry = self
                    .ancestry
                    .push(frame)
                    .map_err(|_| SweepError::Operation("discovery ancestry exceeds bounds"))?;
                Ok(Some(Sample {
                    row,
                    region: &self.region,
                    branch: branch_name,
                    ancestry,
                }))
            })
            .transpose()
        }))
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
        let mut first_error = None;
        for branch in branches(plan) {
            match self.bind_branch(plan, &context, &row, home, branch) {
                Ok(Some(bound)) => return Ok(Some(bound)),
                Ok(None) => {}
                Err(error) => {
                    first_error.get_or_insert(error);
                }
            }
        }
        first_error.map_or(Ok(None), Err)
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
