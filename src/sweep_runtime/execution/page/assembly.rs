use super::{PageParts, PageRequest, WorkLease, call, child, consumer_index::ConsumerIndex, cursor, rows, tag_endpoint};
use crate::sweep_runtime::{SweepError, record, resource_projection};
use inventory_core::{catalog::runtime::{CallablePlan, Emission, PlanPurpose, RuntimeCatalog}, inventory::{AccountId, DiscoveryContext, RegionId, TagObservation}};

const ROWS_PER_CHECKPOINT: usize = 32;
const MAX_PAGE_ROWS: usize = 100_000;
const MAX_PAGE_REFERENCES: usize = 10_000;
const MAX_PAGE_CHILDREN: usize = 10_000;
const MAX_PAGE_TAG_PAIRS: usize = 50_000;

pub(super) struct Context<'a,'catalog> {
    pub catalog: &'a RuntimeCatalog<'catalog>,
    pub account: &'a AccountId,
    pub home: &'a RegionId,
    pub executor: &'a inventory_engine::limiter::ApiExecutor,
    pub consumers: &'a ConsumerIndex<'a,'catalog>,
    pub max_pages: u32,
}

pub(super) async fn checkpoint(executor: &inventory_engine::limiter::ApiExecutor) -> Result<(),SweepError> {
    tokio::task::yield_now().await;
    if executor.is_cancelled() {
        return Err(inventory_engine::limiter::ApiError::Cancelled.into());
    }
    Ok(())
}

pub(super) async fn prepare(
    context: &Context<'_,'_>, lease: &WorkLease, plan: &CallablePlan<'_>,
    cursor: cursor::Cursor, response: call::DecodedResponse, request: &PageRequest,
    ancestry: &DiscoveryContext,
) -> Result<PageParts,SweepError> {
    checkpoint(context.executor).await?;
    let branch = super::super::routing::execution_branch(plan, lease.scope().branch())?;
    let emission = plan.effective_emission(branch);
    if matches!(plan.purpose(),PlanPurpose::TagEndpoint(_)) {
        return tag_endpoint::prepare_page(context.catalog,context.account,context.max_pages,lease,plan,cursor,response);
    }
    let (total_rows,mut selected) = record::item_selection(&response.record,plan.items())?;
    if total_rows > MAX_PAGE_ROWS {
        return Err(SweepError::Operation("page exceeds the atomic observation bound"));
    }
    let mut budget = child::PageContextBudget::new(ancestry,total_rows)?;
    let child_context = child::Context::new(plan,lease.sweep(),
        context.account,context.home,context.consumers);
    let mut parts = PageParts::default();
    let mut tag_pairs = 0;
    let mut offset = 0_usize;
    loop {
        checkpoint(context.executor).await?;
        let mut ordinals = [0_usize;ROWS_PER_CHECKPOINT];
        let mut retained_ordinals = ordinals.iter_mut();
        let mut raw_count = 0_usize;
        let mut items = Vec::new();
        for (ordinal,row) in selected.by_ref().take(ROWS_PER_CHECKPOINT).enumerate() {
            raw_count = raw_count.checked_add(1)
                .ok_or(SweepError::Operation("discovery item ordinal exceeds bound"))?;
            let row = row?;
            if !crate::sweep_runtime::row_policy::selected(plan,branch,&row,&request.record,&response.record)? { continue; }
            let ordinal = offset.checked_add(ordinal)
                .ok_or(SweepError::Operation("discovery item ordinal exceeds bound"))?;
            *retained_ordinals.next().ok_or(SweepError::Operation("discovery item ordinal exceeds bound"))? = ordinal;
            items.try_reserve(1)?;
            items.push(resource_projection::project(plan,&row)?);
        }
        if raw_count == 0 { break; }
        offset = offset.checked_add(raw_count)
            .ok_or(SweepError::Operation("discovery item ordinal exceeds bound"))?;
        if items.is_empty() { continue; }
        let contexts = child::contexts_at(plan,items.iter().zip(ordinals.iter()).map(|(row,ordinal)| (*ordinal,row)),
            &request.record,&response.record,ancestry,lease.scope(),&mut budget)?;
        let observations = if emission == Emission::DiscoveryOnly {
            Vec::new()
        } else {
            rows::observations(context.catalog,
                record::IdentityContext { account:context.account,region:&request.region },plan,
                lease.scope().region(),&items,&request.record,&contexts)?
        };
        let references = rows::references(context.catalog,plan,&request.region,&items,
            &observations,&request.record,lease.scope().parent())?;
        let tags = if emission == Emission::DiscoveryOnly {
            Vec::new()
        } else {
            rows::tags(plan,&items,&observations)?
        };
        let children = child::work(&child_context,&items,&observations,&request.record,&contexts,
            lease.scope(),&mut budget)?;
        append(&mut parts,PageParts { observations,references,tags,children,..PageParts::default() },&mut tag_pairs)?;
    }
    checkpoint(context.executor).await?;
    parts.next = cursor.next(plan.pagination(),&response.record,&response.headers,context.max_pages)?;
    parts.rows = u64::try_from(parts.observations.len())
        .map_err(|_| SweepError::Operation("row count exceeds summary range"))?;
    Ok(parts)
}

fn append(parts: &mut PageParts, next: PageParts, tag_pairs: &mut usize) -> Result<(),SweepError> {
    let pairs = next.tags.iter().try_fold(*tag_pairs,|count,tag: &TagObservation| {
        count.checked_add(tag.tags().len()).filter(|count| *count <= MAX_PAGE_TAG_PAIRS)
            .ok_or(SweepError::Operation("page exceeds the atomic tag pair bound"))
    })?;
    if parts.references.len().checked_add(next.references.len()).is_none_or(|len| len > MAX_PAGE_REFERENCES) {
        return Err(SweepError::Operation("page exceeds the atomic reference bound"));
    }
    if parts.children.len().checked_add(next.children.len()).is_none_or(|len| len > MAX_PAGE_CHILDREN) {
        return Err(SweepError::Operation("page exceeds the atomic child-work bound"));
    }
    parts.observations.try_reserve(next.observations.len())?;
    parts.references.try_reserve(next.references.len())?;
    parts.tags.try_reserve(next.tags.len())?;
    parts.children.try_reserve(next.children.len())?;
    parts.observations.extend(next.observations);
    parts.references.extend(next.references);
    parts.tags.extend(next.tags);
    parts.children.extend(next.children);
    *tag_pairs = pairs;
    Ok(())
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod stage_tests;
