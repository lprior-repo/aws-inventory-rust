pub(in crate::sweep_runtime) mod child;
pub(in crate::sweep_runtime) mod cursor;
pub(in crate::sweep_runtime) mod rows;
mod resource_emissions;
mod tag_endpoint;
mod preparation;
mod assembly;
pub(in crate::sweep_runtime) mod consumer_index;

use inventory_core::{
    catalog::runtime::{CallablePlan, PlanPurpose},
    inventory::ScopeOutcome as StoredOutcome,
    record::Record,
};
use inventory_engine::inventory::{BinaryStore, PageCommit, StoreError, WorkLease};

use super::{RuntimeContext, now_ms};
use crate::sweep_runtime::{ScopeOutcomes, SweepError, SweepSummary, call, input};

pub(super) async fn process(
    context: &RuntimeContext<'_, '_>,
    lease: WorkLease,
) -> Result<SweepSummary, SweepError> {
    if context.bootstrap.executor.is_cancelled() {
        return finish_error(context.store,lease,inventory_engine::limiter::ApiError::Cancelled.into()).await;
    }
    if lease.scope().account() != &context.account {
        return finish_error(
            context.store,
            lease,
            SweepError::Operation("resume account does not match stored scope"),
        )
        .await;
    }
    process_lease(context, lease).await
}

async fn process_lease(
    context: &RuntimeContext<'_, '_>,
    lease: WorkLease,
) -> Result<SweepSummary, SweepError> {
    let plan = match lookup_plan(context, &lease) {
        Ok(plan) => plan,
        Err(error) => return finish_error(context.store, lease, error).await,
    };
    let ancestry = match preparation::ancestry(lease.context(),lease.scope().context()) {
        Ok(ancestry) => ancestry,
        Err(error) => return finish_error(context.store,lease,error).await,
    };
    let decoded = if matches!(plan.purpose(), PlanPurpose::TagEndpoint(_)) {
        tag_endpoint::decode_cursor(lease.token())
    } else { cursor::Cursor::decode(lease.token()) };
    let cursor = match decoded {
        Ok(cursor) => cursor,
        Err(error) => return finish_error(context.store, lease, error).await,
    };
    if cursor.partial || cursor.pages >= context.bootstrap.config.max_pages {
        return finalize(context.store, lease, StoredOutcome::Partial).await;
    }
    let (endpoint, url, base, fields, request_region) = match preparation::prepare(context, &lease, plan, &cursor).await {
        Ok(prepared) => prepared,
        Err(error) => return finish_error(context.store, lease, error).await,
    };
    let request = match input::merge_input(&base,&fields) {
        Ok(request) => request,
        Err(error) => return finish_error(context.store, lease, error).await,
    };
    let call = call::Call {
        client: context.bootstrap.client,
        executor: context.bootstrap.executor,
        credentials: context.bootstrap.credentials,
        plan,
        endpoint: &endpoint,
        url: &url,
        base: &base,
        fields: &fields,
    };
    let response = match call.run().await {
        Ok(response) => response,
        Err(error) => return finish_error(context.store, lease, error).await,
    };
    let request = PageRequest { record:request, region:request_region };
    commit_response(context, lease, plan, cursor, response, request, &ancestry).await
}

fn lookup_plan<'a, 'catalog>(
    context: &'a RuntimeContext<'_, 'catalog>,
    lease: &WorkLease,
) -> Result<&'a CallablePlan<'catalog>, SweepError> {
    let scope = lease.scope();
    let plan = context
        .bootstrap
        .catalog
        .lookup(scope.service().as_str(), scope.operation().as_str())
        .ok_or(SweepError::Operation(
            "persisted work refers to an unknown reviewed operation",
        ))?;
    if !matches!(plan.purpose(), PlanPurpose::Enumeration | PlanPurpose::TagEndpoint(_)) {
        return Err(SweepError::Operation(
            "persisted work is not a reviewed enumeration",
        ));
    }
    super::routing::execution_branch(plan, scope.branch())?;
    Ok(plan)
}


#[derive(Default)]
struct PageParts {
    observations: Vec<inventory_core::inventory::Observation>,
    references: Vec<inventory_core::inventory::UnresolvedReference>,
    tags: Vec<inventory_core::inventory::TagObservation>,
    children: Vec<inventory_engine::inventory::WorkSpec>,
    next: Option<cursor::Advance>,
    rows: u64,
}

struct PageRequest { record:Record, region:inventory_core::inventory::RegionId }

