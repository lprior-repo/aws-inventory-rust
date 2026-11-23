mod chunk;
mod page;
mod selection;

use super::{
    PageParts, PageRequest, WorkLease, call, child, consumer_index::ConsumerIndex, cursor,
    resource_emissions, rows, tag_endpoint,
};
use crate::{SweepError, record, resource_projection};
use inventory_core::{
    catalog::runtime::{CallablePlan, Emission, PlanPurpose, RuntimeCatalog},
    inventory::{AccountId, DiscoveryContext, RegionId, TagObservation},
};

pub(super) const ROWS_PER_CHECKPOINT: usize = 32;
pub(super) const MAX_PAGE_ROWS: usize = 100_000;
const MAX_PAGE_REFERENCES: usize = 10_000;
const MAX_PAGE_CHILDREN: usize = 10_000;
const MAX_PAGE_TAG_PAIRS: usize = 50_000;

pub(super) struct Context<'a, 'catalog> {
    pub catalog: &'a RuntimeCatalog<'catalog>,
    pub account: &'a AccountId,
    pub home: &'a RegionId,
    pub executor: &'a inventory_engine::limiter::ApiExecutor,
    pub consumers: &'a ConsumerIndex<'a, 'catalog>,
    pub max_pages: u32,
}

pub(super) async fn checkpoint(
    executor: &inventory_engine::limiter::ApiExecutor,
) -> Result<(), SweepError> {
    tokio::task::yield_now().await;
    if executor.is_cancelled() {
        return Err(inventory_engine::limiter::ApiError::Cancelled.into());
    }
    Ok(())
}

pub(super) async fn prepare(
    context: &Context<'_, '_>,
    lease: &WorkLease,
    plan: &CallablePlan<'_>,
    cursor: cursor::Cursor,
    response: call::DecodedResponse,
    request: &PageRequest,
    ancestry: &DiscoveryContext,
) -> Result<PageParts, SweepError> {
    checkpoint(context.executor).await?;
    page::Page::new(context, lease, plan, response, request, ancestry)?
        .prepare(cursor)
        .await
}

pub(super) fn append(
    parts: &mut PageParts,
    next: PageParts,
    tag_pairs: &mut usize,
) -> Result<(), SweepError> {
    let pairs = tag_pairs_after(*tag_pairs, &next.tags)?;
    check_length(parts.references.len(), next.references.len(), MAX_PAGE_REFERENCES,
        "page exceeds the atomic reference bound")?;
    check_length(parts.children.len(), next.children.len(), MAX_PAGE_CHILDREN,
        "page exceeds the atomic child-work bound")?;
    append_rows(parts, next)?;
    *tag_pairs = pairs;
    Ok(())
}

fn tag_pairs_after(count: usize, tags: &[TagObservation]) -> Result<usize, SweepError> {
    tags.iter().try_fold(count, |count, tag| {
        count.checked_add(tag.tags().len())
            .filter(|count| *count <= MAX_PAGE_TAG_PAIRS)
            .ok_or(SweepError::Operation("page exceeds the atomic tag pair bound"))
    })
}

fn check_length(
    current: usize,
    next: usize,
    maximum: usize,
    message: &'static str,
) -> Result<(), SweepError> {
    if current.checked_add(next).is_none_or(|len| len > maximum) {
        return Err(SweepError::Operation(message));
    }
    Ok(())
}

fn append_rows(parts: &mut PageParts, next: PageParts) -> Result<(), SweepError> {
    parts.observations.try_reserve(next.observations.len())?;
    parts.references.try_reserve(next.references.len())?;
    parts.tags.try_reserve(next.tags.len())?;
    parts.children.try_reserve(next.children.len())?;
    parts.observations.extend(next.observations);
    parts.references.extend(next.references);
    parts.tags.extend(next.tags);
    parts.children.extend(next.children);
    Ok(())
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod stage_tests;
