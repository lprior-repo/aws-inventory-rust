mod assembly;
pub(crate) mod child;
pub(crate) mod consumer_index;
pub(crate) mod cursor;
mod preparation;
mod resource_emissions;
pub(crate) mod rows;
mod tag_endpoint;

mod settlement;
use inventory_core::{
    catalog::runtime::{CallablePlan, PlanPurpose},
    inventory::ScopeOutcome as StoredOutcome,
    record::Record,
};
use inventory_engine::inventory::{BinaryStore, PageCommit, StoreError, WorkLease};
use settlement::{finalize, finish_error, summary};

use super::{RuntimeContext, now_ms};
use crate::{ScopeOutcomes, SweepError, SweepSummary, call, input};

pub(super) async fn process(
    context: &RuntimeContext<'_, '_, impl crate::CredentialSource>,
    lease: WorkLease,
) -> Result<SweepSummary, SweepError> {
    let result = process_owned(context, lease).await;
    if let Ok(summary) = &result {
        context.progress.scope(summary);
    }
    result
}

async fn process_owned(
    context: &RuntimeContext<'_, '_, impl crate::CredentialSource>,
    lease: WorkLease,
) -> Result<SweepSummary, SweepError> {
    if context.bootstrap.executor.is_cancelled() {
        return finish_error(
            context.store,
            lease,
            inventory_engine::limiter::ApiError::Cancelled.into(),
        )
        .await;
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
    context: &RuntimeContext<'_, '_, impl crate::CredentialSource>,
    lease: WorkLease,
) -> Result<SweepSummary, SweepError> {
    let plan = match lookup_plan(context, &lease) {
        Ok(plan) => plan,
        Err(error) => return finish_error(context.store, lease, error).await,
    };
    let ancestry = match preparation::ancestry(lease.context(), lease.scope().context()) {
        Ok(ancestry) => ancestry,
        Err(error) => return finish_error(context.store, lease, error).await,
    };
    let decoded = if matches!(plan.purpose(), PlanPurpose::TagEndpoint(_)) {
        tag_endpoint::decode_cursor(lease.token())
    } else {
        cursor::Cursor::decode(lease.token())
    };
    let cursor = match decoded {
        Ok(cursor) => cursor,
        Err(error) => return finish_error(context.store, lease, error).await,
    };
    if cursor.partial || cursor.pages >= context.bootstrap.config.max_pages {
        return finalize(context.store, lease, StoredOutcome::Partial).await;
    }
    let (endpoint, url, base, fields, request_region) =
        match preparation::prepare(context, &lease, plan, &cursor).await {
            Ok(prepared) => prepared,
            Err(error) => return finish_error(context.store, lease, error).await,
        };
    let request = match input::merge_input(&base, &fields) {
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
    let request = PageRequest {
        record: request,
        region: request_region,
    };
    commit_response(context, lease, plan, cursor, response, request, &ancestry).await
}

fn lookup_plan<'a, 'catalog>(
    context: &'a RuntimeContext<'_, 'catalog, impl crate::CredentialSource>,
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
    if !matches!(
        plan.purpose(),
        PlanPurpose::Enumeration | PlanPurpose::TagEndpoint(_)
    ) {
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

struct PageRequest {
    record: Record,
    region: inventory_core::inventory::RegionId,
}

async fn commit_response(
    context: &RuntimeContext<'_, '_, impl crate::CredentialSource>,
    lease: WorkLease,
    plan: &CallablePlan<'_>,
    cursor: cursor::Cursor,
    response: call::DecodedResponse,
    request: PageRequest,
    ancestry: &inventory_core::inventory::DiscoveryContext,
) -> Result<SweepSummary, SweepError> {
    let assembly = assembly::Context {
        catalog: context.bootstrap.catalog,
        account: &context.account,
        home: context.bootstrap.home,
        executor: context.bootstrap.executor,
        consumers: &context.consumers,
        max_pages: context.bootstrap.config.max_pages,
    };
    let parts = match assembly::prepare(
        &assembly, &lease, plan, cursor, response, &request, ancestry,
    )
    .await
    {
        Ok(parts) => parts,
        Err(error) => return finish_error(context.store, lease, error).await,
    };
    commit_parts(context, lease, parts).await
}

async fn commit_parts(
    context: &RuntimeContext<'_, '_, impl crate::CredentialSource>,
    lease: WorkLease,
    parts: PageParts,
) -> Result<SweepSummary, SweepError> {
    if let Err(error) = assembly::checkpoint(context.bootstrap.executor).await {
        return finish_error(context.store, lease, error).await;
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
    context: &RuntimeContext<'_, '_, impl crate::CredentialSource>,
    commit: PageCommit,
    done: bool,
    rows: u64,
) -> Result<SweepSummary, SweepError> {
    let fallback_lease = commit.lease.clone();
    match context.store.commit_page(commit).await {
        Ok(result) => {
            let summary = report_commit(result, done, rows)?;
            context.progress.emit(|| crate::ProgressEvent::Page {
                rows: summary.rows,
                resource_writes: summary.resource_writes,
            });
            Ok(summary)
        }
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