async fn commit_response(
    context: &RuntimeContext<'_, '_>,
    lease: WorkLease,
    plan: &CallablePlan<'_>,
    cursor: cursor::Cursor,
    response: call::DecodedResponse,
    request: PageRequest,
    ancestry: &inventory_core::inventory::DiscoveryContext,
) -> Result<SweepSummary, SweepError> {
    let assembly = assembly::Context { catalog:context.bootstrap.catalog, account:&context.account,
        home:context.bootstrap.home,executor:context.bootstrap.executor,consumers:&context.consumers,
        max_pages:context.bootstrap.config.max_pages };
    let parts = match assembly::prepare(&assembly, &lease, plan, cursor, response, &request, ancestry).await {
        Ok(parts) => parts,
        Err(error) => return finish_error(context.store, lease, error).await,
    };
    commit_parts(context, lease, parts).await
}


async fn commit_parts(
    context: &RuntimeContext<'_, '_>,
    lease: WorkLease,
    parts: PageParts,
) -> Result<SweepSummary, SweepError> {
    if let Err(error) = assembly::checkpoint(context.bootstrap.executor).await {
        return finish_error(context.store,lease,error).await;
    }
    let done = parts.next.is_none();
    let continuation_digest = parts
        .next
        .as_ref()
        .and_then(|advance| advance.continuation_digest);
    let commit = PageCommit {
        lease,
        observations: parts.observations,
        edges: Vec::new(),
        unresolved: parts.references,
        tags: parts.tags,
        next_token: parts.next.map(|advance| advance.cursor.into_bytes()),
        continuation_digest,
        children: parts.children,
        done,
        observed_at_ms: now_ms()?,
    };
    commit_page(context, commit, done, parts.rows).await
}

async fn commit_page(
    context: &RuntimeContext<'_, '_>,
    commit: PageCommit,
    done: bool,
    rows: u64,
) -> Result<SweepSummary, SweepError> {
    let fallback_lease = commit.lease.clone();
    match context.store.commit_page(commit).await {
        Ok(result) => report_commit(result, done, rows),
        Err(StoreError::StaleLease) => Ok(SweepSummary::default()),
        Err(error) => finish_error(context.store, fallback_lease, SweepError::Store(error)).await,
    }
}

fn report_commit(
    result: inventory_engine::inventory::CommitResult,
    done: bool,
    rows: u64,
) -> Result<SweepSummary, SweepError> {
    let writes = result
        .stats
        .resources_inserted
        .checked_add(result.stats.resources_changed)
        .and_then(|value| value.checked_add(result.stats.resources_deleted))
        .ok_or(SweepError::Operation("resource write count overflow"))?;
    if result.repeated_token {
        return Ok(summary(StoredOutcome::Partial, rows, writes));
    }
    if done {
        Ok(summary(StoredOutcome::Complete, rows, writes))
    } else {
        Ok(SweepSummary {
            resource_writes: writes,
            rows,
            ..SweepSummary::default()
        })
    }
}

async fn finish_error(
    store: &BinaryStore,
    lease: WorkLease,
    error: SweepError,
) -> Result<SweepSummary, SweepError> {
    let outcome = match &error {
        SweepError::Api(inventory_engine::limiter::ApiError::Denied(_)) => {
            StoredOutcome::Inaccessible
        }
        SweepError::Api(inventory_engine::limiter::ApiError::UnsupportedType(_)) => {
            StoredOutcome::Unsupported
        }
        SweepError::Api(inventory_engine::limiter::ApiError::Cancelled) => {
            finalize(store,lease,StoredOutcome::Partial).await?;
            return Err(error);
        }
        _ => StoredOutcome::Failed,
    };
    finalize(store, lease, outcome).await
}

async fn finalize(
    store: &BinaryStore,
    lease: WorkLease,
    outcome: StoredOutcome,
) -> Result<SweepSummary, SweepError> {
    match store.finish_scope(lease, outcome, now_ms()?).await {
        Ok(stats) => Ok(summary(outcome, 0, stats.resources_deleted)),
        Err(StoreError::StaleLease) => Ok(SweepSummary::default()),
        Err(error) => Err(SweepError::Store(error)),
    }
}

fn summary(outcome: StoredOutcome, rows: u64, writes: u64) -> SweepSummary {
    let mut outcomes = ScopeOutcomes::default();
    outcomes.record(outcome);
    SweepSummary {
        resource_writes: writes,
        rows,
        scopes: 1,
        outcomes,
        graph_pending: 0,
    }
}
